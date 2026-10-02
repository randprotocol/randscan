'use client';

import { useState } from 'react';
import { Pagination } from '@/components/Pagination';
import { BlocksTable } from '@/components/Tables';
import { ErrorState, PageHeader } from '@/components/States';
import { useBlocks } from '@/hooks/useApi';
import { formatNumber } from '@/lib/utils';

const PAGE_SIZE = 25;

export default function BlocksPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = useBlocks(page, PAGE_SIZE);

  if (error && !data) {
    return (
      <>
        <PageHeader title="Blocks" />
        <ErrorState message="Could not load blocks." onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Blocks"
        subtitle={
          data ? `${formatNumber(data.pagination.total)} blocks indexed` : 'Loading blocks…'
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
