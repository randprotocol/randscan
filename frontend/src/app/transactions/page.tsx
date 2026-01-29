'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useTransactions } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { Pagination } from '@/components/Pagination';
import {
  formatNumber,
  formatTimestamp,
  formatLamports,
  shortenHash,
  getStatusColorClass,
  getTransactionTypeLabel,
  cn,
} from '@/lib/utils';
import type { TransactionSummary, TransactionStatus, TransactionType } from '@/types';

const statusFilters: { value: TransactionStatus | 'all'; label: string }[] = [
  { value: 'all', label: 'All Status' },
  { value: 'success', label: 'Success' },
  { value: 'failed', label: 'Failed' },
  { value: 'pending', label: 'Pending' },
];

const typeFilters: { value: TransactionType | 'all'; label: string }[] = [
  { value: 'all', label: 'All Types' },
  { value: 'transfer', label: 'Transfer' },
  { value: 'stake', label: 'Stake' },
  { value: 'vote', label: 'Vote' },
  { value: 'program', label: 'Program' },
  { value: 'system', label: 'System' },
];

export default function TransactionsPage() {
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState<TransactionStatus | 'all'>('all');
  const [typeFilter, setTypeFilter] = useState<TransactionType | 'all'>('all');
  const pageSize = 20;

  const filters = {
    status: statusFilter !== 'all' ? statusFilter : undefined,
    type: typeFilter !== 'all' ? typeFilter : undefined,
  };

  const { data: txResponse, isLoading } = useTransactions(page, pageSize, filters);

  const columns = [
    {
      key: 'signature',
      header: 'Signature',
      render: (tx: TransactionSummary) => (
        <div className="flex items-center gap-2">
          <Link href={`/transactions/${tx.signature}`} className="link font-mono">
            {shortenHash(tx.signature, 8, 8)}
          </Link>
          {tx.is_private && (
            <span className="badge badge-info" title="Private Transaction">
              <svg className="mr-1 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              Private
            </span>
          )}
        </div>
      ),
    },
    {
      key: 'slot',
      header: 'Block',
      render: (tx: TransactionSummary) => (
        <Link href={`/blocks/${tx.slot}`} className="link">
          {formatNumber(tx.slot)}
        </Link>
      ),
    },
    {
      key: 'timestamp',
      header: 'Age',
      render: (tx: TransactionSummary) => (
        <span className="text-dark-300">{formatTimestamp(tx.timestamp)}</span>
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
        <div className="flex items-center gap-1.5">
          <span className={`status-dot ${tx.status === 'success' ? 'status-dot-success' : tx.status === 'failed' ? 'status-dot-error' : 'status-dot-pending'}`} />
          <span className={getStatusColorClass(tx.status)}>
            {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
          </span>
        </div>
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

  const handleStatusChange = (value: TransactionStatus | 'all') => {
    setStatusFilter(value);
    setPage(1);
  };

  const handleTypeChange = (value: TransactionType | 'all') => {
    setTypeFilter(value);
    setPage(1);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Transactions</h1>
        <p className="mt-1 text-dark-400">
          Browse all transactions on the Rand Protocol network
        </p>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4">
        {/* Status Filter */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-dark-400">Status:</span>
          <div className="flex gap-1">
            {statusFilters.map((filter) => (
              <button
                key={filter.value}
                type="button"
                onClick={() => handleStatusChange(filter.value)}
                className={cn(
                  'rounded-lg px-3 py-1.5 text-sm font-medium transition-colors',
                  statusFilter === filter.value
                    ? 'bg-primary-600 text-white'
                    : 'bg-dark-800 text-dark-300 hover:bg-dark-700 hover:text-white'
                )}
              >
                {filter.label}
              </button>
            ))}
          </div>
        </div>

        {/* Type Filter */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-dark-400">Type:</span>
          <select
            value={typeFilter}
            onChange={(e) => handleTypeChange(e.target.value as TransactionType | 'all')}
            className="rounded-lg border border-dark-600 bg-dark-800 px-3 py-1.5 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            {typeFilters.map((filter) => (
              <option key={filter.value} value={filter.value}>
                {filter.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Stats */}
      {txResponse && (
        <div className="flex items-center gap-4 text-sm text-dark-400">
          <span>
            Total: <span className="text-white">{formatNumber(txResponse.total)}</span> transactions
          </span>
          <span>|</span>
          <span>
            Page <span className="text-white">{txResponse.page}</span> of{' '}
            <span className="text-white">{txResponse.total_pages}</span>
          </span>
        </div>
      )}

      {/* Transactions Table */}
      <DataTable
        columns={columns}
        data={txResponse?.data || []}
        keyExtractor={(tx) => tx.signature}
        onRowClick={(tx) => window.location.href = `/transactions/${tx.signature}`}
        isLoading={isLoading}
        emptyMessage="No transactions found"
      />

      {/* Pagination */}
      {txResponse && txResponse.total_pages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={txResponse.total_pages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
