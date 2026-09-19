'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useToken } from '@/hooks/useApi';
import { formatBridgeChain, formatNumber, formatUnits } from '@/lib/utils';
import type { TokenSupplyEvent } from '@/types';

function EventKindBadge({ kind }: { kind: TokenSupplyEvent['kind'] }) {
  const cls =
    kind === 'token_burn'
      ? 'badge badge-bridge'
      : kind === 'register_token'
        ? 'badge badge-mint'
        : 'badge badge-transfer';
  const label = kind === 'register_token' ? 'Initial mint' : kind === 'token_mint' ? 'Mint' : 'Burn';
  return <span className={cls}>{label}</span>;
}

function supplyColumns(decimals: number, symbol: string): Column<TokenSupplyEvent>[] {
  return [
    {
      key: 'height',
      header: 'Height',
      render: (e) => (
        <Link href={`/blocks/${e.height}`} className="link font-mono">
          #{formatNumber(e.height)}
        </Link>
      ),
    },
    {
      key: 'kind',
      header: 'Event',
      render: (e) => <EventKindBadge kind={e.kind} />,
    },
    {
      key: 'tx',
      header: 'Transaction',
      render: (e) => <Hash value={e.tx_hash} href={`/transactions/${e.tx_hash}`} />,
    },
    {
      key: 'delta',
      header: 'Change',
      render: (e) => {
        const negative = e.delta.startsWith('-');
        const magnitude = negative ? e.delta.slice(1) : e.delta;
        return (
          <span className={`font-mono ${negative ? 'text-accent-3' : 'text-accent'}`}>
            {negative ? '-' : '+'}
            {formatUnits(magnitude, decimals)} {symbol}
          </span>
        );
      },
    },
  ];
}

export default function TokenDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: token, error, isLoading, mutate } = useToken(id);

  if (isLoading && !token) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Token not found"
        message={`No RPL token matches "${id}". Use its registry index, 64-hex id or rpl1… text form.`}
        backHref="/tokens"
        backLabel="Back to tokens"
      />
    );
  }

  if (error || !token) {
    return <ErrorState message="Could not load this token." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={token.symbol}
        subtitle={
          <span className="flex flex-col gap-1">
            <span>{token.name}</span>
            <Hash value={token.id_text} full copyable />
          </span>
        }
      />

      <StatsRow columns={4}>
        <StatsCard
          title="Total supply"
          value={`${formatUnits(token.total_supply, token.decimals)} ${token.symbol}`}
        />
        <StatsCard title="Decimals" value={formatNumber(token.decimals)} />
        <StatsCard title="Registry index" value={`#${formatNumber(token.index)}`} />
        <StatsCard
          title="Authority"
          value={token.authority.kind === 'none' ? 'None' : token.authority.kind}
          subtitle={
            token.authority.kind === 'bridge'
              ? `${token.authority.backings.length} backing${token.authority.backings.length === 1 ? '' : 's'}`
              : undefined
          }
        />
      </StatsRow>

      <Panel title="Registration">
        <DetailRow label="Deploy transaction">
          {token.deploy_tx ? (
            <Hash value={token.deploy_tx} href={`/transactions/${token.deploy_tx}`} full />
          ) : (
            <span className="text-mute">Not indexed (registered before this pass)</span>
          )}
        </DetailRow>
        <DetailRow label="Registered at">
          <Link href={`/blocks/${token.registered_at}`} className="link font-mono">
            #{formatNumber(token.registered_at)}
          </Link>
        </DetailRow>
        <DetailRow label="Token id (hex)">
          <Hash value={token.id} full />
        </DetailRow>
        <DetailRow label="Token id (rpl1…)">
          <Hash value={token.id_text} full copyable />
        </DetailRow>
        <DetailRow label="Mint nonce">
          <span className="font-mono">{formatNumber(token.mint_nonce)}</span>
        </DetailRow>
      </Panel>

      {token.authority.kind === 'key' && (
        <Panel title="Mint authority">
          <DetailRow label="Signing key">
            <Hash value={token.authority.key} start={10} end={8} />
          </DetailRow>
          <DetailRow label="Address">
            <Hash value={token.authority.address} href={`/validators/${token.authority.address}`} full />
          </DetailRow>
        </Panel>
      )}

      {token.authority.kind === 'program' && (
        <Panel title="Mint authority">
          <DetailRow label="Program">
            <Hash value={token.authority.program} href={`/programs/${token.authority.program}`} full />
          </DetailRow>
        </Panel>
      )}

      {token.authority.kind === 'bridge' && (
        <Panel title={`Backings (${token.authority.backings.length})`}>
          {token.authority.backings.map((b, i) => (
            <div key={i} className="border-t border-border-soft py-3 first:border-t-0">
              <DetailRow label="Source chain">{formatBridgeChain(b.chain)}</DetailRow>
              <DetailRow label="Source token">
                <span className="font-mono break-all">{b.token}</span>
              </DetailRow>
              <DetailRow label="Source decimals">
                <span className="font-mono">{formatNumber(b.decimals)}</span>
              </DetailRow>
              <DetailRow label="Locked">
                <span className="font-mono text-strong">{formatUnits(b.locked, token.decimals)} {token.symbol}</span>
              </DetailRow>
              <DetailRow label="Minted today / cap">
                <span className="font-mono">
                  {b.minted_today === null ? '—' : formatUnits(b.minted_today, token.decimals)}
                  {b.mint_cap_per_day !== null && (
                    <span className="text-mute"> / {formatUnits(b.mint_cap_per_day, token.decimals)}</span>
                  )}{' '}
                  {token.symbol}
                </span>
              </DetailRow>
            </div>
          ))}
        </Panel>
      )}

      <section className="space-y-4">
        <h2 className="chip">Supply history</h2>
        <DataTable
          columns={supplyColumns(token.decimals, token.symbol)}
          data={token.supply_history}
          keyExtractor={(e) => e.tx_hash}
          emptyMessage="No public mint or burn event indexed for this token yet"
        />
        <p className="text-xs text-mute">
          Every register_token (initial mint), token_mint and token_burn naming this token, oldest
          first — public by design, since it is what makes the total supply auditable. A transfer
          of this token&apos;s notes is a plain shielded transfer and never appears here.
        </p>
      </section>
    </div>
  );
}
