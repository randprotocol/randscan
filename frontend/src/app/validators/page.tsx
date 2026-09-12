'use client';

import Link from 'next/link';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { ErrorState, PageHeader } from '@/components/States';
import { useValidators } from '@/hooks/useApi';
import { formatNumber, formatPercentage, formatStake, formatTimestamp } from '@/lib/utils';
import type { Validator } from '@/types';

function pendingTotal(v: Validator): bigint {
  return v.pending.reduce((sum, p) => {
    try {
      return sum + BigInt(p.amount || '0');
    } catch {
      return sum;
    }
  }, BigInt(0));
}

const columns: Column<Validator>[] = [
  {
    key: 'address',
    header: 'Validator',
    render: (validator) => (
      <Hash value={validator.address} href={`/validators/${validator.address}`} start={10} end={6} />
    ),
  },
  {
    key: 'active',
    header: 'Epoch set',
    render: (validator) =>
      validator.active ? (
        <span className="badge badge-accent">Active</span>
      ) : (
        <span className="badge badge-neutral">Inactive</span>
      ),
  },
  {
    key: 'stake',
    header: 'Stake',
    render: (validator) => <span className="text-text">{formatStake(validator.stake)}</span>,
  },
  {
    key: 'rewards',
    header: 'Rewards',
    render: (validator) => <span className="text-soft">{formatStake(validator.rewards)}</span>,
  },
  {
    key: 'pending',
    header: 'Unbonding',
    render: (validator) =>
      validator.pending.length === 0 ? (
        <span className="text-mute">—</span>
      ) : (
        <span className="text-soft">
          {formatStake(pendingTotal(validator))}
          <span className="ml-1 text-mute">({validator.pending.length})</span>
        </span>
      ),
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
        <ErrorState message="Could not load the validator register." onRetry={() => void mutate()} />
      </>
    );
  }

  const active = (data ?? []).filter((v) => v.active);
  const activeStake = active.reduce((sum, v) => {
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
            ? `${formatNumber(active.length)} active of ${formatNumber(data.length)} registered · ${formatStake(activeStake, 'active stake')}`
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

      <p className="text-xs text-mute">
        The register is the one place this chain stores amounts in the clear. Active validators are
        the set running the current epoch; the leader of view v is the active entry v mod n in
        address order.
      </p>
    </div>
  );
}
