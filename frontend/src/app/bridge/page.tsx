'use client';

import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { useBridge } from '@/hooks/useApi';
import { formatBridgeChain, formatNumber } from '@/lib/utils';
import type { BridgeAsset } from '@/types';

const assetColumns: Column<BridgeAsset>[] = [
  {
    key: 'index',
    header: 'Index',
    render: (a) => <span className="font-mono">#{formatNumber(a.index)}</span>,
  },
  {
    key: 'chain',
    header: 'Source chain',
    render: (a) => <span>{formatBridgeChain(a.chain)}</span>,
  },
  {
    key: 'token',
    header: 'Token',
    render: (a) => <Hash value={a.token} start={10} end={8} />,
  },
  {
    key: 'asset_id',
    header: 'Asset id',
    render: (a) => <Hash value={a.asset_id} start={10} end={8} />,
  },
];

export default function BridgePage() {
  const { data: bridge, error, isLoading, mutate } = useBridge();

  if (isLoading && !bridge) {
    return <DetailSkeleton />;
  }

  if (error || !bridge) {
    return (
      <>
        <PageHeader title="Bridge" />
        <ErrorState message="Could not load the bridge state." onRetry={() => void mutate()} />
      </>
    );
  }

  if (!bridge.enabled) {
    return (
      <div className="space-y-6">
        <PageHeader title="Bridge" subtitle="Guardian bridge state" />
        <Panel>
          <p className="py-6 text-sm text-mute">
            This chain has no bridge section in its genesis: no guardians, no asset registry, and
            no bridged notes.
          </p>
        </Panel>
      </div>
    );
  }

  const emitters = Object.entries(bridge.emitters);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Bridge"
        subtitle="The bridge's public state. Bridged value is notes, so there are no balances here."
      />

      <Panel title="State">
        <DetailRow label="Outbound emitter">
          <Hash value={bridge.emitter} full />
        </DetailRow>
        <DetailRow label="Guardian set">
          <span className="font-mono">
            {bridge.guardian_set_index === null ? '—' : `#${formatNumber(bridge.guardian_set_index)}`}
          </span>
        </DetailRow>
        <DetailRow label={`Guardians (${bridge.guardians.length})`}>
          {bridge.guardians.length === 0 ? (
            <span className="text-mute">—</span>
          ) : (
            <ul className="space-y-1.5">
              {bridge.guardians.map((g, i) => (
                <li key={g} className="flex items-center gap-2">
                  <span className="w-6 flex-shrink-0 text-right font-mono text-xs text-mute">{i}</span>
                  <Hash value={g} full />
                </li>
              ))}
            </ul>
          )}
        </DetailRow>
        <DetailRow label="Messages emitted">
          <span className="font-mono">
            {bridge.burn_sequence === null ? '—' : formatNumber(bridge.burn_sequence)}
          </span>
        </DetailRow>
        <DetailRow label="Next asset index">
          <span className="font-mono">
            {bridge.next_index === null ? '—' : formatNumber(bridge.next_index)}
          </span>
        </DetailRow>
        <DetailRow label={`Trusted emitters (${emitters.length})`}>
          {emitters.length === 0 ? (
            <span className="text-mute">—</span>
          ) : (
            <ul className="space-y-1.5">
              {emitters.map(([chain, address]) => (
                <li key={chain} className="flex flex-wrap items-center gap-2">
                  <span className="text-soft">{formatBridgeChain(Number(chain))}</span>
                  <Hash value={address} full />
                </li>
              ))}
            </ul>
          )}
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">Asset registry</h2>
        <DataTable
          columns={assetColumns}
          data={bridge.assets}
          keyExtractor={(a) => String(a.index)}
          emptyMessage="No bridged asset registered yet"
        />
        <p className="text-xs text-mute">
          The index is the asset word a bridged note carries; index 0 is SHRUGG and is never in
          the registry.
        </p>
      </section>
    </div>
  );
}
