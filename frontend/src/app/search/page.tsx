'use client';

import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { Suspense, useEffect } from 'react';
import { SearchBar } from '@/components/SearchBar';
import { PageLoading } from '@/components/Loading';
import { ErrorState, NotFoundState, PageHeader } from '@/components/States';
import { useSearch } from '@/hooks/useApi';
import type { SearchResult } from '@/types';

export default function SearchPage() {
  return (
    <Suspense fallback={<PageLoading />}>
      <SearchResults />
    </Suspense>
  );
}

function SearchResults() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const query = (searchParams.get('q') ?? '').trim();

  const { data: results, error, isLoading, mutate } = useSearch(query || null);

  // A single unambiguous hit goes straight to its page.
  useEffect(() => {
    if (results && results.length === 1) {
      router.replace(results[0].url);
    }
  }, [results, router]);

  if (!query) {
    return (
      <div className="space-y-6">
        <PageHeader title="Search" subtitle="Find blocks, transactions, accounts and programs" />
        <div className="card-padded">
          <SearchBar autoFocus />
          <p className="mt-4 text-sm text-mute">
            Search by block height, 64-character hash (block, transaction or program) or a base58
            account address.
          </p>
        </div>
      </div>
    );
  }

  if (isLoading && !results) {
    return (
      <div className="space-y-6">
        <PageHeader title="Search" subtitle={`Searching for “${query}”…`} />
        <PageLoading />
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <PageHeader title="Search" subtitle={`Results for “${query}”`} />
        <ErrorState message="The search request failed." onRetry={() => void mutate()} />
      </div>
    );
  }

  if (!results || results.length === 0) {
    return (
      <div className="space-y-6">
        <PageHeader title="Search" subtitle={`Results for “${query}”`} />
        <NotFoundState
          title="No results"
          message={`Nothing on chain matches “${query}”.`}
          backHref="/"
          backLabel="Back to dashboard"
        />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Search"
        subtitle={`${results.length} result${results.length === 1 ? '' : 's'} for “${query}”`}
      />

      <ul className="card divide-y divide-border-soft">
        {results.map((result, index) => (
          <li key={`${result.type}-${result.id}-${index}`}>
            <Link
              href={result.url}
              className="flex items-center gap-4 px-6 py-4 transition-colors hover:bg-bg-soft"
            >
              <ResultIcon type={result.type} />
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium text-strong">{result.title}</p>
                {result.subtitle && (
                  <p className="truncate text-xs text-mute">{result.subtitle}</p>
                )}
                <p className="mt-0.5 truncate font-mono text-xs text-mute">{result.id}</p>
              </div>
              <span className="badge badge-neutral flex-shrink-0 capitalize">{result.type}</span>
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}

function ResultIcon({ type }: { type: SearchResult['type'] }) {
  const paths: Record<SearchResult['type'], string> = {
    block: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4',
    transaction: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4',
    account: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z',
    validator:
      'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
    program: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
  };

  return (
    <span className="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded bg-surface-2 text-accent">
      <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d={paths[type]} />
      </svg>
    </span>
  );
}
