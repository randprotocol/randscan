'use client';

import Link from 'next/link';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { ErrorState, PageHeader } from '@/components/States';
import { useTokens } from '@/hooks/useApi';
import { formatNumber, formatUnits } from '@/lib/utils';
import type { TokenInfo } from '@/types';

function authorityLabel(info: TokenInfo): string {
  switch (info.authority.kind) {
    case 'none':
      return 'None (fixed supply)';
    case 'key':
      return 'Key-signed';
    case 'bridge':
      return `Bridge (${info.authority.backings.length} backing${info.authority.backings.length === 1 ? '' : 's'})`;
    case 'program':
      return 'Program';
  }
}

const columns: Column<TokenInfo>[] = [
  {
    key: 'symbol',
    header: 'Token',
    render: (t) => (
      <span className="flex flex-col">
        <Link href={`/tokens/${t.id_text}`} className="link font-medium">
          {t.symbol}
        </Link>
        <span className="text-xs text-mute">{t.name}</span>
      </span>
    ),
  },
  {
    key: 'index',
    header: 'Index',
    render: (t) => <span className="font-mono">#{formatNumber(t.index)}</span>,
  },
  {
    key: 'decimals',
    header: 'Decimals',
    render: (t) => <span className="font-mono">{formatNumber(t.decimals)}</span>,
  },
  {
    key: 'authority',
    header: 'Authority',
    render: (t) => <span>{authorityLabel(t)}</span>,
  },
  {
    key: 'total_supply',
    header: 'Total supply',
    render: (t) => (
      <span className="font-mono text-strong">
        {formatUnits(t.total_supply, t.decimals)} {t.symbol}
      </span>
    ),
  },
  {
    key: 'registered_at',
    header: 'Registered at',
    render: (t) => (
      <Link href={`/blocks/${t.registered_at}`} className="link font-mono">
        #{formatNumber(t.registered_at)}
      </Link>
    ),
  },
  {
    key: 'id_text',
    header: 'Token id',
    render: (t) => <Hash value={t.id_text} start={10} end={6} />,
  },
];

export default function TokensPage() {
  const { data, error, isLoading, mutate } = useTokens();

  if (error && !data) {
    return (
      <>
        <PageHeader title="Tokens" />
        <ErrorState message="Could not load the token registry." onRetry={() => void mutate()} />
      </>
    );
  }

  if (data && !data.enabled) {
    return (
      <div className="space-y-6">
        <PageHeader title="Tokens" subtitle="The RPL token standard" />
        <p className="rounded border border-border-soft bg-bg-soft px-4 py-6 text-sm text-mute">
          This chain has no token registry: no RPL token has been registered, and this build&apos;s
          node predates the token RPC. Every transfer is RAND.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Tokens"
        subtitle={
          data
            ? `${formatNumber(data.tokens.length)} RPL token${data.tokens.length === 1 ? '' : 's'} registered${
                data.registration_fee !== null
                  ? ` · registration fee ${formatUnits(data.registration_fee)} RAND`
                  : ''
              }`
            : 'Loading the token registry…'
        }
      />

      <DataTable
        columns={columns}
        data={data?.tokens ?? []}
        keyExtractor={(t) => String(t.index)}
        isLoading={isLoading && !data}
        emptyMessage="No RPL token has been registered yet"
      />

      <p className="text-xs text-mute">
        A token&apos;s registration and its mints are public by design (spec §4) — the amount and
        symbol here are exact. What is private is who holds a token&apos;s notes and who a
        transfer moved them between: a transfer of any token is indistinguishable from a plain
        RAND payment on the public pages, and a token&apos;s registry index never appears on a
        transfer&apos;s bundle.
      </p>
    </div>
  );
}
