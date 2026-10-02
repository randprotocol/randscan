'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { Pagination } from '@/components/Pagination';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { BlocksTable, TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useTransactions, useValidator } from '@/hooks/useApi';
import {
  formatDateTime,
  formatNumber,
  formatPercentage,
  formatStake,
  formatTimestamp,
} from '@/lib/utils';
import type { PendingStake } from '@/types';

const PAGE_SIZE = 10;

const pendingColumns: Column<PendingStake>[] = [
  {
    key: 'release_epoch',
    header: 'Release epoch',
    render: (p) => <span className="font-mono">{formatNumber(p.release_epoch)}</span>,
  },
  {
    key: 'amount',
    header: 'Amount',
    render: (p) => <span className="text-text">{formatStake(p.amount)}</span>,
  },
];

export default function ValidatorDetailPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const [page, setPage] = useState(1);
  const { data: validator, error, isLoading, mutate } = useValidator(address);
  const { data: txs, isLoading: txsLoading } = useTransactions(page, PAGE_SIZE, null, {
    validator: error ? null : address,
  });

  if (isLoading && !validator) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Validator not found"
        message={`"${address}" is not in the validator register.`}
        backHref="/validators"
        backLabel="Back to validators"
      />
    );
  }

  if (error || !validator) {
    return <ErrorState message="Could not load this validator." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Validator"
        subtitle={<Hash value={validator.address} full copyable />}
        actions={
          validator.active ? (
            <span className="badge badge-accent">Active this epoch</span>
          ) : (
            <span className="badge badge-neutral">Not in the current set</span>
          )
        }
      />

      <StatsRow columns={4}>
        <StatsCard title="Stake" value={formatStake(validator.stake)} />
        <StatsCard title="Rewards" value={formatStake(validator.rewards)} subtitle="unwithdrawn fees" />
        <StatsCard title="Share of active stake" value={formatPercentage(validator.share_percent)} />
        <StatsCard
          title="Blocks proposed"
          value={formatNumber(validator.blocks_proposed)}
          subtitle={
            validator.last_proposed_timestamp_ms !== null
              ? `last ${formatTimestamp(validator.last_proposed_timestamp_ms)}`
              : undefined
          }
        />
      </StatsRow>

      <Panel title="Register entry">
        <DetailRow label="Address">
          <Hash value={validator.address} full />
        </DetailRow>
        <DetailRow label="Stake">{formatStake(validator.stake)}</DetailRow>
        <DetailRow label="Rewards">{formatStake(validator.rewards)}</DetailRow>
        <DetailRow label="Payout address">
          {validator.payout ? (
            <Hash value={validator.payout} full />
          ) : (
            <span className="text-mute">Not served by this node</span>
          )}
        </DetailRow>
        <DetailRow label="Register nonce">
          <span className="font-mono">{formatNumber(validator.nonce)}</span>
        </DetailRow>
        <DetailRow label="Share of active stake">
          {formatPercentage(validator.share_percent)}
        </DetailRow>
        <DetailRow label="Sort index">
          <span className="font-mono">{formatNumber(validator.sort_index)}</span>
        </DetailRow>
        <DetailRow label="Last proposed block">
          {validator.last_proposed_height === null ? (
            <span className="text-mute">Never</span>
          ) : (
            <span className="inline-flex flex-wrap items-center gap-2">
              <Link href={`/blocks/${validator.last_proposed_height}`} className="link font-mono">
                #{formatNumber(validator.last_proposed_height)}
              </Link>
              {validator.last_proposed_timestamp_ms !== null && (
                <span className="text-mute">
                  {formatDateTime(validator.last_proposed_timestamp_ms)}
                </span>
              )}
            </span>
          )}
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">Unbonding queue</h2>
        <DataTable
          columns={pendingColumns}
          data={validator.pending}
          keyExtractor={(p, ) => `${p.release_epoch}-${p.amount}`}
          emptyMessage="Nothing is unbonding"
        />
      </section>

      <section className="space-y-4">
        <h2 className="chip">Staking transactions</h2>
        <TransactionsTable
          transactions={txs?.data ?? []}
          isLoading={txsLoading && !txs}
          emptyMessage="No bond, unbond, withdraw or mint names this validator"
        />
        {txs && txs.pagination.total_pages > 1 && (
          <Pagination
            currentPage={txs.pagination.page}
            totalPages={txs.pagination.total_pages}
            onPageChange={setPage}
          />
        )}
      </section>

      <section className="space-y-4">
        <h2 className="chip">Recent blocks</h2>
        <BlocksTable
          blocks={validator.recent_blocks}
          emptyMessage="This validator has not proposed any blocks yet"
        />
      </section>
    </div>
  );
}
