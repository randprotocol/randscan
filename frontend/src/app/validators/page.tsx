'use client';

import Link from 'next/link';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { ErrorState, PageHeader } from '@/components/States';
import { useValidators } from '@/hooks/useApi';
import { formatNumber, formatPercentage, formatStake, formatTimestamp } from '@/lib/utils';
import type { Validator } from '@/types';

const columns: Column<Validator>[] = [
  {
    key: 'address',
    header: 'Validator',
    render: (validator) => (
      <Hash value={validator.address} href={`/validators/${validator.address}`} start={10} end={6} />
    ),
  },
  {
    key: 'stake',
    header: 'Stake',
    render: (validator) => <span className="text-text">{formatStake(validator.stake)}</span>,
  },
  {
    key: 'share_percent',
    header: 'Share',
    render: (validator) => (
      <div className="flex items-center gap-2">
        <div className="h-1.5 w-20 overflow-hidden rounded-full bg-surface-2">
          <div
            className="h-full rounded-full bg-accent"
            style={{ width: `${Math.min(100, Math.max(0, validator.share_percent))}%` }}
          />
        </div>
        <span className="text-soft">{formatPercentage(validator.share_percent)}</span>
      </div>
    ),
  },
  {
    key: 'blocks_proposed',
    header: 'Blocks proposed',
    render: (validator) => (
      <span className="text-soft">{formatNumber(validator.blocks_proposed)}</span>
    ),
  },
  {
    key: 'last_proposed',
    header: 'Last proposed',
    render: (validator) =>
      validator.last_proposed_height === null ? (
        <span className="text-mute">Never</span>
      ) : (
        <span className="text-soft">
          <Link href={`/blocks/${validator.last_proposed_height}`} className="link font-mono">
            #{formatNumber(validator.last_proposed_height)}
          </Link>
          {validator.last_proposed_timestamp_ms !== null && (
            <span className="ml-2 text-mute">
              {formatTimestamp(validator.last_proposed_timestamp_ms)}
            </span>
          )}
        </span>
      ),
  },
];

export default function ValidatorsPage() {
  const { data, error, isLoading, mutate } = useValidators();

  if (error && !data) {
    return (
      <>
        <PageHeader title="Validators" />
        <ErrorState message="Could not load the validator set." onRetry={() => void mutate()} />
      </>
    );
  }

  const totalStake = (data ?? []).reduce((sum, v) => {
    try {
      return sum + BigInt(v.stake || '0');
    } catch {
      return sum;
    }
  }, BigInt(0));

  return (
    <div className="space-y-6">
      <PageHeader
        title="Validators"
        subtitle={
          data
            ? `${formatNumber(data.length)} validators · ${formatStake(totalStake, 'total stake')}`
            : 'Loading validators…'
        }
      />

      <DataTable
        columns={columns}
        data={data ?? []}
        keyExtractor={(validator) => validator.address}
        isLoading={isLoading && !data}
        emptyMessage="No validators reported"
      />
    </div>
  );
}
