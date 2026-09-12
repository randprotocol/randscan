'use client';

import Link from 'next/link';
import { Column, DataTable } from './DataTable';
import { Hash } from './Hash';
import { KindBadge } from './KindBadge';
import { formatAmount, formatAssetAmount, formatNumber, formatTimestamp } from '@/lib/utils';
import type { BlockSummary, TransactionSummary } from '@/types';

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

export function blockColumns(): Column<BlockSummary>[] {
  return [
    {
      key: 'height',
      header: 'Height',
      render: (block) => (
        <Link href={`/blocks/${block.height}`} className="link font-mono">
          {formatNumber(block.height)}
        </Link>
      ),
    },
    {
      key: 'hash',
      header: 'Hash',
      render: (block) => <Hash value={block.hash} href={`/blocks/${block.hash}`} />,
    },
    {
      key: 'proposer',
      header: 'Proposer',
      render: (block) => (
        <Hash value={block.proposer} href={`/validators/${block.proposer}`} start={6} end={6} />
      ),
    },
    {
      key: 'tx_count',
      header: 'Txs',
      render: (block) => <span className="font-mono text-soft">{formatNumber(block.tx_count)}</span>,
    },
    {
      key: 'view',
      header: 'View',
      render: (block) => <span className="font-mono text-soft">{formatNumber(block.view)}</span>,
    },
    {
      key: 'timestamp_ms',
      header: 'Age',
      render: (block) => (
        <span className="text-mute" title={String(block.timestamp_ms)}>
          {formatTimestamp(block.timestamp_ms)}
        </span>
      ),
    },
  ];
}

interface BlocksTableProps {
  blocks: BlockSummary[];
  isLoading?: boolean;
  emptyMessage?: string;
}

export function BlocksTable({ blocks, isLoading, emptyMessage }: BlocksTableProps) {
  return (
    <DataTable
      columns={blockColumns()}
      data={blocks}
      keyExtractor={(block) => block.hash}
      isLoading={isLoading}
      emptyMessage={emptyMessage ?? 'No blocks indexed yet'}
    />
  );
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

/**
 * What the action touched: the program of a deploy/call, the validator of a staking move or
 * mint, the public amount of a deposit. A plain transfer shows nothing, by design.
 */
function actionCell(tx: TransactionSummary) {
  const parts: React.ReactNode[] = [];
  if (tx.program) {
    parts.push(
      <Hash key="program" value={tx.program} href={`/programs/${tx.program}`} start={6} end={6} />
    );
  }
  if (tx.validator) {
    parts.push(
      <Hash
        key="validator"
        value={tx.validator}
        href={`/validators/${tx.validator}`}
        start={6}
        end={6}
      />
    );
  }
  if (tx.amount !== null) {
    parts.push(
      <span key="amount" className="font-mono text-text">
        {formatAssetAmount(tx.amount, tx.asset_index)}
      </span>
    );
  }
  if (parts.length === 0) {
    return (
      <span className="text-mute" title="Sender, receiver and amount are hidden inside the bundle">
        {tx.kind === 'transfer' ? 'shielded' : '—'}
      </span>
    );
  }
  return <span className="inline-flex flex-wrap items-center gap-2">{parts}</span>;
}

export function transactionColumns(): Column<TransactionSummary>[] {
  return [
    {
      key: 'hash',
      header: 'Tx hash',
      render: (tx) => <Hash value={tx.hash} href={`/transactions/${tx.hash}`} />,
    },
    {
      key: 'kind',
      header: 'Kind',
      render: (tx) => <KindBadge kind={tx.kind} short />,
    },
    {
      key: 'height',
      header: 'Height',
      render: (tx) => (
        <Link href={`/blocks/${tx.height}`} className="link font-mono">
          {formatNumber(tx.height)}
        </Link>
      ),
    },
    {
      key: 'action',
      header: 'Action',
      render: actionCell,
    },
    {
      key: 'fee',
      header: 'Fee',
      render: (tx) =>
        tx.has_bundle ? (
          <span className="font-mono text-mute">{formatAmount(tx.fee)}</span>
        ) : (
          <span className="text-mute" title="Validator-signed action: no bundle, no fee">
            —
          </span>
        ),
    },
    {
      key: 'timestamp_ms',
      header: 'Age',
      render: (tx) => (
        <span className="text-mute" title={String(tx.timestamp_ms)}>
          {formatTimestamp(tx.timestamp_ms)}
        </span>
      ),
    },
  ];
}

interface TransactionsTableProps {
  transactions: TransactionSummary[];
  isLoading?: boolean;
  emptyMessage?: string;
  /** Drop columns that are redundant in the surrounding context. */
  hideColumns?: string[];
}

export function TransactionsTable({
  transactions,
  isLoading,
  emptyMessage,
  hideColumns = [],
}: TransactionsTableProps) {
  const columns = transactionColumns().filter((column) => !hideColumns.includes(column.key));

  return (
    <DataTable
      columns={columns}
      data={transactions}
      keyExtractor={(tx) => tx.hash}
      isLoading={isLoading}
      emptyMessage={emptyMessage ?? 'No transactions found'}
    />
  );
}
