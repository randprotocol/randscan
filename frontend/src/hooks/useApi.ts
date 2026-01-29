'use client';

import useSWR, { SWRConfiguration } from 'swr';
import {
  getStats,
  getBlocks,
  getBlock,
  getLatestBlocks,
  getTransactions,
  getTransaction,
  getLatestTransactions,
  getAccount,
  getAccountTransactions,
  getValidators,
  getValidator,
  getTokens,
  getToken,
  search,
} from '@/lib/api';
import type {
  NetworkStats,
  BlockSummary,
  BlockDetail,
  TransactionSummary,
  TransactionDetail,
  AccountDetail,
  AccountTransaction,
  Validator,
  ValidatorDetail,
  TokenMint,
  SearchResult,
  PaginatedResponse,
} from '@/types';

const defaultConfig: SWRConfiguration = {
  revalidateOnFocus: false,
  revalidateOnReconnect: true,
  dedupingInterval: 5000,
};

// Network Stats
export function useStats(config?: SWRConfiguration) {
  return useSWR<NetworkStats>('stats', getStats, {
    ...defaultConfig,
    refreshInterval: 5000, // Refresh every 5 seconds
    ...config,
  });
}

// Blocks
export function useBlocks(page = 1, pageSize = 20, config?: SWRConfiguration) {
  return useSWR<PaginatedResponse<BlockSummary>>(
    ['blocks', page, pageSize],
    () => getBlocks(page, pageSize),
    { ...defaultConfig, ...config }
  );
}

export function useBlock(slotOrHash: string | number | null, config?: SWRConfiguration) {
  return useSWR<BlockDetail>(
    slotOrHash ? ['block', slotOrHash] : null,
    () => getBlock(slotOrHash!),
    { ...defaultConfig, ...config }
  );
}

export function useLatestBlocks(limit = 5, config?: SWRConfiguration) {
  return useSWR<BlockSummary[]>(
    ['latestBlocks', limit],
    () => getLatestBlocks(limit),
    {
      ...defaultConfig,
      refreshInterval: 3000, // Refresh frequently for latest blocks
      ...config,
    }
  );
}

// Transactions
export function useTransactions(
  page = 1,
  pageSize = 20,
  filters?: {
    type?: string;
    status?: string;
    from_slot?: number;
    to_slot?: number;
  },
  config?: SWRConfiguration
) {
  return useSWR<PaginatedResponse<TransactionSummary>>(
    ['transactions', page, pageSize, filters],
    () => getTransactions(page, pageSize, filters),
    { ...defaultConfig, ...config }
  );
}

export function useTransaction(signature: string | null, config?: SWRConfiguration) {
  return useSWR<TransactionDetail>(
    signature ? ['transaction', signature] : null,
    () => getTransaction(signature!),
    { ...defaultConfig, ...config }
  );
}

export function useLatestTransactions(limit = 5, config?: SWRConfiguration) {
  return useSWR<TransactionSummary[]>(
    ['latestTransactions', limit],
    () => getLatestTransactions(limit),
    {
      ...defaultConfig,
      refreshInterval: 3000,
      ...config,
    }
  );
}

// Accounts
export function useAccount(address: string | null, config?: SWRConfiguration) {
  return useSWR<AccountDetail>(
    address ? ['account', address] : null,
    () => getAccount(address!),
    { ...defaultConfig, ...config }
  );
}

export function useAccountTransactions(
  address: string | null,
  page = 1,
  pageSize = 20,
  config?: SWRConfiguration
) {
  return useSWR<PaginatedResponse<AccountTransaction>>(
    address ? ['accountTransactions', address, page, pageSize] : null,
    () => getAccountTransactions(address!, page, pageSize),
    { ...defaultConfig, ...config }
  );
}

// Validators
export function useValidators(
  page = 1,
  pageSize = 20,
  status?: 'active' | 'delinquent' | 'inactive',
  config?: SWRConfiguration
) {
  return useSWR<PaginatedResponse<Validator>>(
    ['validators', page, pageSize, status],
    () => getValidators(page, pageSize, status),
    { ...defaultConfig, ...config }
  );
}

export function useValidator(identity: string | null, config?: SWRConfiguration) {
  return useSWR<ValidatorDetail>(
    identity ? ['validator', identity] : null,
    () => getValidator(identity!),
    { ...defaultConfig, ...config }
  );
}

// Tokens
export function useTokens(page = 1, pageSize = 20, config?: SWRConfiguration) {
  return useSWR<PaginatedResponse<TokenMint>>(
    ['tokens', page, pageSize],
    () => getTokens(page, pageSize),
    { ...defaultConfig, ...config }
  );
}

export function useToken(mint: string | null, config?: SWRConfiguration) {
  return useSWR<TokenMint>(
    mint ? ['token', mint] : null,
    () => getToken(mint!),
    { ...defaultConfig, ...config }
  );
}

// Search
export function useSearch(query: string | null, config?: SWRConfiguration) {
  return useSWR<SearchResult[]>(
    query && query.length >= 2 ? ['search', query] : null,
    () => search(query!),
    {
      ...defaultConfig,
      dedupingInterval: 1000,
      ...config,
    }
  );
}
