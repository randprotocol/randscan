'use client';

import dynamic from 'next/dynamic';
import { useMemo } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { CopyButton } from '@/components/Hash';
import { StatsCard, StatsCardSkeleton, StatsRow } from '@/components/StatsCard';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { useProvers } from '@/hooks/useApi';
import { cn, formatAmount, formatDateTime, formatLocation, formatNumber } from '@/lib/utils';
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

const memberColumns: Column<MemberRow>[] = [
  {
    key: 'host',
    header: 'Host',
    render: (m) => <span className="font-mono text-strong">{m.label}</span>,
  },
  {
    key: 'prover',
    header: 'Prover',
    render: (m) => <span className="text-soft">{m.prover.name}</span>,
  },
  {
    key: 'location',
    header: 'Location',
    render: (m) => (
      <span className={cn(m.geo ? 'text-soft' : 'text-mute')}>
        {m.geo ? formatLocation(m.geo) : 'Not located yet'}
      </span>
    ),
  },
  {
    key: 'org',
    header: 'Network',
    render: (m) => (m.geo?.org ? <span className="text-soft">{m.geo.org}</span> : <span className="text-mute">—</span>),
  },
];

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** The map popup for a member host: the host, its prover, where it is. */
function memberPopup(node: NodeInfo): string {
  const [label, prover] = node.peer_id.split('\u0000');
  return (
    '<div style="min-width:190px;font-size:12px;line-height:1.55">' +
    `<div style="font-weight:500;color:var(--color-text-strong)">${escapeHtml(label)}</div>` +
    `<div style="color:var(--color-text-soft)">${escapeHtml(prover ?? '')}</div>` +
    `<div style="margin-top:5px;color:var(--color-text)">${escapeHtml(formatLocation(node.geo))}</div>` +
    (node.geo?.org ? `<div style="color:var(--color-text-mute)">${escapeHtml(node.geo.org)}</div>` : '') +
    '</div>'
  );
}

function StatusBadge({ prover }: { prover: ProverView }) {
  if (prover.up) return <span className="badge badge-bridge">Answering</span>;
  if (prover.checked_at_ms === null) return <span className="badge badge-neutral">Not checked yet</span>;
  return <span className="badge badge-neutral">Not answering</span>;
}

function ProverPanel({ prover }: { prover: ProverView }) {
  const info = prover.info;
  const q = info?.queue ?? null;
  return (
    <Panel title={prover.name} actions={<StatusBadge prover={prover} />}>
      <DetailRow label="Endpoint">
        <span className="inline-flex items-center gap-1.5">
          <span className="break-all font-mono text-text">{prover.url}</span>
          <CopyButton value={prover.url} />
        </span>
      </DetailRow>
      <DetailRow label="Operator">{prover.operator}</DetailRow>
      <DetailRow label="Fingerprint">
        <span className="inline-flex flex-wrap items-center gap-2">
          <span className="font-mono text-strong">{prover.fingerprint}</span>
          {prover.fingerprint_matches === false && (
            <span className="badge badge-neutral">
              the endpoint reports {info?.kem_fingerprint ?? 'another key'}
            </span>
          )}
        </span>
      </DetailRow>
      <DetailRow label="Capacity">
        {q ? (
          <span>
            <span className="font-mono text-strong">{formatNumber(q.proving)}</span> proving ·{' '}
            <span className="font-mono">{formatNumber(q.depth)}</span> waiting ·{' '}
            <span className="font-mono">{formatNumber(q.max)}</span>{' '}
            {q.max === 1 ? 'slot' : 'slots'}
            <span className="text-mute"> (one bundle at a time per member)</span>
          </span>
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label="Fee">
        {info ? (
          info.fee ? (
            <span className="font-mono">{formatAmount(info.fee.amount)} a bundle</span>
          ) : (
            <span>Free</span>
          )
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label="Proves">
        {info ? (
          <span className="text-soft">
            {info.witness_kinds.length > 0 ? info.witness_kinds.join(', ').replace('_', '-') + ' jobs' : '—'} on{' '}
            {info.backend === 'cuda' ? 'GPU' : 'CPU'} · build {info.version ?? 'unknown'}
          </span>
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label="Last checked">
        {prover.checked_at_ms === null ? (
          <span className="text-mute">—</span>
        ) : (
          <span className="text-soft">
            {formatDateTime(prover.checked_at_ms)}
            {!prover.up && prover.last_up_ms !== null && (
              <span className="text-mute"> · last answered {formatDateTime(prover.last_up_ms)}</span>
            )}
          </span>
        )}
      </DetailRow>
      {!prover.up && prover.error && (
        <DetailRow label="Error">
          <span className="break-all text-xs text-mute">{prover.error}</span>
        </DetailRow>
      )}
      {prover.pairing_url && (
        <DetailRow label="Pairing">
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
        <PageHeader title="Provers" />
        <ErrorState message="Could not load the provers." onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Provers"
        subtitle="Delegated provers: where a wallet that cannot prove itself sends its bundle, and where those machines are"
      />

      <StatsRow columns={4}>
        {isLoading && !provers ? (
          Array.from({ length: 4 }).map((_, i) => <StatsCardSkeleton key={i} />)
        ) : (
          <>
            <StatsCard
              title="Provers"
              value={formatNumber(summary.provers)}
              subtitle={`${formatNumber(summary.up)} answering`}
            />
            <StatsCard title="Prover hosts" value={formatNumber(summary.members)} />
            <StatsCard title="Countries" value={formatNumber(summary.countries)} />
            <StatsCard title="Job slots" value={formatNumber(summary.slots)} subtitle="free and busy, answering provers" />
          </>
        )}
      </StatsRow>

      <NodeMap nodes={mapNodes} popup={memberPopup} />

      <section className="space-y-4">
        <h2 className="chip">Prover hosts</h2>
        <DataTable
          columns={memberColumns}
          data={members}
          keyExtractor={(m) => m.key}
          isLoading={isLoading && !provers}
          emptyMessage="No prover is known to this explorer"
        />
        <p className="text-xs text-mute">
          A pool answers at one address and hands each job to a member host with a free slot, so a
          host is placed here by the explorer&apos;s own list, not discovered on the network.
          Locations come from each host&apos;s public IP; the addresses themselves are not shown.
        </p>
      </section>

      <div className="space-y-6">
        {(provers ?? []).map((p) => (
          <ProverPanel key={p.url} prover={p} />
        ))}
      </div>

      <p className="text-xs text-mute">
        A delegated prover makes a wallet&apos;s bundle proof from a job sealed to its key. It is
        sent the wallet&apos;s viewing key, never its spend key: whoever runs it can read that
        wallet&apos;s history, and nothing more. Anyone can run their own (<code>rand-prover</code>)
        and pair a wallet with it instead.
      </p>
    </div>
  );
}
