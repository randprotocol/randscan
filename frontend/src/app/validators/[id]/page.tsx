'use client';

import { use } from 'react';
import Link from 'next/link';
import { useValidator } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { DetailSkeleton } from '@/components/Loading';
import {
  formatNumber,
  formatLamports,
  formatPercentage,
  formatTimestamp,
  shortenHash,
  copyToClipboard,
  getValidatorStatusColorClass,
  cn,
} from '@/lib/utils';
import type { BlockSummary } from '@/types';

interface ValidatorDetailPageProps {
  params: Promise<{ id: string }>;
}

export default function ValidatorDetailPage({ params }: ValidatorDetailPageProps) {
  const { id } = use(params);
  const { data: validator, isLoading, error } = useValidator(id);

  if (isLoading) {
    return <DetailSkeleton />;
  }

  if (error || !validator) {
    return (
      <div className="flex h-96 flex-col items-center justify-center">
        <h2 className="text-xl font-semibold text-white">Validator Not Found</h2>
        <p className="mt-2 text-dark-400">
          The validator with identity &quot;{shortenHash(id, 8, 8)}&quot; could not be found.
        </p>
        <Link href="/validators" className="mt-4 link">
          Back to Validators
        </Link>
      </div>
    );
  }

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
        <span className="text-white">{block.transaction_count}</span>
      ),
    },
    {
      key: 'rewards',
      header: 'Rewards',
      className: 'text-right',
      render: (block: BlockSummary) => (
        <span className="text-dark-300">{formatLamports(block.rewards)}</span>
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
        <span className={`text-sm text-white ${mono ? 'font-mono' : ''} break-all`}>{value}</span>
        {copyable && (
          <button
            type="button"
            onClick={() => copyToClipboard(copyable)}
            className="flex-shrink-0 text-dark-400 hover:text-white"
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
          <Link href="/validators" className="text-dark-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <div className="flex items-center gap-3">
            {validator.icon_url ? (
              <img
                src={validator.icon_url}
                alt=""
                className="h-12 w-12 rounded-full bg-dark-700"
              />
            ) : (
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary-600 text-xl font-bold text-white">
                {(validator.name || validator.identity)[0]?.toUpperCase()}
              </div>
            )}
            <div>
              <h1 className="text-2xl font-bold text-white">
                {validator.name || shortenHash(validator.identity, 8, 8)}
              </h1>
              <div className="flex items-center gap-2">
                <span className={`status-dot ${
                  validator.status === 'active' ? 'status-dot-success' :
                  validator.status === 'delinquent' ? 'status-dot-pending' :
                  'status-dot-inactive'
                }`} />
                <span className={cn('text-sm', getValidatorStatusColorClass(validator.status))}>
                  {validator.status.charAt(0).toUpperCase() + validator.status.slice(1)}
                </span>
              </div>
            </div>
          </div>
        </div>
        <p className="mt-2 font-mono text-sm text-dark-400">{validator.identity}</p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Total Stake</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatLamports(validator.stake)}</p>
          <p className="mt-1 text-xs text-dark-400">ATLAS</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Commission</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatPercentage(validator.commission, 0)}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Uptime</p>
          <p className={cn(
            'mt-1 text-2xl font-bold',
            validator.uptime_percentage >= 99 ? 'text-green-400' :
            validator.uptime_percentage >= 95 ? 'text-yellow-400' :
            'text-red-400'
          )}>
            {formatPercentage(validator.uptime_percentage, 2)}
          </p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Skip Rate</p>
          <p className={cn(
            'mt-1 text-2xl font-bold',
            validator.skip_rate <= 1 ? 'text-green-400' :
            validator.skip_rate <= 5 ? 'text-yellow-400' :
            'text-red-400'
          )}>
            {formatPercentage(validator.skip_rate, 2)}
          </p>
        </div>
      </div>

      {/* Description */}
      {validator.description && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <p className="text-dark-300">{validator.description}</p>
        </div>
      )}

      {/* Validator Details */}
      <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
        <h2 className="mb-4 text-lg font-semibold text-white">Validator Details</h2>

        <InfoRow label="Identity" value={shortenHash(validator.identity, 16, 16)} copyable={validator.identity} mono />
        <InfoRow label="Vote Account" value={
          <Link href={`/account/${validator.vote_account}`} className="link">
            {shortenHash(validator.vote_account, 8, 8)}
          </Link>
        } copyable={validator.vote_account} />
        <InfoRow label="Activated Stake" value={`${formatLamports(validator.activated_stake)} ATLAS`} />
        <InfoRow label="Last Vote" value={
          <Link href={`/blocks/${validator.last_vote}`} className="link">{formatNumber(validator.last_vote)}</Link>
        } />
        <InfoRow label="Root Slot" value={formatNumber(validator.root_slot)} />
        <InfoRow label="Credits (Epoch)" value={formatNumber(validator.epoch_credits)} />
        <InfoRow label="Credits (Total)" value={formatNumber(validator.credits)} />
        <InfoRow label="Epoch Vote Account" value={validator.epoch_vote_account ? 'Yes' : 'No'} />
        {validator.version && <InfoRow label="Version" value={validator.version} />}
        {validator.website && (
          <InfoRow label="Website" value={
            <a href={validator.website} target="_blank" rel="noopener noreferrer" className="link">
              {validator.website}
            </a>
          } />
        )}
        {validator.keybase_username && (
          <InfoRow label="Keybase" value={
            <a
              href={`https://keybase.io/${validator.keybase_username}`}
              target="_blank"
              rel="noopener noreferrer"
              className="link"
            >
              @{validator.keybase_username}
            </a>
          } />
        )}
      </div>

      {/* Recent Blocks */}
      {validator.recent_blocks && validator.recent_blocks.length > 0 && (
        <div>
          <h2 className="mb-4 text-lg font-semibold text-white">
            Recent Blocks ({validator.recent_blocks.length})
          </h2>
          <DataTable
            columns={blockColumns}
            data={validator.recent_blocks}
            keyExtractor={(block) => block.slot.toString()}
            onRowClick={(block) => window.location.href = `/blocks/${block.slot}`}
          />
        </div>
      )}

      {/* Stake History Chart Placeholder */}
      {validator.stake_history && validator.stake_history.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">Stake History</h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-600 text-left text-sm text-dark-400">
                  <th className="pb-2">Epoch</th>
                  <th className="pb-2 text-right">Effective Stake</th>
                  <th className="pb-2 text-right">Activating</th>
                  <th className="pb-2 text-right">Deactivating</th>
                </tr>
              </thead>
              <tbody>
                {validator.stake_history.slice(0, 10).map((entry) => (
                  <tr key={entry.epoch} className="border-b border-dark-700 text-sm">
                    <td className="py-2 text-white">{formatNumber(entry.epoch)}</td>
                    <td className="py-2 text-right font-mono text-white">
                      {formatLamports(entry.effective_stake)}
                    </td>
                    <td className="py-2 text-right font-mono text-green-400">
                      +{formatLamports(entry.activating_stake)}
                    </td>
                    <td className="py-2 text-right font-mono text-red-400">
                      -{formatLamports(entry.deactivating_stake)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
