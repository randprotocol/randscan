'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { useState } from 'react';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { Pagination } from '@/components/Pagination';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { AccountTransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useAccount, useAccountTransactions } from '@/hooks/useApi';
import { formatAmount, formatNumber, formatStake } from '@/lib/utils';

const PAGE_SIZE = 25;

export default function AccountDetailPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const [page, setPage] = useState(1);

  const { data: account, error, isLoading, mutate } = useAccount(address);
  const { data: txs, isLoading: txsLoading } = useAccountTransactions(
    error ? null : address,
    page,
    PAGE_SIZE
  );

  if (isLoading && !account) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Account not found"
        message={`No activity has been indexed for "${address}". Accounts appear once they send or receive a transaction.`}
        backHref="/"
        backLabel="Back to dashboard"
      />
    );
  }

  if (error || !account) {
    return <ErrorState message="Could not load this account." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Account"
        subtitle={<Hash value={account.address} full copyable />}
        actions={
          account.is_validator ? (
            <Link href={`/validators/${account.address}`} className="badge badge-accent">
              Validator
            </Link>
          ) : undefined
        }
      />

      <StatsRow columns={4}>
        <StatsCard title="Balance" value={formatAmount(account.balance)} />
        <StatsCard title="Transactions" value={formatNumber(account.tx_count)} />
        <StatsCard title="Nonce" value={formatNumber(account.nonce)} />
        <StatsCard
          title={account.is_validator ? 'Stake' : 'Programs deployed'}
          value={
            account.is_validator && account.stake !== null
              ? formatStake(account.stake)
              : formatNumber(account.programs_deployed)
          }
          subtitle={
            account.is_validator && account.stake !== null
              ? `${formatNumber(account.programs_deployed)} programs deployed`
              : undefined
          }
        />
      </StatsRow>

      <Panel title="Details">
        <DetailRow label="Address">
          <Hash value={account.address} full />
        </DetailRow>
        <DetailRow label="Balance">{formatAmount(account.balance)}</DetailRow>
        <DetailRow label="Nonce">
          <span className="font-mono">{formatNumber(account.nonce)}</span>
        </DetailRow>
        <DetailRow label="Transaction count">
          <span className="font-mono">{formatNumber(account.tx_count)}</span>
        </DetailRow>
        <DetailRow label="First seen">
          <Link href={`/blocks/${account.first_seen_height}`} className="link font-mono">
            #{formatNumber(account.first_seen_height)}
          </Link>
        </DetailRow>
        <DetailRow label="Last seen">
          <Link href={`/blocks/${account.last_seen_height}`} className="link font-mono">
            #{formatNumber(account.last_seen_height)}
          </Link>
        </DetailRow>
        <DetailRow label="Validator">
          {account.is_validator ? (
            <span className="inline-flex flex-wrap items-center gap-2">
              <span className="badge badge-accent">Active validator</span>
              {account.stake !== null && (
                <span className="text-soft">staking {formatStake(account.stake)}</span>
              )}
              <Link href={`/validators/${account.address}`} className="link">
                View validator →
              </Link>
            </span>
          ) : (
            <span className="text-mute">No</span>
          )}
        </DetailRow>
        <DetailRow label="Programs deployed">
          <span className="font-mono">{formatNumber(account.programs_deployed)}</span>
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">Transactions</h2>
        <AccountTransactionsTable
          transactions={txs?.data ?? []}
          isLoading={txsLoading && !txs}
        />
        {txs && (
          <Pagination
            currentPage={txs.pagination.page}
            totalPages={txs.pagination.total_pages}
            onPageChange={setPage}
          />
        )}
      </section>
    </div>
  );
}
