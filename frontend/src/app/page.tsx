'use client';

import Link from 'next/link';
import { useStats, useLatestBlocks, useLatestTransactions } from '@/hooks/useApi';
import { useBlocksSubscription, useStatsSubscription } from '@/hooks/useWebSocket';
import { StatsCard, StatsCardSkeleton } from '@/components/StatsCard';
import { DataTable } from '@/components/DataTable';
import {
  formatNumber,
  formatCompactNumber,
  formatTimestamp,
  formatLamports,
  shortenHash,
  formatPercentage,
  getStatusColorClass,
  getTransactionTypeLabel,
} from '@/lib/utils';
import type { BlockSummary, TransactionSummary, WebSocketBlockUpdate, WebSocketStatsUpdate } from '@/types';
import { useCallback, useState } from 'react';

export default function DashboardPage() {
  const { data: stats, mutate: mutateStats } = useStats();
  const { data: blocks, mutate: mutateBlocks } = useLatestBlocks(5);
  const { data: transactions } = useLatestTransactions(5);

  // State for real-time updates
  const [realtimeBlocks, setRealtimeBlocks] = useState<BlockSummary[]>([]);

  // Handle real-time block updates
  const handleBlockUpdate = useCallback((block: WebSocketBlockUpdate) => {
    const newBlock: BlockSummary = {
      slot: block.slot,
      blockhash: block.blockhash,
      parent_slot: block.slot - 1,
      timestamp: block.timestamp,
      transaction_count: block.transaction_count,
      leader: block.leader,
      rewards: '0',
    };

    setRealtimeBlocks((prev) => [newBlock, ...prev.slice(0, 4)]);
    mutateBlocks();
  }, [mutateBlocks]);

  // Handle real-time stats updates
  const handleStatsUpdate = useCallback((newStats: WebSocketStatsUpdate) => {
    if (stats) {
      mutateStats({
        ...stats,
        block_height: newStats.block_height,
        tps: newStats.tps,
        total_transactions: newStats.total_transactions,
      }, false);
    }
  }, [stats, mutateStats]);

  // Subscribe to real-time updates
  const { isConnected } = useBlocksSubscription(handleBlockUpdate);
  useStatsSubscription(handleStatsUpdate);

  // Merge real-time blocks with fetched blocks
  const displayBlocks = realtimeBlocks.length > 0 ? realtimeBlocks : blocks;

  const blockColumns = [
    {
      key: 'slot',
      header: 'Slot',
      render: (block: BlockSummary) => (
        <Link href={`/blocks/${block.slot}`} className="link font-mono">
          {formatNumber(block.slot)}
        </Link>
      ),
    },
    {
      key: 'timestamp',
      header: 'Age',
      render: (block: BlockSummary) => (
        <span className="text-dark-300">{formatTimestamp(block.timestamp)}</span>
      ),
    },
    {
      key: 'transaction_count',
      header: 'Txns',
      render: (block: BlockSummary) => (
        <span className="text-white">{block.transaction_count}</span>
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
  ];

  const txColumns = [
    {
      key: 'signature',
      header: 'Signature',
      render: (tx: TransactionSummary) => (
        <Link href={`/transactions/${tx.signature}`} className="link font-mono">
          {shortenHash(tx.signature, 8, 8)}
        </Link>
      ),
    },
    {
      key: 'type',
      header: 'Type',
      render: (tx: TransactionSummary) => (
        <span className="badge badge-neutral">{getTransactionTypeLabel(tx.type)}</span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (tx: TransactionSummary) => (
        <span className={getStatusColorClass(tx.status)}>
          {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
        </span>
      ),
    },
    {
      key: 'fee',
      header: 'Fee',
      render: (tx: TransactionSummary) => (
        <span className="text-dark-300">{formatLamports(tx.fee)} ATLAS</span>
      ),
    },
  ];

  return (
    <div className="space-y-8">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Dashboard</h1>
          <p className="mt-1 text-dark-400">
            Overview of the Rand Protocol network
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span
            className={`status-dot ${isConnected ? 'status-dot-success' : 'status-dot-error'}`}
          />
          <span className="text-sm text-dark-400">
            {isConnected ? 'Live' : 'Connecting...'}
          </span>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats ? (
          <>
            <StatsCard
              title="Block Height"
              value={formatNumber(stats.block_height)}
              subtitle={`Slot ${formatNumber(stats.slot)}`}
              icon={
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                </svg>
              }
            />
            <StatsCard
              title="TPS"
              value={stats.tps.toFixed(0)}
              subtitle={`Avg: ${stats.average_tps.toFixed(1)}`}
              icon={
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              }
            />
            <StatsCard
              title="Validators"
              value={formatNumber(stats.active_validators)}
              subtitle={`Stake: ${formatCompactNumber(Number(stats.total_stake) / 1e9)} ATLAS`}
              icon={
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              }
            />
            <StatsCard
              title="Epoch"
              value={formatNumber(stats.epoch)}
              subtitle={`Progress: ${formatPercentage(stats.epoch_progress)}`}
              icon={
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              }
            />
          </>
        ) : (
          <>
            <StatsCardSkeleton />
            <StatsCardSkeleton />
            <StatsCardSkeleton />
            <StatsCardSkeleton />
          </>
        )}
      </div>

      {/* Latest Blocks & Transactions */}
      <div className="grid gap-8 lg:grid-cols-2">
        {/* Latest Blocks */}
        <div>
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-white">Latest Blocks</h2>
            <Link href="/blocks" className="link text-sm">
              View all
            </Link>
          </div>
          <DataTable
            columns={blockColumns}
            data={displayBlocks || []}
            keyExtractor={(block) => block.slot.toString()}
            onRowClick={(block) => window.location.href = `/blocks/${block.slot}`}
            isLoading={!displayBlocks}
          />
        </div>

        {/* Latest Transactions */}
        <div>
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-white">Latest Transactions</h2>
            <Link href="/transactions" className="link text-sm">
              View all
            </Link>
          </div>
          <DataTable
            columns={txColumns}
            data={transactions || []}
            keyExtractor={(tx) => tx.signature}
            onRowClick={(tx) => window.location.href = `/transactions/${tx.signature}`}
            isLoading={!transactions}
          />
        </div>
      </div>
    </div>
  );
}
