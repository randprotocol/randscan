'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { Pagination } from '@/components/Pagination';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { ErrorState, PageHeader } from '@/components/States';
import { useNotes, useStats } from '@/hooks/useApi';
import { formatNumber } from '@/lib/utils';
import type { Note } from '@/types';

const PAGE_SIZE = 25;

const columns: Column<Note>[] = [
  {
    key: 'leaf_index',
    header: 'Leaf',
    render: (note) => (
      <Link href={`/notes/${note.cm}`} className="link font-mono">
        #{formatNumber(note.leaf_index)}
      </Link>
    ),
  },
  {
    key: 'cm',
    header: 'Commitment',
    render: (note) => <Hash value={note.cm} href={`/notes/${note.cm}`} start={12} end={8} />,
  },
  {
    key: 'height',
    header: 'Height',
    render: (note) => (
      <Link href={`/blocks/${note.height}`} className="link font-mono">
        {formatNumber(note.height)}
      </Link>
    ),
  },
  {
    key: 'tx_hash',
    header: 'Created by',
    render: (note) =>
      note.tx_hash ? (
        <Hash value={note.tx_hash} href={`/transactions/${note.tx_hash}`} />
      ) : (
        <span className="text-mute">{note.height === 0 ? 'genesis' : 'deposit'}</span>
      ),
  },
];

export default function NotesPage() {
  const [page, setPage] = useState(1);
  const { data, error, isLoading, mutate } = useNotes(page, PAGE_SIZE);
  const { data: stats } = useStats();

  if (error && !data) {
    return (
      <>
        <PageHeader title="Notes" />
        <ErrorState message="Could not load the commitment tree." onRetry={() => void mutate()} />
      </>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Notes"
        subtitle={
          data ? `${formatNumber(data.pagination.total)} leaves in the commitment tree` : 'Loading notes…'
        }
      />

      <StatsRow columns={3}>
        <StatsCard title="Notes created" value={formatNumber(stats?.notes ?? 0)} subtitle="tree leaves" />
        <StatsCard title="Notes spent" value={formatNumber(stats?.nullifiers ?? 0)} subtitle="nullifiers published" />
        <StatsCard
          title="Tree root"
          value={stats?.tree_root ? `${stats.tree_root.slice(0, 10)}…` : '—'}
          subtitle="the anchor a wallet proves against"
        />
      </StatsRow>

      <DataTable
        columns={columns}
        data={data?.data ?? []}
        keyExtractor={(note) => String(note.leaf_index)}
        isLoading={isLoading && !data}
        emptyMessage="No leaves indexed yet"
      />

      {data && (
        <Pagination
          currentPage={data.pagination.page}
          totalPages={data.pagination.total_pages}
          onPageChange={setPage}
        />
      )}

      <p className="text-xs text-mute">
        Every note is a commitment to a hidden owner and amount. A leaf shows which transaction
        created it when the commitment was on the wire; genesis, withdraw and bridge deposit
        notes are not linked. Which leaf a nullifier spends is never public.
      </p>
    </div>
  );
}
