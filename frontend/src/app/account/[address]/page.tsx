'use client';

import { use, useState } from 'react';
import Link from 'next/link';
import { useAccount, useAccountTransactions } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { Pagination } from '@/components/Pagination';
import { DetailSkeleton } from '@/components/Loading';
import {
  formatNumber,
  formatLamports,
  formatTimestamp,
  shortenHash,
  copyToClipboard,
  getStatusColorClass,
  getTransactionTypeLabel,
  cn,
} from '@/lib/utils';
import type { AccountTransaction, TokenAccount, StakeAccount } from '@/types';

interface AccountPageProps {
  params: Promise<{ address: string }>;
}

export default function AccountPage({ params }: AccountPageProps) {
  const { address } = use(params);
  const [txPage, setTxPage] = useState(1);

  const { data: account, isLoading, error } = useAccount(address);
  const { data: txResponse, isLoading: txLoading } = useAccountTransactions(address, txPage, 10);

  if (isLoading) {
    return <DetailSkeleton />;
  }

  if (error || !account) {
    return (
      <div className="flex h-96 flex-col items-center justify-center">
        <h2 className="text-xl font-semibold text-white">Account Not Found</h2>
        <p className="mt-2 text-dark-400">
          The account with address &quot;{shortenHash(address, 8, 8)}&quot; could not be found.
        </p>
        <Link href="/" className="mt-4 link">
          Back to Dashboard
        </Link>
      </div>
    );
  }

  const txColumns = [
    {
      key: 'signature',
      header: 'Signature',
      render: (tx: AccountTransaction) => (
        <Link href={`/transactions/${tx.signature}`} className="link font-mono">
          {shortenHash(tx.signature, 8, 8)}
        </Link>
      ),
    },
    {
      key: 'timestamp',
      header: 'Age',
      render: (tx: AccountTransaction) => (
        <span className="text-dark-300">{formatTimestamp(tx.timestamp)}</span>
      ),
    },
    {
      key: 'type',
      header: 'Type',
      render: (tx: AccountTransaction) => (
        <span className="badge badge-neutral">{getTransactionTypeLabel(tx.type)}</span>
      ),
    },
    {
      key: 'direction',
      header: 'Direction',
      render: (tx: AccountTransaction) => (
        <span className={cn(
          'badge',
          tx.direction === 'in' ? 'badge-success' : tx.direction === 'out' ? 'badge-error' : 'badge-neutral'
        )}>
          {tx.direction === 'in' ? 'IN' : tx.direction === 'out' ? 'OUT' : 'SELF'}
        </span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (tx: AccountTransaction) => (
        <span className={getStatusColorClass(tx.status)}>
          {tx.status.charAt(0).toUpperCase() + tx.status.slice(1)}
        </span>
      ),
    },
    {
      key: 'amount',
      header: 'Amount',
      className: 'text-right',
      render: (tx: AccountTransaction) => (
        <span className={cn(
          'font-mono',
          tx.direction === 'in' ? 'text-green-400' : tx.direction === 'out' ? 'text-red-400' : 'text-dark-300'
        )}>
          {tx.amount ? `${tx.direction === 'in' ? '+' : '-'}${formatLamports(tx.amount)}` : '-'}
        </span>
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
          <Link href="/" className="text-dark-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <h1 className="text-2xl font-bold text-white">Account</h1>
          {account.is_validator && (
            <span className="badge badge-info">Validator</span>
          )}
        </div>
        <div className="mt-2 flex items-center gap-2">
          <p className="font-mono text-sm text-dark-400 break-all">{address}</p>
          <button
            type="button"
            onClick={() => copyToClipboard(address)}
            className="flex-shrink-0 text-dark-400 hover:text-white"
          >
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
      </div>

      {/* Balance Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">ATLAS Balance</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatLamports(account.atlas_balance)}</p>
          <p className="mt-1 text-xs text-dark-400">ATLAS</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">SHRUG Balance</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatLamports(account.shrug_balance)}</p>
          <p className="mt-1 text-xs text-dark-400">SHRUG</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Token Accounts</p>
          <p className="mt-1 text-2xl font-bold text-white">{account.token_accounts.length}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Data Size</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatNumber(account.data_size)}</p>
          <p className="mt-1 text-xs text-dark-400">bytes</p>
        </div>
      </div>

      {/* Account Details */}
      <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
        <h2 className="mb-4 text-lg font-semibold text-white">Account Details</h2>

        <InfoRow label="Address" value={shortenHash(account.address, 16, 16)} copyable={account.address} mono />
        <InfoRow label="Owner" value={
          <Link href={`/account/${account.owner}`} className="link">
            {shortenHash(account.owner, 8, 8)}
          </Link>
        } />
        <InfoRow label="Lamports" value={formatNumber(Number(account.lamports))} />
        <InfoRow label="Executable" value={account.executable ? 'Yes' : 'No'} />
        <InfoRow label="Rent Epoch" value={formatNumber(account.rent_epoch)} />

        {account.is_validator && account.validator_identity && (
          <>
            <div className="my-4 border-t border-dark-600" />
            <h3 className="mb-3 text-md font-semibold text-white">Validator Info</h3>
            <InfoRow label="Identity" value={
              <Link href={`/validators/${account.validator_identity}`} className="link">
                {shortenHash(account.validator_identity, 8, 8)}
              </Link>
            } />
          </>
        )}
      </div>

      {/* Token Accounts */}
      {account.token_accounts.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">
            Token Accounts ({account.token_accounts.length})
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-600 text-left text-sm text-dark-400">
                  <th className="pb-2">Token</th>
                  <th className="pb-2">Mint</th>
                  <th className="pb-2 text-right">Balance</th>
                </tr>
              </thead>
              <tbody>
                {account.token_accounts.map((token: TokenAccount) => (
                  <tr key={token.address} className="border-b border-dark-700 text-sm">
                    <td className="py-2">
                      <span className="font-medium text-white">
                        {token.mint_symbol || token.mint_name || 'Unknown'}
                      </span>
                    </td>
                    <td className="py-2">
                      <Link href={`/tokens/${token.mint}`} className="link font-mono">
                        {shortenHash(token.mint, 6, 6)}
                      </Link>
                    </td>
                    <td className="py-2 text-right font-mono text-white">
                      {token.ui_balance.toLocaleString(undefined, { maximumFractionDigits: 4 })}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Stake Accounts */}
      {account.stake_accounts && account.stake_accounts.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">
            Stake Accounts ({account.stake_accounts.length})
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-600 text-left text-sm text-dark-400">
                  <th className="pb-2">Address</th>
                  <th className="pb-2">Validator</th>
                  <th className="pb-2">Status</th>
                  <th className="pb-2 text-right">Stake</th>
                </tr>
              </thead>
              <tbody>
                {account.stake_accounts.map((stake: StakeAccount) => (
                  <tr key={stake.address} className="border-b border-dark-700 text-sm">
                    <td className="py-2">
                      <Link href={`/account/${stake.address}`} className="link font-mono">
                        {shortenHash(stake.address, 6, 6)}
                      </Link>
                    </td>
                    <td className="py-2">
                      <Link href={`/validators/${stake.voter}`} className="link font-mono">
                        {shortenHash(stake.voter, 6, 6)}
                      </Link>
                    </td>
                    <td className="py-2">
                      <span className={cn(
                        'badge',
                        stake.status === 'active' ? 'badge-success' :
                        stake.status === 'activating' ? 'badge-warning' :
                        stake.status === 'deactivating' ? 'badge-error' :
                        'badge-neutral'
                      )}>
                        {stake.status}
                      </span>
                    </td>
                    <td className="py-2 text-right font-mono text-white">
                      {formatLamports(stake.stake)} ATLAS
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Transaction History */}
      <div>
        <h2 className="mb-4 text-lg font-semibold text-white">Transaction History</h2>
        <DataTable
          columns={txColumns}
          data={txResponse?.data || []}
          keyExtractor={(tx) => tx.signature}
          onRowClick={(tx) => window.location.href = `/transactions/${tx.signature}`}
          isLoading={txLoading}
          emptyMessage="No transactions found"
        />

        {txResponse && txResponse.total_pages > 1 && (
          <Pagination
            currentPage={txPage}
            totalPages={txResponse.total_pages}
            onPageChange={setTxPage}
            className="mt-4"
          />
        )}
      </div>
    </div>
  );
}
