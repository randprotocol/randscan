'use client';

import { use } from 'react';
import Link from 'next/link';
import { useTransaction } from '@/hooks/useApi';
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
  cn,
} from '@/lib/utils';

interface TransactionDetailPageProps {
  params: Promise<{ id: string }>;
}

export default function TransactionDetailPage({ params }: TransactionDetailPageProps) {
  const { id } = use(params);
  const { data: tx, isLoading, error } = useTransaction(id);

  if (isLoading) {
    return <DetailSkeleton />;
  }

  if (error || !tx) {
    return (
      <div className="flex h-96 flex-col items-center justify-center">
        <h2 className="text-xl font-semibold text-white">Transaction Not Found</h2>
        <p className="mt-2 text-dark-400">
          The transaction with signature &quot;{shortenHash(id, 8, 8)}&quot; could not be found.
        </p>
        <Link href="/transactions" className="mt-4 link">
          Back to Transactions
        </Link>
      </div>
    );
  }

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
          <Link href="/transactions" className="text-dark-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <h1 className="text-2xl font-bold text-white">Transaction Details</h1>
        </div>
        <div className="mt-2 flex items-center gap-2">
          <p className="font-mono text-sm text-dark-400">{shortenHash(tx.signature, 16, 16)}</p>
          <button
            type="button"
            onClick={() => copyToClipboard(tx.signature)}
            className="text-dark-400 hover:text-white"
          >
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Status</p>
          <div className="mt-1 flex items-center gap-2">
            <span className={`status-dot ${tx.status === 'success' ? 'status-dot-success' : tx.status === 'failed' ? 'status-dot-error' : 'status-dot-pending'}`} />
            <span className={cn('text-xl font-semibold', getStatusColorClass(tx.status))}>
              {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
            </span>
          </div>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Type</p>
          <p className="mt-1 text-xl font-semibold text-white">{getTransactionTypeLabel(tx.type)}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Block</p>
          <Link href={`/blocks/${tx.slot}`} className="mt-1 block text-xl font-semibold text-primary-400 hover:text-primary-300">
            {formatNumber(tx.slot)}
          </Link>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Fee</p>
          <p className="mt-1 text-xl font-semibold text-white">{formatLamports(tx.fee)} ATLAS</p>
        </div>
      </div>

      {/* Privacy Indicator */}
      {tx.is_private && (
        <div className="rounded-xl border border-primary-600/30 bg-primary-900/20 p-4">
          <div className="flex items-center gap-3">
            <svg className="h-6 w-6 text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <div>
              <p className="font-semibold text-white">Private Transaction</p>
              <p className="text-sm text-dark-300">
                Privacy Level: <span className="text-primary-400">{tx.privacy_level || 'shielded'}</span>
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Error Display */}
      {tx.error && (
        <div className="rounded-xl border border-red-600/30 bg-red-900/20 p-4">
          <div className="flex items-start gap-3">
            <svg className="mt-0.5 h-5 w-5 flex-shrink-0 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <p className="font-semibold text-red-400">Transaction Failed</p>
              <p className="mt-1 font-mono text-sm text-red-300">{tx.error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Transaction Details */}
      <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
        <h2 className="mb-4 text-lg font-semibold text-white">Overview</h2>

        <InfoRow label="Signature" value={shortenHash(tx.signature, 20, 20)} copyable={tx.signature} mono />
        <InfoRow label="Block" value={
          <Link href={`/blocks/${tx.slot}`} className="link">{formatNumber(tx.slot)}</Link>
        } />
        <InfoRow label="Timestamp" value={formatDateTime(tx.block_time)} />
        <InfoRow label="Recent Blockhash" value={shortenHash(tx.recent_blockhash, 12, 12)} copyable={tx.recent_blockhash} mono />
        <InfoRow label="Fee" value={`${formatLamports(tx.fee)} ATLAS`} />
        {tx.compute_units_consumed !== undefined && (
          <InfoRow label="Compute Units" value={formatNumber(tx.compute_units_consumed)} />
        )}

        <div className="my-4 border-t border-dark-600" />
        <h3 className="mb-3 text-md font-semibold text-white">Signers</h3>
        {tx.signers.map((signer, index) => (
          <InfoRow
            key={signer}
            label={index === 0 ? 'Fee Payer' : `Signer ${index + 1}`}
            value={
              <Link href={`/account/${signer}`} className="link">
                {shortenHash(signer, 8, 8)}
              </Link>
            }
            copyable={signer}
            mono
          />
        ))}
      </div>

      {/* Instructions */}
      {tx.instructions && tx.instructions.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">
            Instructions ({tx.instructions.length})
          </h2>
          <div className="space-y-4">
            {tx.instructions.map((ix, index) => (
              <div key={index} className="rounded-lg border border-dark-600 bg-dark-900 p-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium text-dark-400">#{index + 1}</span>
                  {ix.program_name && (
                    <span className="badge badge-info">{ix.program_name}</span>
                  )}
                </div>
                <div className="mt-2">
                  <p className="text-xs text-dark-400">Program</p>
                  <Link href={`/account/${ix.program_id}`} className="link font-mono text-sm">
                    {shortenHash(ix.program_id, 8, 8)}
                  </Link>
                </div>
                {ix.accounts.length > 0 && (
                  <div className="mt-3">
                    <p className="text-xs text-dark-400">Accounts ({ix.accounts.length})</p>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {ix.accounts.slice(0, 5).map((account, accIndex) => (
                        <Link
                          key={accIndex}
                          href={`/account/${account}`}
                          className="link font-mono text-xs"
                        >
                          {shortenHash(account, 4, 4)}
                        </Link>
                      ))}
                      {ix.accounts.length > 5 && (
                        <span className="text-xs text-dark-400">
                          +{ix.accounts.length - 5} more
                        </span>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Log Messages */}
      {tx.log_messages && tx.log_messages.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">Program Logs</h2>
          <div className="max-h-64 overflow-y-auto rounded-lg bg-dark-900 p-4">
            <pre className="font-mono text-xs text-dark-300">
              {tx.log_messages.map((log, index) => (
                <div
                  key={index}
                  className={cn(
                    'py-0.5',
                    log.includes('Error') || log.includes('failed') ? 'text-red-400' : '',
                    log.includes('success') ? 'text-green-400' : ''
                  )}
                >
                  {log}
                </div>
              ))}
            </pre>
          </div>
        </div>
      )}

      {/* Balance Changes */}
      {tx.pre_balances && tx.pre_balances.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">Balance Changes</h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-600 text-left text-sm text-dark-400">
                  <th className="pb-2">Account</th>
                  <th className="pb-2 text-right">Before</th>
                  <th className="pb-2 text-right">After</th>
                  <th className="pb-2 text-right">Change</th>
                </tr>
              </thead>
              <tbody>
                {tx.signers.slice(0, tx.pre_balances.length).map((account, index) => {
                  const pre = BigInt(tx.pre_balances[index] || '0');
                  const post = BigInt(tx.post_balances[index] || '0');
                  const change = post - pre;

                  return (
                    <tr key={index} className="border-b border-dark-700 text-sm">
                      <td className="py-2">
                        <Link href={`/account/${account}`} className="link font-mono">
                          {shortenHash(account, 6, 6)}
                        </Link>
                      </td>
                      <td className="py-2 text-right font-mono text-dark-300">
                        {formatLamports(pre.toString())}
                      </td>
                      <td className="py-2 text-right font-mono text-dark-300">
                        {formatLamports(post.toString())}
                      </td>
                      <td className={cn(
                        'py-2 text-right font-mono',
                        change > 0n ? 'text-green-400' : change < 0n ? 'text-red-400' : 'text-dark-400'
                      )}>
                        {change > 0n ? '+' : ''}{formatLamports(change.toString())}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
