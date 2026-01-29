'use client';

import { useSearchParams, useRouter } from 'next/navigation';
import { Suspense } from 'react';
import Link from 'next/link';
import { useSearch } from '@/hooks/useApi';
import { PageLoading } from '@/components/Loading';
import { shortenHash, getSearchQueryType } from '@/lib/utils';
import type { SearchResult } from '@/types';

function SearchContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const query = searchParams.get('q') || '';

  const { data: results, isLoading, error } = useSearch(query);

  // Check for exact match and redirect
  if (query && !isLoading && results?.length === 1) {
    const queryType = getSearchQueryType(query);
    if (queryType !== 'unknown') {
      router.replace(results[0].url);
      return <PageLoading />;
    }
  }

  const getResultIcon = (type: SearchResult['type']) => {
    switch (type) {
      case 'block':
        return (
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        );
      case 'transaction':
        return (
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
        );
      case 'account':
        return (
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        );
      case 'validator':
        return (
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
        );
      case 'token':
        return (
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
    }
  };

  const getTypeLabel = (type: SearchResult['type']) => {
    const labels: Record<SearchResult['type'], string> = {
      block: 'Block',
      transaction: 'Transaction',
      account: 'Account',
      validator: 'Validator',
      token: 'Token',
    };
    return labels[type];
  };

  const groupedResults = results?.reduce((acc, result) => {
    if (!acc[result.type]) {
      acc[result.type] = [];
    }
    acc[result.type].push(result);
    return acc;
  }, {} as Record<SearchResult['type'], SearchResult[]>);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Search Results</h1>
        {query && (
          <p className="mt-1 text-dark-400">
            Results for &quot;<span className="text-white font-mono">{shortenHash(query, 16, 16)}</span>&quot;
          </p>
        )}
      </div>

      {/* Loading */}
      {isLoading && <PageLoading />}

      {/* Error */}
      {error && (
        <div className="rounded-xl border border-red-600/30 bg-red-900/20 p-6 text-center">
          <p className="text-red-400">Failed to search. Please try again.</p>
        </div>
      )}

      {/* No Query */}
      {!query && !isLoading && (
        <div className="flex h-64 flex-col items-center justify-center rounded-xl border border-dark-700 bg-dark-800">
          <svg className="h-12 w-12 text-dark-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <p className="mt-4 text-dark-400">Enter a search query to find blocks, transactions, accounts, validators, or tokens.</p>
        </div>
      )}

      {/* No Results */}
      {query && !isLoading && results && results.length === 0 && (
        <div className="flex h-64 flex-col items-center justify-center rounded-xl border border-dark-700 bg-dark-800">
          <svg className="h-12 w-12 text-dark-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p className="mt-4 text-dark-400">No results found for your search.</p>
          <p className="mt-2 text-sm text-dark-500">
            Try searching for a block number, transaction signature, or account address.
          </p>
        </div>
      )}

      {/* Results by Type */}
      {groupedResults && Object.entries(groupedResults).map(([type, typeResults]) => (
        <div key={type} className="space-y-3">
          <h2 className="text-lg font-semibold text-white">
            {getTypeLabel(type as SearchResult['type'])}s ({typeResults.length})
          </h2>
          <div className="space-y-2">
            {typeResults.map((result, index) => (
              <Link
                key={`${result.type}-${result.id}-${index}`}
                href={result.url}
                className="flex items-center gap-4 rounded-lg border border-dark-700 bg-dark-800 p-4 transition-colors hover:border-dark-600 hover:bg-dark-700"
              >
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-dark-700 text-primary-400">
                  {getResultIcon(result.type)}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-white">{result.label}</p>
                  {result.description && (
                    <p className="truncate text-sm text-dark-400">{result.description}</p>
                  )}
                </div>
                <span className="flex-shrink-0 rounded bg-dark-700 px-2.5 py-1 text-xs font-medium text-dark-300">
                  {getTypeLabel(result.type)}
                </span>
                <svg className="h-5 w-5 flex-shrink-0 text-dark-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                </svg>
              </Link>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export default function SearchPage() {
  return (
    <Suspense fallback={<PageLoading />}>
      <SearchContent />
    </Suspense>
  );
}
