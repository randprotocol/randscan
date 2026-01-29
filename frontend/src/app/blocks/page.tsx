'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useBlocks } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { Pagination } from '@/components/Pagination';
import {
  formatNumber,
  formatTimestamp,
  shortenHash,
} from '@/lib/utils';
import type { BlockSummary } from '@/types';

export default function BlocksPage() {
  const [page, setPage] = useState(1);
  const pageSize = 20;

  const { data: blocksResponse, isLoading } = useBlocks(page, pageSize);

  const columns = [
    {
      key: 'slot',
      header: 'Slot',
      render: (block: BlockSummary) => (
        <Link href={`/blocks/${block.slot}`} className="link font-mono font-medium">
          {formatNumber(block.slot)}
        </Link>
      ),
    },
    {
      key: 'blockhash',
      header: 'Block Hash',
      render: (block: BlockSummary) => (
        <Link href={`/blocks/${block.slot}`} className="link font-mono">
          {shortenHash(block.blockhash, 8, 8)}
        </Link>
      ),
    },
    {
      key: 'timestamp',
      header: 'Time',
      render: (block: BlockSummary) => (
        <span className="text-dark-300">{formatTimestamp(block.timestamp)}</span>
      ),
    },
    {
      key: 'transaction_count',
      header: 'Transactions',
      className: 'text-right',
      render: (block: BlockSummary) => (
        <span className="text-white">{formatNumber(block.transaction_count)}</span>
      ),
    },
    {
      key: 'leader',
      header: 'Leader',
      render: (block: BlockSummary) => (
        <Link href={`/validators/${block.leader}`} className="link font-mono">
          {shortenHash(block.leader)}
        </Link>
      ),
    },
    {
      key: 'rewards',
      header: 'Rewards',
      className: 'text-right',
      render: (block: BlockSummary) => (
        <span className="text-dark-300">{formatNumber(Number(block.rewards))} lamports</span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Blocks</h1>
        <p className="mt-1 text-dark-400">
          Browse all blocks on the Rand Protocol network
        </p>
      </div>

      {/* Stats */}
      {blocksResponse && (
        <div className="flex items-center gap-4 text-sm text-dark-400">
          <span>
            Total: <span className="text-white">{formatNumber(blocksResponse.total)}</span> blocks
          </span>
          <span>|</span>
          <span>
            Page <span className="text-white">{blocksResponse.page}</span> of{' '}
            <span className="text-white">{blocksResponse.total_pages}</span>
          </span>
        </div>
      )}

      {/* Blocks Table */}
      <DataTable
        columns={columns}
        data={blocksResponse?.data || []}
        keyExtractor={(block) => block.slot.toString()}
        onRowClick={(block) => window.location.href = `/blocks/${block.slot}`}
        isLoading={isLoading}
        emptyMessage="No blocks found"
      />

      {/* Pagination */}
      {blocksResponse && blocksResponse.total_pages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={blocksResponse.total_pages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
