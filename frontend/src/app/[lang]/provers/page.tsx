'use client';

import dynamic from 'next/dynamic';
import { useCallback, useMemo } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { CopyButton } from '@/components/Hash';
import { StatsCard, StatsCardSkeleton, StatsRow } from '@/components/StatsCard';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { useProvers } from '@/hooks/useApi';
import { cn } from '@/lib/utils';
import { useFmt, useT, type Fmt } from '@/i18n/client';
import type { NodeInfo, ProverView } from '@/types';

// Leaflet touches `window` at import time, so the map is client-only.
const NodeMap = dynamic(() => import('@/components/NodeMap'), {
  ssr: false,
  loading: () => (
    <div className="card flex h-[480px] items-center justify-center">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-border border-t-accent" />
    </div>
  ),
});

/** One row of the members table: a host and the prover it serves. */
interface MemberRow {
  key: string;
  label: string;
  prover: ProverView;
  geo: NodeInfo['geo'];
}

const memberColumnsFor = (t: (key: string) => string, fmt: Fmt): Column<MemberRow>[] => [
  {
    key: 'host',
    header: t('provers.columns.host'),
    render: (m) => <span className="font-mono text-strong">{m.label}</span>,
  },
  {
    key: 'prover',
    header: t('provers.columns.prover'),
    render: (m) => <span className="text-soft">{m.prover.name}</span>,
  },
  {
    key: 'location',
    header: t('provers.columns.location'),
    render: (m) => (
      <span className={cn(m.geo ? 'text-soft' : 'text-mute')}>
        {m.geo ? fmt.location(m.geo) : t('provers.notLocated')}
      </span>
    ),
  },
  {
    key: 'org',
    header: t('provers.columns.network'),
    render: (m) => (m.geo?.org ? <span className="text-soft">{m.geo.org}</span> : <span className="text-mute">—</span>),
  },
];

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** The map popup for a member host: the host, its prover, where it is. */
function memberPopup(node: NodeInfo, fmt: Fmt): string {
  const [label, prover] = node.peer_id.split('\u0000');
  return (
    '<div style="min-width:190px;font-size:12px;line-height:1.55">' +
    `<div style="font-weight:500;color:var(--color-text-strong)">${escapeHtml(label)}</div>` +
    `<div style="color:var(--color-text-soft)">${escapeHtml(prover ?? '')}</div>` +
    `<div style="margin-top:5px;color:var(--color-text)">${escapeHtml(fmt.location(node.geo))}</div>` +
    (node.geo?.org ? `<div style="color:var(--color-text-mute)">${escapeHtml(node.geo.org)}</div>` : '') +
    '</div>'
  );
}

function StatusBadge({ prover }: { prover: ProverView }) {
  const { t } = useT();
  if (prover.up) return <span className="badge badge-bridge">{t('provers.status.answering')}</span>;
  if (prover.checked_at_ms === null)
    return <span className="badge badge-neutral">{t('provers.status.notChecked')}</span>;
  return <span className="badge badge-neutral">{t('provers.status.notAnswering')}</span>;
}

function ProverPanel({ prover }: { prover: ProverView }) {
  const info = prover.info;
  const q = info?.queue ?? null;
  const { t, tp } = useT();
  const fmt = useFmt();
  return (
    <Panel title={prover.name} actions={<StatusBadge prover={prover} />}>
      <DetailRow label={t('provers.panel.endpoint')}>
        <span className="inline-flex items-center gap-1.5">
          <span className="break-all font-mono text-text">{prover.url}</span>
          <CopyButton value={prover.url} />
        </span>
      </DetailRow>
      <DetailRow label={t('provers.panel.operator')}>{prover.operator}</DetailRow>
      <DetailRow label={t('provers.panel.fingerprint')}>
        <span className="inline-flex flex-wrap items-center gap-2">
          <span className="font-mono text-strong">{prover.fingerprint}</span>
          {prover.fingerprint_matches === false && (
            <span className="badge badge-neutral">
              {info?.kem_fingerprint
                ? t('provers.panel.endpointReports', { fingerprint: info.kem_fingerprint })
                : t('provers.panel.endpointReportsOther')}
            </span>
          )}
        </span>
      </DetailRow>
      <DetailRow label={t('provers.panel.capacity')}>
        {q ? (
          <span>
            {tp('provers.panel.capacityValue', q.max, {
              proving: fmt.number(q.proving),
              depth: fmt.number(q.depth),
            })}
            <span className="text-mute"> {t('provers.panel.capacityNote')}</span>
          </span>
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label={t('provers.panel.fee')}>
        {info ? (
          info.fee ? (
            <span className="font-mono">{t('provers.panel.feeValue', { amount: fmt.amount(info.fee.amount) })}</span>
          ) : (
            <span>{t('provers.panel.free')}</span>
          )
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label={t('provers.panel.proves')}>
        {info ? (
          <span className="text-soft">
            {t('provers.panel.provesValue', {
              jobs:
                info.witness_kinds.length > 0
                  ? t('provers.panel.jobs', { kinds: info.witness_kinds.join(', ').replace('_', '-') })
                  : '—',
              backend: info.backend === 'cuda' ? 'GPU' : 'CPU',
              version: info.version ?? t('provers.panel.unknownBuild'),
            })}
          </span>
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label={t('provers.panel.lastChecked')}>
        {prover.checked_at_ms === null ? (
          <span className="text-mute">—</span>
        ) : (
          <span className="text-soft">
            {fmt.dateTime(prover.checked_at_ms)}
            {!prover.up && prover.last_up_ms !== null && (
              <span className="text-mute">
                {' '}
                · {t('provers.panel.lastAnswered', { time: fmt.dateTime(prover.last_up_ms) })}
              </span>
            )}
          </span>
        )}
      </DetailRow>
      {!prover.up && prover.error && (
        <DetailRow label={t('provers.panel.error')}>
          <span className="break-all text-xs text-mute">{prover.error}</span>
        </DetailRow>
      )}
      {prover.pairing_url && (
        <DetailRow label={t('provers.panel.pairing')}>
          <a href={prover.pairing_url} target="_blank" rel="noopener noreferrer" className="link break-all">
            {prover.pairing_url}
          </a>
        </DetailRow>
      )}
    </Panel>
  );
}

export default function ProversPage() {
  const { data: provers, error, isLoading, mutate } = useProvers();
  const { t, rich } = useT();
  const fmt = useFmt();
  const memberColumns = memberColumnsFor(t, fmt);
  const popup = useCallback((node: NodeInfo) => memberPopup(node, fmt), [fmt]);

  const members = useMemo<MemberRow[]>(
    () =>
      (provers ?? []).flatMap((p) =>
        p.members.map((m) => ({ key: `${p.url}-${m.label}`, label: m.label, prover: p, geo: m.geo }))
      ),
    [provers]
  );

  // The map draws NodeInfo; a member becomes one, its label and prover packed in `peer_id` for
  // the popup. No address is carried: the API serves none.
  const mapNodes = useMemo<NodeInfo[]>(
    () =>
      members.map((m) => ({
        peer_id: `${m.label}\u0000${m.prover.name}`,
        ip: null,
        port: null,
        connected_secs: null,
        is_self: false,
        role: 'validator',
        geo: m.geo,
      })),
    [members]
  );

  const summary = useMemo(() => {
    const located = members.filter((m) => m.geo !== null);
    const countries = new Set(
      located.map((m) => m.geo?.country_code ?? m.geo?.country).filter((v): v is string => !!v)
    );
    const up = (provers ?? []).filter((p) => p.up).length;
    const slots = (provers ?? []).reduce((n, p) => n + (p.up ? p.info?.queue?.max ?? 0 : 0), 0);
    return { members: members.length, countries: countries.size, up, provers: provers?.length ?? 0, slots };
  }, [members, provers]);

  if (error && !provers) {
    return (
      <>
        <PageHeader title={t('provers.title')} />
        <ErrorState message={t('provers.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('provers.title')}
        subtitle={t('provers.subtitle')}
      />

      <StatsRow columns={4}>
        {isLoading && !provers ? (
          Array.from({ length: 4 }).map((_, i) => <StatsCardSkeleton key={i} />)
        ) : (
          <>
            <StatsCard
              title={t('provers.stats.provers')}
              value={fmt.number(summary.provers)}
              subtitle={t('provers.stats.answering', { count: fmt.number(summary.up) })}
            />
            <StatsCard title={t('provers.stats.hosts')} value={fmt.number(summary.members)} />
            <StatsCard title={t('provers.stats.countries')} value={fmt.number(summary.countries)} />
            <StatsCard
              title={t('provers.stats.slots')}
              value={fmt.number(summary.slots)}
              subtitle={t('provers.stats.slotsSub')}
            />
          </>
        )}
      </StatsRow>

      <NodeMap nodes={mapNodes} popup={popup} />

      <section className="space-y-4">
        <h2 className="chip">{t('provers.hosts')}</h2>
        <DataTable
          columns={memberColumns}
          data={members}
          keyExtractor={(m) => m.key}
          isLoading={isLoading && !provers}
          emptyMessage={t('provers.empty')}
        />
        <p className="text-xs text-mute">
          {t('provers.hostsHelp')}
        </p>
      </section>

      <div className="space-y-6">
        {(provers ?? []).map((p) => (
          <ProverPanel key={p.url} prover={p} />
        ))}
      </div>

      <p className="text-xs text-mute">
        {rich('provers.help')}
      </p>
    </div>
  );
}
