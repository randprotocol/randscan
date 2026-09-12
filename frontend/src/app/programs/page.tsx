'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { Pagination } from '@/components/Pagination';
import { ErrorState, PageHeader } from '@/components/States';
import { usePrograms } from '@/hooks/useApi';
import { formatNumber } from '@/lib/utils';
import type { ProgramSummary } from '@/types';

const PAGE_SIZE = 25;

const columns: Column<ProgramSummary>[] = [
  {
    key: 'id',
    header: 'Program',
    render: (program) => <Hash value={program.id} href={`/programs/${program.id}`} />,
  },
  {
    key: 'deployed_at_height',
    header: 'Deployed at',
    render: (program) => (
      <Link href={`/blocks/${program.deployed_at_height}`} className="link font-mono">
        #{formatNumber(program.deployed_at_height)}
      </Link>
    ),
  },
  {
    key: 'words_len',
    header: 'Words',
    render: (program) => <span className="text-soft">{formatNumber(program.words_len)}</span>,
  },
  {
    key: 'call_count',
    header: 'Calls',
    render: (program) => <span className="text-soft">{formatNumber(program.call_count)}</span>,
  },
  {
    key: 'last_called_height',
    header: 'Last called',
    render: (program) =>
      program.last_called_height === null ? (
        <span className="text-mute">Never</span>
      ) : (
        <Link href={`/blocks/${program.last_called_height}`} className="link font-mono">
          #{formatNumber(program.last_called_height)}
        </Link>
      ),
  },
];

export default function ProgramsPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = usePrograms(page, PAGE_SIZE);

  if (error && !data) {
    return (
      <>
        <PageHeader title="Programs" />
        <ErrorState message="Could not load programs." onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Programs"
        subtitle={
          data ? `${formatNumber(data.pagination.total)} programs deployed` : 'Loading programs…'
        }
      />

      <DataTable
        columns={columns}
        data={data?.data ?? []}
        keyExtractor={(program) => program.id}
        isLoading={isLoading && !data}
        emptyMessage="No programs deployed yet"
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
