'use client';

import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { Pagination } from '@/components/Pagination';
import { ErrorState, PageHeader } from '@/components/States';
import { usePrograms } from '@/hooks/useApi';
import { L, useFmt, useT, type Fmt } from '@/i18n/client';
import type { ProgramSummary } from '@/types';

const PAGE_SIZE = 25;

const programColumns = (t: (key: string) => string, fmt: Fmt): Column<ProgramSummary>[] => [
  {
    key: 'id',
    header: t('programs.columns.program'),
    render: (program) => <Hash value={program.id} href={`/programs/${program.id}`} />,
  },
  {
    key: 'deployed_at_height',
    header: t('programs.columns.deployedAt'),
    render: (program) => (
      <L href={`/blocks/${program.deployed_at_height}`} className="link font-mono">
        #{fmt.number(program.deployed_at_height)}
      </L>
    ),
  },
  {
    key: 'words_len',
    header: t('programs.columns.words'),
    render: (program) => <span className="text-soft">{fmt.number(program.words_len)}</span>,
  },
  {
    key: 'call_count',
    header: t('programs.columns.calls'),
    render: (program) => <span className="text-soft">{fmt.number(program.call_count)}</span>,
  },
  {
    key: 'last_called_height',
    header: t('programs.columns.lastCalled'),
    render: (program) =>
      program.last_called_height === null ? (
        <span className="text-mute">{t('programs.never')}</span>
      ) : (
        <L href={`/blocks/${program.last_called_height}`} className="link font-mono">
          #{fmt.number(program.last_called_height)}
        </L>
      ),
  },
];

export default function ProgramsPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = usePrograms(page, PAGE_SIZE);
  const { t, tp } = useT();
  const fmt = useFmt();
  const columns = programColumns(t, fmt);

  if (error && !data) {
    return (
      <>
        <PageHeader title={t('programs.title')} />
        <ErrorState message={t('programs.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('programs.title')}
        subtitle={
          data ? tp('programs.deployed', data.pagination.total) : t('programs.loading')
        }
      />

      <DataTable
        columns={columns}
        data={data?.data ?? []}
        keyExtractor={(program) => program.id}
        isLoading={isLoading && !data}
        emptyMessage={t('programs.empty')}
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
