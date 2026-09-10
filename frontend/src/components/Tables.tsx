'use client';

import Link from 'next/link';
import { Column, DataTable } from './DataTable';
import { Hash } from './Hash';
import { KindBadge } from './KindBadge';
import { formatAmount, formatNumber, formatTimestamp } from '@/lib/utils';
import type { AccountTransaction, BlockSummary, TransactionSummary } from '@/types';

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

/** Amount cell: transfers/mints carry an amount, deploys/calls usually do not. */
function amountCell(tx: TransactionSummary) {
  if (tx.amount === null) return <span className="text-mute">—</span>;
  return <span className="font-mono text-text">{formatAmount(tx.amount)}</span>;
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
      key: 'sender',
      header: 'From',
      render: (tx) => <Hash value={tx.sender} href={`/account/${tx.sender}`} start={6} end={6} />,
    },
    {
      key: 'to',
      header: 'To',
      render: (tx) =>
        tx.to ? (
          <Hash value={tx.to} href={`/account/${tx.to}`} start={6} end={6} />
        ) : tx.program ? (
          <Hash value={tx.program} href={`/programs/${tx.program}`} start={6} end={6} />
        ) : (
          <span className="text-mute">—</span>
        ),
    },
    {
      key: 'amount',
      header: 'Amount',
      render: amountCell,
    },
    {
      key: 'fee',
      header: 'Fee',
      render: (tx) => <span className="font-mono text-mute">{formatAmount(tx.fee)}</span>,
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

// ---------------------------------------------------------------------------
// Account transactions (adds the sender/recipient role)
// ---------------------------------------------------------------------------

function RoleCell({ role }: { role: AccountTransaction['role'] }) {
  const outgoing = role === 'sender';
  return (
    <span
      className={
        outgoing
          ? 'inline-flex items-center gap-1.5 text-accent-3'
          : 'inline-flex items-center gap-1.5 text-accent'
      }
    >
      <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d={outgoing ? 'M14 5l7 7m0 0l-7 7m7-7H3' : 'M10 19l-7-7m0 0l7-7m-7 7h18'}
        />
      </svg>
      {outgoing ? 'Sent' : 'Received'}
    </span>
  );
}

interface AccountTransactionsTableProps {
  transactions: AccountTransaction[];
  isLoading?: boolean;
}

export function AccountTransactionsTable({
  transactions,
  isLoading,
}: AccountTransactionsTableProps) {
  const columns: Column<AccountTransaction>[] = [
    {
      key: 'role',
      header: 'Role',
      render: (tx) => <RoleCell role={tx.role} />,
    },
    ...(transactionColumns() as Column<AccountTransaction>[]),
  ];

  return (
    <DataTable
      columns={columns}
      data={transactions}
      keyExtractor={(tx) => `${tx.hash}-${tx.role}`}
      isLoading={isLoading}
      emptyMessage="No transactions for this account"
    />
  );
}
