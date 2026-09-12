'use client';

import useSWR, { type SWRConfiguration, type SWRResponse } from 'swr';
import * as api from '@/lib/api';
import type {
  ApiKey,
  BlockDetail,
  BlockSummary,
  BridgeState,
  Health,
  NetworkStats,
  NodeInfo,
  Note,
  Paginated,
  ProgramDetail,
  ProgramSummary,
  SearchResult,
  Supply,
  TransactionDetail,
  TransactionKind,
  TransactionSummary,
  User,
  Validator,
  ValidatorDetail,
} from '@/types';

const defaultConfig: SWRConfiguration = {
  revalidateOnFocus: false,
  revalidateOnReconnect: true,
  dedupingInterval: 2000,
  shouldRetryOnError: (err: unknown) => !api.isNotFoundError(err),
};

/** Detail pages need a stable "this thing does not exist" signal. */
export function isNotFound(error: unknown): boolean {
  return api.isNotFoundError(error);
}

// ---------------------------------------------------------------------------
// Health & stats
// ---------------------------------------------------------------------------

export function useHealth(config?: SWRConfiguration): SWRResponse<Health> {
  return useSWR<Health>('health', api.getHealth, {
    ...defaultConfig,
    refreshInterval: 30_000,
    ...config,
  });
}

export function useStats(config?: SWRConfiguration): SWRResponse<NetworkStats> {
  return useSWR<NetworkStats>('stats', api.getStats, {
    ...defaultConfig,
    refreshInterval: 10_000,
    ...config,
  });
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

export function useBlocks(
  page = 1,
  limit = 25,
  proposer?: string,
  config?: SWRConfiguration
): SWRResponse<Paginated<BlockSummary>> {
  return useSWR<Paginated<BlockSummary>>(
    ['blocks', page, limit, proposer ?? null],
    () => api.getBlocks(page, limit, proposer),
    { ...defaultConfig, keepPreviousData: true, ...config }
  );
}

export function useLatestBlocks(
  limit = 10,
  config?: SWRConfiguration
): SWRResponse<BlockSummary[]> {
  return useSWR<BlockSummary[]>(
    ['blocks/latest', limit],
    () => api.getLatestBlocks(limit),
    { ...defaultConfig, refreshInterval: 15_000, ...config }
  );
}

export function useBlock(
  id: string | number | null,
  config?: SWRConfiguration
): SWRResponse<BlockDetail> {
  return useSWR<BlockDetail>(
    id === null || id === '' ? null : ['block', String(id)],
    () => api.getBlock(id as string | number),
    { ...defaultConfig, ...config }
  );
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

export function useTransactions(
  page = 1,
  limit = 25,
  kind?: TransactionKind | null,
  filter?: { validator?: string | null; program?: string | null; height?: number | null },
  config?: SWRConfiguration
): SWRResponse<Paginated<TransactionSummary>> {
  const validator = filter?.validator ?? null;
  const program = filter?.program ?? null;
  const height = filter?.height ?? null;
  return useSWR<Paginated<TransactionSummary>>(
    ['transactions', page, limit, kind ?? null, validator, program, height],
    () => api.getTransactions({ page, limit, kind, validator, program, height }),
    { ...defaultConfig, keepPreviousData: true, ...config }
  );
}

export function useLatestTransactions(
  limit = 10,
  config?: SWRConfiguration
): SWRResponse<TransactionSummary[]> {
  return useSWR<TransactionSummary[]>(
    ['transactions/latest', limit],
    () => api.getLatestTransactions(limit),
    { ...defaultConfig, refreshInterval: 15_000, ...config }
  );
}

export function useTransaction(
  hash: string | null,
  config?: SWRConfiguration
): SWRResponse<TransactionDetail> {
  return useSWR<TransactionDetail>(
    hash ? ['transaction', hash] : null,
    () => api.getTransaction(hash as string),
    { ...defaultConfig, ...config }
  );
}

// ---------------------------------------------------------------------------
// Notes, bridge, supply
// ---------------------------------------------------------------------------

export function useNotes(
  page = 1,
  limit = 25,
  config?: SWRConfiguration
): SWRResponse<Paginated<Note>> {
  return useSWR<Paginated<Note>>(['notes', page, limit], () => api.getNotes(page, limit), {
    ...defaultConfig,
    keepPreviousData: true,
    refreshInterval: 15_000,
    ...config,
  });
}

export function useNote(
  id: string | null,
  config?: SWRConfiguration
): SWRResponse<Note> {
  return useSWR<Note>(id ? ['note', id] : null, () => api.getNote(id as string), {
    ...defaultConfig,
    ...config,
  });
}

export function useBridge(config?: SWRConfiguration): SWRResponse<BridgeState> {
  return useSWR<BridgeState>('bridge', api.getBridge, {
    ...defaultConfig,
    refreshInterval: 30_000,
    ...config,
  });
}

/** `null` when the node serves no supply audit (404). */
export function useSupply(config?: SWRConfiguration): SWRResponse<Supply | null> {
  return useSWR<Supply | null>(
    'supply',
    async () => {
      try {
        return await api.getSupply();
      } catch (err) {
        if (api.isNotFoundError(err)) return null;
        throw err;
      }
    },
    { ...defaultConfig, refreshInterval: 30_000, ...config }
  );
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

export function useValidators(config?: SWRConfiguration): SWRResponse<Validator[]> {
  return useSWR<Validator[]>('validators', api.getValidators, {
    ...defaultConfig,
    refreshInterval: 30_000,
    ...config,
  });
}

export function useValidator(
  address: string | null,
  config?: SWRConfiguration
): SWRResponse<ValidatorDetail> {
  return useSWR<ValidatorDetail>(
    address ? ['validator', address] : null,
    () => api.getValidator(address as string),
    { ...defaultConfig, ...config }
  );
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

export function usePrograms(
  page = 1,
  limit = 25,
  config?: SWRConfiguration
): SWRResponse<Paginated<ProgramSummary>> {
  return useSWR<Paginated<ProgramSummary>>(
    ['programs', page, limit],
    () => api.getPrograms(page, limit),
    { ...defaultConfig, keepPreviousData: true, ...config }
  );
}

export function useProgram(
  id: string | null,
  config?: SWRConfiguration
): SWRResponse<ProgramDetail> {
  return useSWR<ProgramDetail>(
    id ? ['program', id] : null,
    () => api.getProgram(id as string),
    { ...defaultConfig, ...config }
  );
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

export function useNodes(config?: SWRConfiguration): SWRResponse<NodeInfo[]> {
  return useSWR<NodeInfo[]>('nodes', api.getNodes, {
    ...defaultConfig,
    refreshInterval: 30_000,
    ...config,
  });
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

export function useSearch(
  q: string | null,
  config?: SWRConfiguration
): SWRResponse<SearchResult[]> {
  return useSWR<SearchResult[]>(
    q && q.trim() !== '' ? ['search', q.trim()] : null,
    () => api.search((q as string).trim()),
    { ...defaultConfig, ...config }
  );
}

// ---------------------------------------------------------------------------
// Accounts and API keys
// ---------------------------------------------------------------------------

/** The signed-in user, or `null` when there is no session. Never retries a 401. */
export function useMe(config?: SWRConfiguration): SWRResponse<User | null> {
  return useSWR<User | null>(
    'me',
    async () => {
      try {
        return await api.getMe();
      } catch (err) {
        if (err instanceof api.ApiError && err.status === 401) return null;
        throw err;
      }
    },
    { ...defaultConfig, shouldRetryOnError: false, ...config }
  );
}

export function useApiKeys(enabled: boolean, config?: SWRConfiguration): SWRResponse<ApiKey[]> {
  return useSWR<ApiKey[]>(enabled ? 'keys' : null, api.listKeys, {
    ...defaultConfig,
    shouldRetryOnError: false,
    ...config,
  });
}
