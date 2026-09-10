'use client';

import dynamic from 'next/dynamic';
import { useMemo } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { StatsCard, StatsCardSkeleton, StatsRow } from '@/components/StatsCard';
import { ErrorState, PageHeader } from '@/components/States';
import { useNodes } from '@/hooks/useApi';
import {
  cn,
  formatConnectedTime,
  formatEndpoint,
  formatLocation,
  formatNumber,
  getNodeRoleBadgeClass,
  getNodeRoleLabel,
} from '@/lib/utils';
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

const columns: Column<NodeInfo>[] = [
  {
    key: 'role',
    header: 'Role',
    render: (node) => (
      <span className="inline-flex items-center gap-2">
        <span className={getNodeRoleBadgeClass(node.role)}>{getNodeRoleLabel(node.role)}</span>
        {node.is_self && <span className="badge badge-call">This node</span>}
      </span>
    ),
  },
  {
    key: 'peer_id',
    header: 'Peer ID',
    render: (node) => <Hash value={node.peer_id} start={10} end={8} />,
  },
  {
    key: 'endpoint',
    header: 'Address',
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
    header: 'Location',
    render: (node) => (
      <span className={cn(node.geo ? 'text-soft' : 'text-mute')}>
        {node.geo ? formatLocation(node.geo) : 'Unknown location'}
      </span>
    ),
  },
  {
    key: 'org',
    header: 'Network',
    render: (node) =>
      node.geo?.org ? (
        <span className="text-soft">{node.geo.org}</span>
      ) : (
        <span className="text-mute">—</span>
      ),
  },
  {
    key: 'connected',
    header: 'Connected',
    render: (node) => (
      <span className="text-mute">{formatConnectedTime(node.connected_secs)}</span>
    ),
  },
];

export default function NodesPage() {
  const { data: nodes, error, isLoading, mutate } = useNodes();

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
        <PageHeader title="Nodes" />
        <ErrorState
          message="Could not load the peer list from the node."
          onRetry={() => void mutate()}
        />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Nodes"
        subtitle="Full nodes running Rand Protocol, located by peer address"
      />

      <StatsRow columns={3}>
        {isLoading && !nodes ? (
          Array.from({ length: 3 }).map((_, i) => <StatsCardSkeleton key={i} />)
        ) : (
          <>
            <StatsCard title="Nodes" value={formatNumber(summary.total)} />
            <StatsCard
              title="Geolocated"
              value={formatNumber(summary.geolocated)}
              subtitle={
                summary.total > summary.geolocated
                  ? `${formatNumber(summary.total - summary.geolocated)} without location`
                  : undefined
              }
            />
            <StatsCard title="Countries" value={formatNumber(summary.countries)} />
          </>
        )}
      </StatsRow>

      <NodeMap nodes={nodes ?? []} />

      <div className="flex flex-wrap items-center gap-4 text-xs text-mute">
        <LegendDot varName="--color-accent-3" label="This node" />
        <LegendDot varName="--color-accent" label="Validator" />
        <LegendDot varName="--color-accent-2" label="Peer" />
      </div>

      <section className="space-y-4">
        <h2 className="chip">Peers</h2>
        <DataTable
          columns={columns}
          data={nodes ?? []}
          keyExtractor={(node) => node.peer_id}
          isLoading={isLoading && !nodes}
          emptyMessage="No peers are currently connected"
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
