'use client';

import dynamic from 'next/dynamic';
import { useMemo } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { StatsCard, StatsCardSkeleton, StatsRow } from '@/components/StatsCard';
import { ErrorState, PageHeader } from '@/components/States';
import { useNodes } from '@/hooks/useApi';
import { cn, formatEndpoint, getNodeRoleBadgeClass } from '@/lib/utils';
import { useFmt, useT, type Fmt } from '@/i18n/client';
import type { NodeInfo } from '@/types';

// Leaflet touches `window` at import time, so the map is client-only.
const NodeMap = dynamic(() => import('@/components/NodeMap'), {
  ssr: false,
  loading: () => (
    <div className="card flex h-[480px] items-center justify-center">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-border border-t-accent" />
    </div>
  ),
});

const nodeColumns = (t: (key: string) => string, fmt: Fmt): Column<NodeInfo>[] => [
  {
    key: 'role',
    header: t('nodes.columns.role'),
    render: (node) => (
      <span className="inline-flex items-center gap-2">
        <span className={getNodeRoleBadgeClass(node.role)}>{t(`nodes.roles.${node.role}`)}</span>
        {node.is_self && <span className="badge badge-call">{t('nodes.thisNode')}</span>}
      </span>
    ),
  },
  {
    key: 'peer_id',
    header: t('nodes.columns.peerId'),
    render: (node) => <Hash value={node.peer_id} start={10} end={8} />,
  },
  {
    key: 'endpoint',
    header: t('nodes.columns.address'),
    render: (node) => {
      const endpoint = formatEndpoint(node.ip, node.port);
      return endpoint === '—' ? (
        <span className="text-mute">—</span>
      ) : (
        <span className="font-mono text-soft">{endpoint}</span>
      );
    },
  },
  {
    key: 'location',
    header: t('nodes.columns.location'),
    render: (node) => (
      <span className={cn(node.geo ? 'text-soft' : 'text-mute')}>
        {node.geo ? fmt.location(node.geo) : t('nodes.unknownLocation')}
      </span>
    ),
  },
  {
    key: 'org',
    header: t('nodes.columns.network'),
    render: (node) =>
      node.geo?.org ? (
        <span className="text-soft">{node.geo.org}</span>
      ) : (
        <span className="text-mute">—</span>
      ),
  },
  {
    key: 'connected',
    header: t('nodes.columns.connected'),
    render: (node) => (
      <span className="text-mute">{fmt.connected(node.connected_secs)}</span>
    ),
  },
];

export default function NodesPage() {
  const { data: nodes, error, isLoading, mutate } = useNodes();
  const { t } = useT();
  const fmt = useFmt();
  const columns = nodeColumns(t, fmt);

  const summary = useMemo(() => {
    const list = nodes ?? [];
    const geolocated = list.filter((node) => node.geo !== null);
    const countries = new Set(
      geolocated
        .map((node) => node.geo?.country_code ?? node.geo?.country)
        .filter((value): value is string => typeof value === 'string' && value.trim() !== '')
    );
    return { total: list.length, geolocated: geolocated.length, countries: countries.size };
  }, [nodes]);

  if (error && !nodes) {
    return (
      <>
        <PageHeader title={t('nodes.title')} />
        <ErrorState
          message={t('nodes.error')}
          onRetry={() => void mutate()}
        />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('nodes.title')}
        subtitle={t('nodes.subtitle')}
      />

      <StatsRow columns={3}>
        {isLoading && !nodes ? (
          Array.from({ length: 3 }).map((_, i) => <StatsCardSkeleton key={i} />)
        ) : (
          <>
            <StatsCard title={t('nodes.stats.nodes')} value={fmt.number(summary.total)} />
            <StatsCard
              title={t('nodes.stats.geolocated')}
              value={fmt.number(summary.geolocated)}
              subtitle={
                summary.total > summary.geolocated
                  ? t('nodes.stats.withoutLocation', { count: fmt.number(summary.total - summary.geolocated) })
                  : undefined
              }
            />
            <StatsCard title={t('nodes.stats.countries')} value={fmt.number(summary.countries)} />
          </>
        )}
      </StatsRow>

      <NodeMap nodes={nodes ?? []} />

      <div className="flex flex-wrap items-center gap-4 text-xs text-mute">
        <LegendDot varName="--color-negative" label={t('nodes.thisNode')} />
        <LegendDot varName="--color-accent" label={t('nodes.roles.validator')} />
        <LegendDot varName="--color-accent-2" label={t('nodes.roles.peer')} />
      </div>

      <section className="space-y-4">
        <h2 className="chip">{t('nodes.peers')}</h2>
        <DataTable
          columns={columns}
          data={nodes ?? []}
          keyExtractor={(node) => node.peer_id}
          isLoading={isLoading && !nodes}
          emptyMessage={t('nodes.empty')}
        />
      </section>
    </div>
  );
}

function LegendDot({ varName, label }: { varName: string; label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span
        className="inline-block h-2 w-2 rounded-full"
        style={{ background: `var(${varName})` }}
      />
      {label}
    </span>
  );
}
