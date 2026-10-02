'use client';

import { useState } from 'react';
import { Pagination } from '@/components/Pagination';
import { BlocksTable } from '@/components/Tables';
import { ErrorState, PageHeader } from '@/components/States';
import { useBlocks } from '@/hooks/useApi';
import { useFmt, useT } from '@/i18n/client';

const PAGE_SIZE = 25;

export default function BlocksPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = useBlocks(page, PAGE_SIZE);
  const { t, tp } = useT();
  const fmt = useFmt();

  if (error && !data) {
    return (
      <>
        <PageHeader title={t('blocks.title')} />
        <ErrorState message={t('blocks.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('blocks.title')}
        subtitle={
          data ? tp('blocks.indexed', data.pagination.total) : t('blocks.loading')
        }
      />

      <BlocksTable blocks={data?.data ?? []} isLoading={isLoading && !data} />

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
