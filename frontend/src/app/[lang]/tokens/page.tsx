'use client';

import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { ErrorState, PageHeader } from '@/components/States';
import { useTokens } from '@/hooks/useApi';
import { L, useFmt, useT, type Fmt } from '@/i18n/client';
import type { TokenInfo } from '@/types';

type T = ReturnType<typeof useT>;

function authorityLabel(info: TokenInfo, { t, tp }: T): string {
  switch (info.authority.kind) {
    case 'none':
      return t('tokens.authority.none');
    case 'key':
      return t('tokens.authority.key');
    case 'bridge':
      return tp('tokens.authority.bridge', info.authority.backings.length);
    case 'program':
      return t('tokens.authority.program');
  }
}

const tokenColumns = (tr: T, fmt: Fmt): Column<TokenInfo>[] => [
  {
    key: 'symbol',
    header: tr.t('tokens.columns.token'),
    render: (t) => (
      <span className="flex flex-col">
        <L href={`/tokens/${t.id_text}`} className="link font-medium">
          {t.symbol}
        </L>
        <span className="text-xs text-mute">{t.name}</span>
      </span>
    ),
  },
  {
    key: 'index',
    header: tr.t('tokens.columns.index'),
    render: (t) => <span className="font-mono">#{fmt.number(t.index)}</span>,
  },
  {
    key: 'decimals',
    header: tr.t('tokens.columns.decimals'),
    render: (t) => <span className="font-mono">{fmt.number(t.decimals)}</span>,
  },
  {
    key: 'authority',
    header: tr.t('tokens.columns.authority'),
    render: (t) => <span>{authorityLabel(t, tr)}</span>,
  },
  {
    key: 'total_supply',
    header: tr.t('tokens.columns.totalSupply'),
    render: (t) => (
      <span className="font-mono text-strong">
        {fmt.units(t.total_supply, t.decimals)} {t.symbol}
      </span>
    ),
  },
  {
    key: 'registered_at',
    header: tr.t('tokens.columns.registeredAt'),
    render: (t) => (
      <L href={`/blocks/${t.registered_at}`} className="link font-mono">
        #{fmt.number(t.registered_at)}
      </L>
    ),
  },
  {
    key: 'id_text',
    header: tr.t('tokens.columns.tokenId'),
    render: (t) => <Hash value={t.id_text} start={10} end={6} />,
  },
];

export default function TokensPage() {
  const { data, error, isLoading, mutate } = useTokens();
  const tr = useT();
  const { t, tp } = tr;
  const fmt = useFmt();
  const columns = tokenColumns(tr, fmt);

  if (error && !data) {
    return (
      <>
        <PageHeader title={t('tokens.title')} />
        <ErrorState message={t('tokens.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  if (data && !data.enabled) {
    return (
      <div className="space-y-6">
        <PageHeader title={t('tokens.title')} subtitle={t('tokens.standard')} />
        <p className="rounded border border-border-soft bg-bg-soft px-4 py-6 text-sm text-mute">
          {t('tokens.disabled')}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('tokens.title')}
        subtitle={
          data
            ? data.registration_fee !== null
              ? tp('tokens.registeredWithFee', data.tokens.length, {
                  fee: fmt.units(data.registration_fee),
                })
              : tp('tokens.registered', data.tokens.length)
            : t('tokens.loading')
        }
      />

      <DataTable
        columns={columns}
        data={data?.tokens ?? []}
        keyExtractor={(t) => String(t.index)}
        isLoading={isLoading && !data}
        emptyMessage={t('tokens.empty')}
      />

      <p className="text-xs text-mute">
        {t('tokens.help')}
      </p>
    </div>
  );
}
