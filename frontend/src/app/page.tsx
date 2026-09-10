'use client';

import Link from 'next/link';
import { useCallback, useEffect, useState } from 'react';
import { StatsCard, StatsCardSkeleton, StatsRow } from '@/components/StatsCard';
import { BlocksTable, TransactionsTable } from '@/components/Tables';
import { Hash } from '@/components/Hash';
import { LiveIndicator, SectionHeading } from '@/components/States';
import { useLatestBlocks, useLatestTransactions, useStats } from '@/hooks/useApi';
import { useNewBlocks, useNewTransactions, useStatsUpdates } from '@/hooks/useWebSocket';
import {
  formatAmount,
  formatDurationMs,
  formatNumber,
  formatStake,
  TOKEN_SYMBOL,
} from '@/lib/utils';
import type { BlockSummary, NetworkStats, TransactionSummary } from '@/types';

const LATEST_LIMIT = 10;

export default function DashboardPage() {
  const { data: fetchedStats, isLoading: statsLoading } = useStats();
  const { data: fetchedBlocks, isLoading: blocksLoading } = useLatestBlocks(LATEST_LIMIT);
  const { data: fetchedTxs, isLoading: txsLoading } = useLatestTransactions(LATEST_LIMIT);

  const [liveStats, setLiveStats] = useState<NetworkStats | null>(null);
  const [liveBlocks, setLiveBlocks] = useState<BlockSummary[]>([]);
  const [liveTxs, setLiveTxs] = useState<TransactionSummary[]>([]);

  // Whenever a fresh REST payload lands, drop the locally accumulated deltas.
  useEffect(() => {
    if (fetchedBlocks) setLiveBlocks([]);
  }, [fetchedBlocks]);

  useEffect(() => {
    if (fetchedTxs) setLiveTxs([]);
  }, [fetchedTxs]);

  const onNewBlock = useCallback((block: BlockSummary) => {
    setLiveBlocks((current) =>
      current.some((b) => b.hash === block.hash) ? current : [block, ...current].slice(0, LATEST_LIMIT)
    );
  }, []);

  const onNewTransaction = useCallback((tx: TransactionSummary) => {
    setLiveTxs((current) =>
      current.some((t) => t.hash === tx.hash) ? current : [tx, ...current].slice(0, LATEST_LIMIT)
    );
  }, []);

  const { isConnected } = useNewBlocks(onNewBlock);
  useNewTransactions(onNewTransaction);
  useStatsUpdates(setLiveStats);

  const stats = liveStats ?? fetchedStats;

  const blocks = dedupe(
    [...liveBlocks, ...(fetchedBlocks ?? [])],
    (block) => block.hash
  ).slice(0, LATEST_LIMIT);

  const transactions = dedupe(
    [...liveTxs, ...(fetchedTxs ?? [])],
    (tx) => tx.hash
  ).slice(0, LATEST_LIMIT);

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="font-serif text-4xl font-medium tracking-tight text-strong">
            Rand Protocol Explorer
          </h1>
          <p className="mt-2 text-sm text-soft">
            {stats
              ? `Chain ${stats.chain_id} · ${stats.symbol} · ${stats.decimals} decimals`
              : 'Connecting to the network…'}
          </p>
        </div>
        <LiveIndicator isConnected={isConnected} />
      </div>

      {/* Primary stats */}
      <StatsRow columns={4}>
        {statsLoading && !stats ? (
          Array.from({ length: 4 }).map((_, i) => <StatsCardSkeleton key={i} />)
        ) : (
          <>
            <StatsCard
              title="Block height"
              value={formatNumber(stats?.height ?? 0)}
              subtitle={stats?.node_syncing ? 'Node syncing' : 'In sync'}
            />
            <StatsCard
              title="Validators"
              value={formatNumber(stats?.validator_count ?? 0)}
              subtitle={stats ? formatStake(stats.total_stake) : undefined}
            />
            <StatsCard
              title="Transactions"
              value={formatNumber(stats?.total_transactions ?? 0)}
              subtitle={stats ? `${formatNumber(stats.total_accounts)} accounts` : undefined}
            />
            <StatsCard
              title="Programs"
              value={formatNumber(stats?.program_count ?? 0)}
              subtitle={stats?.confidential ? 'Confidential calls enabled' : 'Confidential calls off'}
            />
          </>
        )}
      </StatsRow>

      {/* Secondary stats */}
      <StatsRow columns={5}>
        <MiniStat label="Avg block time" value={stats ? formatDurationMs(stats.avg_block_time_ms) : '—'} />
        <MiniStat label="Peers" value={stats ? formatNumber(stats.peer_count) : '—'} />
        <MiniStat label="Mempool" value={stats ? formatNumber(stats.mempool_size) : '—'} />
        <MiniStat
          label="Chain / view"
          value={stats ? `${stats.chain_id} / ${formatNumber(stats.view)}` : '—'}
        />
        <MiniStat
          label="Current leader"
          value={
            stats?.current_leader ? (
              <Hash
                value={stats.current_leader}
                href={`/validators/${stats.current_leader}`}
                start={6}
                end={4}
              />
            ) : (
              '—'
            )
          }
        />
      </StatsRow>

      {stats && (
        <p className="text-xs text-mute">
          Total supply {formatAmount(stats.total_supply)} · faucet{' '}
          {stats.faucet ? 'enabled' : 'disabled'} · {TOKEN_SYMBOL} has {stats.decimals} decimals
        </p>
      )}

      {/* Latest blocks + transactions */}
      <div className="grid gap-6 xl:grid-cols-2">
        <section className="space-y-4">
          <SectionHeading
            label="Latest blocks"
            actions={
              <Link href="/blocks" className="link text-sm">
                View all →
              </Link>
            }
          />
          <BlocksTable blocks={blocks} isLoading={blocksLoading && blocks.length === 0} />
        </section>

        <section className="space-y-4">
          <SectionHeading
            label="Latest transactions"
            actions={
              <Link href="/transactions" className="link text-sm">
                View all →
              </Link>
            }
          />
          <TransactionsTable
            transactions={transactions}
            isLoading={txsLoading && transactions.length === 0}
            hideColumns={['fee', 'to']}
          />
        </section>
      </div>
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="stat">
      <div className="truncate text-lg font-medium text-strong">{value}</div>
      <p className="stat-label">{label}</p>
    </div>
  );
}

function dedupe<T>(items: T[], key: (item: T) => string): T[] {
  const seen = new Set<string>();
  const out: T[] = [];
  for (const item of items) {
    const k = key(item);
    if (seen.has(k)) continue;
    seen.add(k);
    out.push(item);
  }
  return out;
}
