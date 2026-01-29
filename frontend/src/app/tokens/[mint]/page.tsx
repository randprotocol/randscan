'use client';

import { use, useState } from 'react';
import Link from 'next/link';
import { useToken } from '@/hooks/useApi';
import { getTokenSupply, getTokenHolders } from '@/lib/api';
import { DetailSkeleton } from '@/components/Loading';
import { Pagination } from '@/components/Pagination';
import {
  formatNumber,
  formatCompactNumber,
  formatPercentage,
  shortenHash,
  copyToClipboard,
} from '@/lib/utils';
import type { TokenSupply, TokenHolder, PaginatedResponse } from '@/types';
import useSWR from 'swr';

interface TokenDetailPageProps {
  params: Promise<{ mint: string }>;
}

export default function TokenDetailPage({ params }: TokenDetailPageProps) {
  const { mint } = use(params);
  const [holdersPage, setHoldersPage] = useState(1);

  const { data: token, isLoading, error } = useToken(mint);
  const { data: supply } = useSWR<TokenSupply>(
    token ? ['tokenSupply', mint] : null,
    () => getTokenSupply(mint)
  );
  const { data: holdersResponse } = useSWR<PaginatedResponse<TokenHolder>>(
    token ? ['tokenHolders', mint, holdersPage] : null,
    () => getTokenHolders(mint, holdersPage, 10)
  );

  if (isLoading) {
    return <DetailSkeleton />;
  }

  if (error || !token) {
    return (
      <div className="flex h-96 flex-col items-center justify-center">
        <h2 className="text-xl font-semibold text-white">Token Not Found</h2>
        <p className="mt-2 text-dark-400">
          The token with mint &quot;{shortenHash(mint, 8, 8)}&quot; could not be found.
        </p>
        <Link href="/tokens" className="mt-4 link">
          Back to Tokens
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
          <Link href="/tokens" className="text-dark-400 hover:text-white">
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
          </Link>
          <div className="flex items-center gap-3">
            {token.icon_url ? (
              <img
                src={token.icon_url}
                alt=""
                className="h-12 w-12 rounded-full bg-dark-700"
              />
            ) : (
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary-600 text-xl font-bold text-white">
                {(token.symbol || token.name || token.mint)[0]?.toUpperCase()}
              </div>
            )}
            <div>
              <h1 className="text-2xl font-bold text-white">
                {token.name || 'Unknown Token'}
              </h1>
              {token.symbol && (
                <p className="text-dark-400">{token.symbol}</p>
              )}
            </div>
          </div>
        </div>
        <p className="mt-2 font-mono text-sm text-dark-400">{token.mint}</p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Total Supply</p>
          <p className="mt-1 text-2xl font-bold text-white">
            {formatCompactNumber(token.ui_supply)}
          </p>
          {token.symbol && <p className="mt-1 text-xs text-dark-400">{token.symbol}</p>}
        </div>
        {supply && (
          <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
            <p className="text-sm text-dark-400">Circulating Supply</p>
            <p className="mt-1 text-2xl font-bold text-white">
              {formatCompactNumber(supply.ui_circulating_supply)}
            </p>
            {token.symbol && <p className="mt-1 text-xs text-dark-400">{token.symbol}</p>}
          </div>
        )}
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Decimals</p>
          <p className="mt-1 text-2xl font-bold text-white">{token.decimals}</p>
        </div>
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-4">
          <p className="text-sm text-dark-400">Holders</p>
          <p className="mt-1 text-2xl font-bold text-white">{formatNumber(token.holder_count)}</p>
        </div>
      </div>

      {/* Token Details */}
      <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
        <h2 className="mb-4 text-lg font-semibold text-white">Token Details</h2>

        <InfoRow label="Mint Address" value={shortenHash(token.mint, 16, 16)} copyable={token.mint} mono />
        {token.name && <InfoRow label="Name" value={token.name} />}
        {token.symbol && <InfoRow label="Symbol" value={token.symbol} />}
        <InfoRow label="Decimals" value={token.decimals} />
        <InfoRow label="Total Supply" value={`${formatNumber(token.ui_supply)} ${token.symbol || ''}`} />
        <InfoRow label="Initialized" value={token.is_initialized ? 'Yes' : 'No'} />

        {token.mint_authority && (
          <InfoRow label="Mint Authority" value={
            <Link href={`/account/${token.mint_authority}`} className="link">
              {shortenHash(token.mint_authority, 8, 8)}
            </Link>
          } copyable={token.mint_authority} />
        )}
        {token.freeze_authority && (
          <InfoRow label="Freeze Authority" value={
            <Link href={`/account/${token.freeze_authority}`} className="link">
              {shortenHash(token.freeze_authority, 8, 8)}
            </Link>
          } copyable={token.freeze_authority} />
        )}
      </div>

      {/* Top Holders */}
      {holdersResponse && holdersResponse.data.length > 0 && (
        <div className="rounded-xl border border-dark-700 bg-dark-800 p-6">
          <h2 className="mb-4 text-lg font-semibold text-white">
            Top Holders ({formatNumber(holdersResponse.total)})
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-dark-600 text-left text-sm text-dark-400">
                  <th className="pb-2 w-12">#</th>
                  <th className="pb-2">Address</th>
                  <th className="pb-2 text-right">Balance</th>
                  <th className="pb-2 text-right">% of Supply</th>
                </tr>
              </thead>
              <tbody>
                {holdersResponse.data.map((holder: TokenHolder, index: number) => (
                  <tr key={holder.address} className="border-b border-dark-700 text-sm">
                    <td className="py-2 text-dark-400">
                      {(holdersPage - 1) * 10 + index + 1}
                    </td>
                    <td className="py-2">
                      <Link href={`/account/${holder.address}`} className="link font-mono">
                        {shortenHash(holder.address, 8, 8)}
                      </Link>
                    </td>
                    <td className="py-2 text-right font-mono text-white">
                      {formatNumber(holder.ui_balance)}
                    </td>
                    <td className="py-2 text-right text-dark-300">
                      {formatPercentage(holder.percentage, 2)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {holdersResponse.total_pages > 1 && (
            <Pagination
              currentPage={holdersPage}
              totalPages={holdersResponse.total_pages}
              onPageChange={setHoldersPage}
              className="mt-4"
            />
          )}
        </div>
      )}
    </div>
  );
}
