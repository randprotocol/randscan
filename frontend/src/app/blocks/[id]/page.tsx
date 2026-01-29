'use client';

import { use } from 'react';
import Link from 'next/link';
import { useBlock } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { DetailSkeleton } from '@/components/Loading';
import {
  formatNumber,
  formatDateTime,
  formatTimestamp,
  formatLamports,
  shortenHash,
  copyToClipboard,
  getStatusColorClass,
  getTransactionTypeLabel,
} from '@/lib/utils';
import type { TransactionSummary } from '@/types';

interface BlockDetailPageProps {
  params: Promise<{ id: string }>;
}

export default function BlockDetailPage({ params }: BlockDetailPageProps) {
  const { id } = use(params);
  const { data: block, isLoading, error } = useBlock(id);

  if (isLoading) {
    return <DetailSkeleton />;
  }

  if (error || !block) {
    return (
      <div className="flex h-96 flex-col items-center justify-center">
        <h2 className="text-xl font-semibold text-white">Block Not Found</h2>
        <p className="mt-2 text-dark-400">
          The block with slot or hash &quot;{id}&quot; could not be found.
        </p>
        <Link href="/blocks" className="mt-4 link">
          Back to Blocks
        </Link>
      </div>
    );
  }

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
      key: 'signer',
      header: 'Signer',
      render: (tx: TransactionSummary) => (
        <Link href={`/account/${tx.signer}`} className="link font-mono">
          {shortenHash(tx.signer)}
        </Link>
      ),
    },
    {
      key: 'fee',
      header: 'Fee',
      className: 'text-right',
      render: (tx: TransactionSummary) => (
        <span className="text-dark-300">{formatLamports(tx.fee)}</span>
      ),
    },
  ];

  const InfoRow = ({
    label,
    value,
    copyable,
    mono,
  }: {
    label: string;
    value: React.ReactNode;
    copyable?: string;
    mono?: boolean;
  }) => (
    <div className="flex flex-col gap-1 border-b border-dark-700 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
      <span className="text-sm font-medium text-dark-400">{label}</span>
      <div className="flex items-center gap-2">
        <span className={`text-sm text-white ${mono ? 'font-mono' : ''}`}>{value}</span>
        {copyable && (
          <button
            type="button"
            onClick={() => copyToClipboard(copyable)}
            className="text-dark-400 hover:text-white"
          >
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <div className="flex items-center gap-3">
          <Link href="/blocks" className="text-dark-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <h1 className="text-2xl font-bold text-white">Block #{formatNumber(block.slot)}</h1>
        </div>
        <p className="mt-1 font-mono text-sm text-dark-400">{block.blockhash}</p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Slot</p>
          <p className="mt-1 text-xl font-semibold text-white">{formatNumber(block.slot)}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Transactions</p>
          <p className="mt-1 text-xl font-semibold text-white">{formatNumber(block.transaction_count)}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Rewards</p>
          <p className="mt-1 text-xl font-semibold text-white">{formatLamports(block.rewards)} ATLAS</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Timestamp</p>
          <p className="mt-1 text-lg font-semibold text-white">{formatTimestamp(block.timestamp)}</p>
        </div>
      </div>

      {/* Block Details */}
      <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
        <h2 className="mb-4 text-lg font-semibold text-white">Block Details</h2>

        <InfoRow label="Block Hash" value={shortenHash(block.blockhash, 16, 16)} copyable={block.blockhash} mono />
        <InfoRow label="Parent Slot" value={
          <Link href={`/blocks/${block.parent_slot}`} className="link">
            {formatNumber(block.parent_slot)}
          </Link>
        } />
        <InfoRow label="Parent Hash" value={shortenHash(block.parent_blockhash, 16, 16)} copyable={block.parent_blockhash} mono />
        <InfoRow label="Timestamp" value={formatDateTime(block.timestamp)} />
        <InfoRow label="Leader" value={
          <Link href={`/validators/${block.leader}`} className="link">
            {shortenHash(block.leader, 8, 8)}
          </Link>
        } />
        {block.leader_identity && (
          <InfoRow label="Leader Identity" value={block.leader_identity} />
        )}
        <InfoRow label="Rewards" value={`${formatLamports(block.rewards)} ATLAS`} />
        <InfoRow label="Fee Rewards" value={`${formatLamports(block.fee_rewards)} ATLAS`} />
        <InfoRow label="Previous Blockhash" value={shortenHash(block.previous_blockhash, 16, 16)} copyable={block.previous_blockhash} mono />

        {/* QC Info */}
        {block.qc_hash && (
          <>
            <div className="my-4 border-t border-dark-600" />
            <h3 className="mb-3 text-md font-semibold text-white">Quorum Certificate</h3>
            <InfoRow label="QC Hash" value={shortenHash(block.qc_hash, 16, 16)} copyable={block.qc_hash} mono />
            {block.qc_slot && (
              <InfoRow label="QC Slot" value={formatNumber(block.qc_slot)} />
            )}
          </>
        )}
      </div>

      {/* Transactions */}
      {block.transactions && block.transactions.length > 0 && (
        <div>
          <h2 className="mb-4 text-lg font-semibold text-white">
            Transactions ({formatNumber(block.transactions.length)})
          </h2>
          <DataTable
            columns={txColumns}
            data={block.transactions}
            keyExtractor={(tx) => tx.signature}
            onRowClick={(tx) => window.location.href = `/transactions/${tx.signature}`}
          />
        </div>
      )}

      {/* Navigation */}
      <div className="flex items-center justify-between pt-4">
        <Link
          href={`/blocks/${block.parent_slot}`}
          className="flex items-center gap-2 text-sm text-dark-400 hover:text-white"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
          Previous Block
        </Link>
        <Link
          href={`/blocks/${block.slot + 1}`}
          className="flex items-center gap-2 text-sm text-dark-400 hover:text-white"
        >
          Next Block
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </Link>
      </div>
    </div>
  );
}
