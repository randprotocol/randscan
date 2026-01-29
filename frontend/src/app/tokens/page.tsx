'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useTokens } from '@/hooks/useApi';
import { DataTable } from '@/components/DataTable';
import { Pagination } from '@/components/Pagination';
import { formatNumber, formatCompactNumber, shortenHash } from '@/lib/utils';
import type { TokenMint } from '@/types';

export default function TokensPage() {
  const [page, setPage] = useState(1);
  const pageSize = 20;

  const { data: tokensResponse, isLoading } = useTokens(page, pageSize);

  const columns = [
    {
      key: 'rank',
      header: '#',
      className: 'w-12',
      render: (_: TokenMint, index: number) => (
        <span className="text-dark-400">{(page - 1) * pageSize + index + 1}</span>
      ),
    },
    {
      key: 'token',
      header: 'Token',
      render: (token: TokenMint) => (
        <div className="flex items-center gap-3">
          {token.icon_url ? (
            <img
              src={token.icon_url}
              alt=""
              className="h-8 w-8 rounded-full bg-dark-700"
            />
          ) : (
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-dark-700 text-sm font-medium text-dark-300">
              {(token.symbol || token.name || token.mint)[0]?.toUpperCase()}
            </div>
          )}
          <div>
            <Link href={`/tokens/${token.mint}`} className="link font-medium">
              {token.name || shortenHash(token.mint, 6, 6)}
            </Link>
            {token.symbol && (
              <p className="text-xs text-dark-400">{token.symbol}</p>
            )}
          </div>
        </div>
      ),
    },
    {
      key: 'mint',
      header: 'Mint Address',
      render: (token: TokenMint) => (
        <Link href={`/tokens/${token.mint}`} className="link font-mono text-sm">
          {shortenHash(token.mint, 6, 6)}
        </Link>
      ),
    },
    {
      key: 'decimals',
      header: 'Decimals',
      className: 'text-center',
      render: (token: TokenMint) => (
        <span className="text-dark-300">{token.decimals}</span>
      ),
    },
    {
      key: 'supply',
      header: 'Total Supply',
      className: 'text-right',
      render: (token: TokenMint) => (
        <span className="text-white">
          {formatCompactNumber(token.ui_supply)}
        </span>
      ),
    },
    {
      key: 'holders',
      header: 'Holders',
      className: 'text-right',
      render: (token: TokenMint) => (
        <span className="text-dark-300">{formatNumber(token.holder_count)}</span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Tokens</h1>
        <p className="mt-1 text-dark-400">
          Browse all tokens on the Rand Protocol network
        </p>
      </div>

      {/* Stats */}
      {tokensResponse && (
        <div className="flex items-center gap-4 text-sm text-dark-400">
          <span>
            Total: <span className="text-white">{formatNumber(tokensResponse.total)}</span> tokens
          </span>
          <span>|</span>
          <span>
            Page <span className="text-white">{tokensResponse.page}</span> of{' '}
            <span className="text-white">{tokensResponse.total_pages}</span>
          </span>
        </div>
      )}

      {/* Tokens Table */}
      <DataTable
        columns={columns}
        data={tokensResponse?.data || []}
        keyExtractor={(token) => token.mint}
        onRowClick={(token) => window.location.href = `/tokens/${token.mint}`}
        isLoading={isLoading}
        emptyMessage="No tokens found"
      />

      {/* Pagination */}
      {tokensResponse && tokensResponse.total_pages > 1 && (
        <Pagination
          currentPage={page}
          totalPages={tokensResponse.total_pages}
          onPageChange={setPage}
        />
      )}
    </div>
  );
}
