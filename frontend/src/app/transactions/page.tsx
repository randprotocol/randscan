'use client';

import { useState } from 'react';
import { Pagination } from '@/components/Pagination';
import { TransactionsTable } from '@/components/Tables';
import { ErrorState, PageHeader } from '@/components/States';
import { useTransactions } from '@/hooks/useApi';
import { formatNumber, getKindLabel, TRANSACTION_KINDS } from '@/lib/utils';
import type { TransactionKind } from '@/types';

const PAGE_SIZE = 25;

type KindFilter = TransactionKind | 'all';

export default function TransactionsPage() {
  const [page, setPage] = useState(1);
  const [kind, setKind] = useState<KindFilter>('all');

  const { data, error, isLoading, mutate } = useTransactions(
    page,
    PAGE_SIZE,
    kind === 'all' ? null : kind
  );

  const onKindChange = (value: KindFilter) => {
    setKind(value);
    setPage(1);
  };

  const filterControl = (
    <label className="flex items-center gap-2 text-sm text-mute">
      <span>Kind</span>
      <select
        value={kind}
        onChange={(event) => onKindChange(event.target.value as KindFilter)}
        className="rounded border border-border bg-surface px-3 py-1.5 text-sm text-strong focus:border-accent focus:outline-none"
      >
        <option value="all">All kinds</option>
        {TRANSACTION_KINDS.map((value) => (
          <option key={value} value={value}>
            {getKindLabel(value)}
          </option>
        ))}
      </select>
    </label>
  );

  if (error && !data) {
    return (
      <>
        <PageHeader title="Transactions" actions={filterControl} />
        <ErrorState message="Could not load transactions." onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Transactions"
        subtitle={
          data
            ? `${formatNumber(data.pagination.total)} transactions${
                kind === 'all' ? '' : ` of kind ${getKindLabel(kind)}`
              }`
            : 'Loading transactions…'
        }
        actions={filterControl}
      />

      <TransactionsTable
        transactions={data?.data ?? []}
        isLoading={isLoading && !data}
        emptyMessage={
          kind === 'all' ? 'No transactions found' : `No ${getKindLabel(kind)} transactions found`
        }
      />

      {data && (
        <Pagination
          currentPage={data.pagination.page}
          totalPages={data.pagination.total_pages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
