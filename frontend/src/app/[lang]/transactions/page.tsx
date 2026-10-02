'use client';

import { useState } from 'react';
import { Pagination } from '@/components/Pagination';
import { TransactionsTable } from '@/components/Tables';
import { ErrorState, PageHeader } from '@/components/States';
import { useTransactions } from '@/hooks/useApi';
import { TRANSACTION_KINDS } from '@/lib/utils';
import { useT } from '@/i18n/client';
import type { TransactionKind } from '@/types';

const PAGE_SIZE = 25;

type KindFilter = TransactionKind | 'all';

export default function TransactionsPage() {
  const [page, setPage] = useState(1);
  const [kind, setKind] = useState<KindFilter>('all');
  const { t, tp } = useT();
  const kindLabel = (k: string) => t(`transactions.kinds.${k}`);

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
      <span>{t('transactions.kind')}</span>
      <select
        value={kind}
        onChange={(event) => onKindChange(event.target.value as KindFilter)}
        className="rounded border border-border bg-surface px-3 py-1.5 text-sm text-strong focus:border-accent focus:outline-none"
      >
        <option value="all">{t('transactions.allKinds')}</option>
        {TRANSACTION_KINDS.map((value) => (
          <option key={value} value={value}>
            {kindLabel(value)}
          </option>
        ))}
      </select>
    </label>
  );

  if (error && !data) {
    return (
      <>
        <PageHeader title={t('transactions.title')} actions={filterControl} />
        <ErrorState message={t('transactions.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('transactions.title')}
        subtitle={
          data
            ? kind === 'all'
              ? tp('transactions.count', data.pagination.total)
              : tp('transactions.countOfKind', data.pagination.total, { kind: kindLabel(kind) })
            : t('transactions.loading')
        }
        actions={filterControl}
      />

      <TransactionsTable
        transactions={data?.data ?? []}
        isLoading={isLoading && !data}
        emptyMessage={
          kind === 'all' ? t('transactions.empty') : t('transactions.emptyOfKind', { kind: kindLabel(kind) })
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
