'use client';

import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { Pagination } from '@/components/Pagination';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { ErrorState, PageHeader } from '@/components/States';
import { useNotes, useStats } from '@/hooks/useApi';
import { L, useFmt, useT, type Fmt } from '@/i18n/client';
import type { Note } from '@/types';

const PAGE_SIZE = 25;

const noteColumns = (t: (key: string) => string, fmt: Fmt): Column<Note>[] => [
  {
    key: 'leaf_index',
    header: t('notes.columns.leaf'),
    render: (note) => (
      <L href={`/notes/${note.cm}`} className="link font-mono">
        #{fmt.number(note.leaf_index)}
      </L>
    ),
  },
  {
    key: 'cm',
    header: t('notes.columns.commitment'),
    render: (note) => <Hash value={note.cm} href={`/notes/${note.cm}`} start={12} end={8} />,
  },
  {
    key: 'height',
    header: t('notes.columns.height'),
    render: (note) => (
      <L href={`/blocks/${note.height}`} className="link font-mono">
        {fmt.number(note.height)}
      </L>
    ),
  },
  {
    key: 'tx_hash',
    header: t('notes.columns.createdBy'),
    render: (note) =>
      note.tx_hash ? (
        <Hash value={note.tx_hash} href={`/transactions/${note.tx_hash}`} />
      ) : (
        <span className="text-mute">{note.height === 0 ? t('notes.genesis') : t('notes.deposit')}</span>
      ),
  },
];

export default function NotesPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = useNotes(page, PAGE_SIZE);
  const { data: stats } = useStats();
  const { t, tp } = useT();
  const fmt = useFmt();
  const columns = noteColumns(t, fmt);

  if (error && !data) {
    return (
      <>
        <PageHeader title={t('notes.title')} />
        <ErrorState message={t('notes.error')} onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('notes.title')}
        subtitle={
          data ? tp('notes.leaves', data.pagination.total) : t('notes.loading')
        }
      />

      <StatsRow columns={3}>
        <StatsCard title={t('notes.created')} value={fmt.number(stats?.notes ?? 0)} subtitle={t('notes.createdSub')} />
        <StatsCard title={t('notes.spent')} value={fmt.number(stats?.nullifiers ?? 0)} subtitle={t('notes.spentSub')} />
        <StatsCard
          title={t('notes.treeRoot')}
          value={stats?.tree_root ? `${stats.tree_root.slice(0, 10)}…` : '—'}
          subtitle={t('notes.treeRootSub')}
        />
      </StatsRow>

      <DataTable
        columns={columns}
        data={data?.data ?? []}
        keyExtractor={(note) => String(note.leaf_index)}
        isLoading={isLoading && !data}
        emptyMessage={t('notes.empty')}
      />

      {data && (
        <Pagination
          currentPage={data.pagination.page}
          totalPages={data.pagination.total_pages}
          onPageChange={setPage}
        />
      )}

      <p className="text-xs text-mute">
        {t('notes.help')}
      </p>
    </div>
  );
}
