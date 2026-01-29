'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useValidators } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { Pagination } from '@/components/Pagination';
import {
  formatNumber,
  formatLamports,
  formatPercentage,
  shortenHash,
  getValidatorStatusColorClass,
  cn,
} from '@/lib/utils';
import type { Validator } from '@/types';

type ValidatorStatus = 'active' | 'delinquent' | 'inactive';

const statusFilters: { value: ValidatorStatus | 'all'; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'delinquent', label: 'Delinquent' },
  { value: 'inactive', label: 'Inactive' },
];

export default function ValidatorsPage() {
  const [page, setPage] = useState(1);
  const [statusFilter, setStatusFilter] = useState<ValidatorStatus | 'all'>('all');
  const pageSize = 20;

  const { data: validatorsResponse, isLoading } = useValidators(
    page,
    pageSize,
    statusFilter !== 'all' ? statusFilter : undefined
  );

  const columns = [
    {
      key: 'rank',
      header: '#',
      className: 'w-12',
      render: (_: Validator, index: number) => (
        <span className="text-dark-400">{(page - 1) * pageSize + index + 1}</span>
      ),
    },
    {
      key: 'identity',
      header: 'Validator',
      render: (validator: Validator) => (
        <div className="flex items-center gap-3">
          {validator.icon_url ? (
            <img
              src={validator.icon_url}
              alt=""
              className="h-8 w-8 rounded-full bg-dark-700"
            />
          ) : (
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-dark-700 text-sm font-medium text-dark-300">
              {(validator.name || validator.identity)[0]?.toUpperCase()}
            </div>
          )}
          <div>
            <Link href={`/validators/${validator.identity}`} className="link font-medium">
              {validator.name || shortenHash(validator.identity, 6, 6)}
            </Link>
            <p className="font-mono text-xs text-dark-400">{shortenHash(validator.identity, 4, 4)}</p>
          </div>
        </div>
      ),
    },
    {
      key: 'stake',
      header: 'Stake',
      className: 'text-right',
      render: (validator: Validator) => (
        <span className="text-white">{formatLamports(validator.stake)} ATLAS</span>
      ),
    },
    {
      key: 'commission',
      header: 'Commission',
      className: 'text-right',
      render: (validator: Validator) => (
        <span className="text-dark-300">{formatPercentage(validator.commission, 0)}</span>
      ),
    },
    {
      key: 'uptime',
      header: 'Uptime',
      className: 'text-right',
      render: (validator: Validator) => (
        <span className={cn(
          validator.uptime_percentage >= 99 ? 'text-green-400' :
          validator.uptime_percentage >= 95 ? 'text-yellow-400' :
          'text-red-400'
        )}>
          {formatPercentage(validator.uptime_percentage, 2)}
        </span>
      ),
    },
    {
      key: 'skip_rate',
      header: 'Skip Rate',
      className: 'text-right',
      render: (validator: Validator) => (
        <span className={cn(
          validator.skip_rate <= 1 ? 'text-green-400' :
          validator.skip_rate <= 5 ? 'text-yellow-400' :
          'text-red-400'
        )}>
          {formatPercentage(validator.skip_rate, 2)}
        </span>
      ),
    },
    {
      key: 'last_vote',
      header: 'Last Vote',
      className: 'text-right',
      render: (validator: Validator) => (
        <Link href={`/blocks/${validator.last_vote}`} className="link">
          {formatNumber(validator.last_vote)}
        </Link>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (validator: Validator) => (
        <div className="flex items-center gap-1.5">
          <span className={`status-dot ${
            validator.status === 'active' ? 'status-dot-success' :
            validator.status === 'delinquent' ? 'status-dot-pending' :
            'status-dot-inactive'
          }`} />
          <span className={getValidatorStatusColorClass(validator.status)}>
            {validator.status.charAt(0).toUpperCase() + validator.status.slice(1)}
          </span>
        </div>
      ),
    },
  ];

  const handleStatusChange = (value: ValidatorStatus | 'all') => {
    setStatusFilter(value);
    setPage(1);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Validators</h1>
        <p className="mt-1 text-dark-400">
          Browse all validators securing the Rand Protocol network
        </p>
      </div>

      {/* Filters */}
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

      {/* Stats */}
      {validatorsResponse && (
        <div className="flex items-center gap-4 text-sm text-dark-400">
          <span>
            Total: <span className="text-white">{formatNumber(validatorsResponse.total)}</span> validators
          </span>
          <span>|</span>
          <span>
            Page <span className="text-white">{validatorsResponse.page}</span> of{' '}
            <span className="text-white">{validatorsResponse.total_pages}</span>
          </span>
        </div>
      )}

      {/* Validators Table */}
      <DataTable
        columns={columns}
        data={validatorsResponse?.data || []}
        keyExtractor={(validator) => validator.identity}
        onRowClick={(validator) => window.location.href = `/validators/${validator.identity}`}
        isLoading={isLoading}
        emptyMessage="No validators found"
      />

      {/* Pagination */}
      {validatorsResponse && validatorsResponse.total_pages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={validatorsResponse.total_pages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
