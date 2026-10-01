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
  formatBinaryBytes,
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
          <h1 className="text-4xl font-semibold tracking-tight text-strong">
            Rand Protocol Explorer
          </h1>
          <p className="mt-2 text-sm text-soft">
            {stats
              ? `Chain ${stats.chain_id}`
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
              value={
                stats
                  ? `${formatNumber(stats.active_validator_count)} / ${formatNumber(stats.validator_count)}`
                  : '0'
              }
              subtitle={stats ? formatStake(stats.total_stake, 'active stake') : undefined}
            />
            <StatsCard
              title="Transactions"
              value={formatNumber(stats?.total_transactions ?? 0)}
              subtitle={stats ? `${formatNumber(stats.program_count)} programs` : undefined}
            />
            <StatsCard
              title="Notes"
              value={formatNumber(stats?.notes ?? 0)}
              subtitle={stats ? `${formatNumber(stats.nullifiers)} spent` : undefined}
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
          label="Epoch"
          value={
            stats && stats.epoch !== null
              ? `${formatNumber(stats.epoch)} (${formatNumber(stats.epoch_blocks ?? 0)} blocks)`
              : '—'
          }
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
        <p className="flex flex-wrap items-center gap-x-2 text-xs text-mute">
          <span>
            {stats.total_supply !== '0'
              ? `Total supply ${formatAmount(stats.total_supply)}`
              : 'Supply audit unavailable'}
          </span>
          <span>· view {formatNumber(stats.view)}</span>
          <span className="inline-flex items-center gap-1">
            · tree root{' '}
            {stats.tree_root ? <Hash value={stats.tree_root} start={8} end={6} /> : '—'}
          </span>
          <span className="inline-flex items-center gap-1">
            · bundle guest{' '}
            {stats.hc_bundle ? <Hash value={stats.hc_bundle} start={8} end={6} /> : '—'}
          </span>
          <span>· faucet {stats.faucet ? 'enabled' : 'disabled'}</span>
          <span>· confidential calls {stats.confidential ? 'on' : 'off'}</span>
          <span>
            · {TOKEN_SYMBOL} has {stats.decimals} decimals
          </span>
          {stats.genesis_hash && (
            <span className="inline-flex items-center gap-1">
              · genesis <Hash value={stats.genesis_hash} start={8} end={6} />
            </span>
          )}
          {stats.node_version && (
            <span>
              · node {stats.node_version}
              {stats.node_git_sha ? ` (${stats.node_git_sha.slice(0, 7)})` : ''}
            </span>
          )}
        </p>
      )}

      {stats?.limits && (
        <p className="flex flex-wrap items-center gap-x-2 text-xs text-mute">
          <span>Limits: programs {formatNumber(stats.limits.max_program_words)} words</span>
          <span>· proofs {formatBinaryBytes(stats.limits.max_proof_bytes)}</span>
          <span>· blocks {formatBinaryBytes(stats.limits.max_block_bytes)}</span>
          <span>· call envelopes {formatBinaryBytes(stats.limits.max_call_envelope_bytes)}</span>
          <span>
            · public input{' '}
            {stats.limits.max_program_public_words > 0
              ? `${formatNumber(stats.limits.max_program_public_words)} words`
              : 'off'}
          </span>
          <span>
            · note envelopes{' '}
            {stats.limits.envelope_bytes
              ? `${formatNumber(stats.limits.envelope_bytes)} B (memo)`
              : '1,348 B (no memo)'}
          </span>
          <span>· v0.6 rules {stats.limits.hardening_v6 ? 'on' : 'off'}</span>
          <span className="inline-flex items-center gap-1">
            · auth guest{' '}
            {stats.limits.hc_auth ? <Hash value={stats.limits.hc_auth} start={8} end={6} /> : 'none'}
          </span>
          <span>· proof window {formatNumber(stats.limits.proof_window_blocks ?? 256)} blocks</span>
          {stats.limits.binding_domain != null && (
            <span>
              · bindings over {stats.limits.binding_domain === 1 ? 'the genesis hash' : 'the chain id'}
            </span>
          )}
        </p>
      )}

      {stats?.limits && (stats.limits.testnet || stats.limits.admission_by_vote || stats.limits.slashing) && (
        <p className="flex flex-wrap items-center gap-x-2 text-xs text-mute">
          <span>Network:{stats.limits.testnet ? ' testnet (the genesis marker)' : ' mainnet'}</span>
          {stats.limits.admission_by_vote && (
            <span>
              · new validators admitted by the set&apos;s vote (<Link href="/validators" className="link">admitted keys</Link>)
            </span>
          )}
          <span>
            · equivocation slashing{' '}
            {stats.limits.slashing
              ? `${(stats.limits.slashing.equivocation_bps / 100).toFixed(2)}% of stake, jailed ${formatNumber(stats.limits.slashing.jail_epochs)} epochs`
              : 'off'}
          </span>
        </p>
      )}

      {stats?.limits && stats.limits.gas_metering && (
        <p className="flex flex-wrap items-center gap-x-2 text-xs text-mute">
          <span>
            Gas:{' '}
            {stats.limits.gas_metering === 'circuit'
              ? "the chain's own section, metered in-circuit"
              : "this node's policy, priced off the proof header"}
          </span>
          <span>
            · {formatAmount((stats.gas_prices ?? stats.limits).gas_price ?? '0')} per gas
          </span>
          <span>
            · {formatAmount((stats.gas_prices ?? stats.limits).byte_price ?? '0')} per KiB
          </span>
          {(stats.limits.max_gas_price || stats.limits.max_byte_price) && (
            <span>
              · ceilings {formatAmount(stats.limits.max_gas_price ?? '0')} / gas,{' '}
              {formatAmount(stats.limits.max_byte_price ?? '0')} / KiB
              {stats.limits.byte_load === 'paying' ? ' (only paying bytes move the byte price)' : ''}
            </span>
          )}
          {stats.limits.bundle_gas_limit !== null && (
            <span>· every bundle declares {formatNumber(stats.limits.bundle_gas_limit)} gas</span>
          )}
          <span>
            · prices{' '}
            {stats.limits.adjust_bps !== null
              ? `move ${(stats.limits.adjust_bps / 100).toFixed(2).replace(/\.?0+$/, '')}% a block by fullness`
              : 'fixed'}
          </span>
          {stats.limits.program_state && (
            <span>
              · program state on: {formatAmount(stats.limits.program_state.cell_fee)} a cell
              created, at most {formatNumber(stats.limits.program_state.max_reads)} reads,{' '}
              {formatNumber(stats.limits.program_state.max_writes)} writes and{' '}
              {formatNumber(stats.limits.program_state.max_payouts)} payouts an invoke
            </span>
          )}
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
            hideColumns={['fee']}
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
