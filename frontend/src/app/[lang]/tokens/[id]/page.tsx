'use client';

import { useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useToken } from '@/hooks/useApi';
import { knownBridgeChainName } from '@/lib/utils';
import { L, useFmt, useLocalizePath, useT, type Fmt } from '@/i18n/client';
import type { TokenSupplyEvent } from '@/types';

/** The chain's name when it is a known one, so the formatter can word an unknown id. */
function EventKindBadge({ kind }: { kind: TokenSupplyEvent['kind'] }) {
  const { t } = useT();
  const cls =
    kind === 'token_burn'
      ? 'badge badge-bridge'
      : kind === 'register_token'
        ? 'badge badge-mint'
        : 'badge badge-transfer';
  const label =
    kind === 'register_token'
      ? t('token.initialMint')
      : kind === 'token_mint'
        ? t('token.mint')
        : t('token.burn');
  return <span className={cls}>{label}</span>;
}

function supplyColumns(
  decimals: number,
  symbol: string,
  t: (key: string) => string,
  fmt: Fmt,
): Column<TokenSupplyEvent>[] {
  return [
    {
      key: 'height',
      header: t('token.height'),
      render: (e) => (
        <L href={`/blocks/${e.height}`} className="link font-mono">
          #{fmt.number(e.height)}
        </L>
      ),
    },
    {
      key: 'kind',
      header: t('token.event'),
      render: (e) => <EventKindBadge kind={e.kind} />,
    },
    {
      key: 'tx',
      header: t('token.transaction'),
      render: (e) => <Hash value={e.tx_hash} href={`/transactions/${e.tx_hash}`} />,
    },
    {
      key: 'delta',
      header: t('token.change'),
      render: (e) => {
        const negative = e.delta.startsWith('-');
        const magnitude = negative ? e.delta.slice(1) : e.delta;
        return (
          <span className={`font-mono ${negative ? 'text-negative' : 'text-accent'}`}>
            {negative ? '-' : '+'}
            {fmt.units(magnitude, decimals)} {symbol}
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
  const router = useRouter();
  const localize = useLocalizePath();
  const { t, tp } = useT();
  const fmt = useFmt();

  // A token's address is its `rpl1…` id: a page reached by registry index or hex id moves there.
  useEffect(() => {
    if (token && token.id_text && id !== token.id_text) {
      router.replace(localize(`/tokens/${token.id_text}`));
    }
  }, [token, id, router, localize]);

  if (isLoading && !token) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title={t('token.notFound')}
        message={t('token.notFoundMessage', { id })}
        backHref="/tokens"
        backLabel={t('token.back')}
      />
    );
  }

  if (error || !token) {
    return <ErrorState message={t('token.loadError')} onRetry={() => void mutate()} />;
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
          title={t('token.totalSupply')}
          value={`${fmt.units(token.total_supply, token.decimals)} ${token.symbol}`}
        />
        <StatsCard title={t('token.decimals')} value={fmt.number(token.decimals)} />
        <StatsCard title={t('token.registryIndex')} value={`#${fmt.number(token.index)}`} />
        <StatsCard
          title={t('token.authority')}
          value={t(`token.authorityKind.${token.authority.kind}`)}
          subtitle={
            token.authority.kind === 'bridge'
              ? tp('token.backingCount', token.authority.backings.length)
              : undefined
          }
        />
      </StatsRow>

      <Panel title={t('token.registration')}>
        <DetailRow label={t('token.deployTx')}>
          {token.deploy_tx ? (
            <Hash value={token.deploy_tx} href={`/transactions/${token.deploy_tx}`} full />
          ) : (
            <span className="text-mute">{t('token.notIndexed')}</span>
          )}
        </DetailRow>
        <DetailRow label={t('token.registeredAt')}>
          <L href={`/blocks/${token.registered_at}`} className="link font-mono">
            #{fmt.number(token.registered_at)}
          </L>
        </DetailRow>
        <DetailRow label={t('token.idHex')}>
          <Hash value={token.id} full />
        </DetailRow>
        <DetailRow label={t('token.idText')}>
          <Hash value={token.id_text} full copyable />
        </DetailRow>
        <DetailRow label={t('token.mintNonce')}>
          <span className="font-mono">{fmt.number(token.mint_nonce)}</span>
        </DetailRow>
      </Panel>

      {token.authority.kind === 'key' && (
        <Panel title={t('token.mintAuthority')}>
          <DetailRow label={t('token.signingKey')}>
            <Hash value={token.authority.key} start={10} end={8} />
          </DetailRow>
          <DetailRow label={t('token.address')}>
            <Hash value={token.authority.address} href={`/validators/${token.authority.address}`} full />
          </DetailRow>
        </Panel>
      )}

      {token.authority.kind === 'program' && (
        <Panel title={t('token.mintAuthority')}>
          <DetailRow label={t('token.program')}>
            <Hash value={token.authority.program} href={`/programs/${token.authority.program}`} full />
          </DetailRow>
        </Panel>
      )}

      {token.authority.kind === 'bridge' && (
        <Panel title={t('token.backings', { count: fmt.number(token.authority.backings.length) })}>
          {token.authority.backings.map((b, i) => (
            <div key={i} className="border-t border-border-soft py-3 first:border-t-0">
              <DetailRow label={t('token.sourceChain')}>
                {fmt.bridgeChain(b.chain, knownBridgeChainName(b.chain))}
              </DetailRow>
              <DetailRow label={t('token.sourceToken')}>
                <span className="font-mono break-all">{b.token}</span>
              </DetailRow>
              <DetailRow label={t('token.sourceDecimals')}>
                <span className="font-mono">{fmt.number(b.decimals)}</span>
              </DetailRow>
              <DetailRow label={t('token.locked')}>
                <span className="font-mono text-strong">{fmt.units(b.locked, token.decimals)} {token.symbol}</span>
              </DetailRow>
              <DetailRow
                label={
                  typeof b.mint_window_secs === 'number'
                    ? t('token.mintedIn', { window: fmt.mintWindow(b.mint_window_secs) })
                    : t('token.mintedToday')
                }
              >
                <span className="font-mono">
                  {b.minted_today === null ? '—' : fmt.units(b.minted_today, token.decimals)}
                  {b.mint_cap_per_day !== null && (
                    <span className="text-mute"> / {fmt.units(b.mint_cap_per_day, token.decimals)}</span>
                  )}{' '}
                  {token.symbol}
                </span>
              </DetailRow>
              {b.mint_headroom != null && (
                <DetailRow label={t('token.roomToMint')}>
                  <span
                    className="font-mono"
                    title={t('token.roomToMintHelp')}
                  >
                    {fmt.units(b.mint_headroom, token.decimals)} {token.symbol}
                  </span>
                </DetailRow>
              )}
            </div>
          ))}
        </Panel>
      )}

      <section className="space-y-4">
        <h2 className="chip">{t('token.supplyHistory')}</h2>
        <DataTable
          columns={supplyColumns(token.decimals, token.symbol, t, fmt)}
          data={token.supply_history}
          keyExtractor={(e) => e.tx_hash}
          emptyMessage={t('token.empty')}
        />
        <p className="text-xs text-mute">
          {t('token.historyHelp')}
        </p>
      </section>
    </div>
  );
}
