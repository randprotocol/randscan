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
  TokenSupply,
  TokenHolder,
  SearchResult,
  PaginatedResponse,
  ApiError,
} from '@/types';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';
const API_VERSION = '/api/v1';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = `${baseUrl}${API_VERSION}`;
  }

  private async fetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const error: ApiError = {
        error: 'API Error',
        message: `Request failed with status ${response.status}`,
        status: response.status,
      };

      try {
        const body = await response.json();
        error.message = body.message || body.error || error.message;
      } catch {
        // Use default message
      }

      throw error;
    }

    return response.json();
  }

  // Stats
  async getStats(): Promise<NetworkStats> {
    return this.fetch<NetworkStats>('/stats');
  }

  // Blocks
  async getBlocks(page = 1, pageSize = 20): Promise<PaginatedResponse<BlockSummary>> {
    return this.fetch<PaginatedResponse<BlockSummary>>(
      `/blocks?page=${page}&page_size=${pageSize}`
    );
  }

  async getBlock(slotOrHash: string | number): Promise<BlockDetail> {
    return this.fetch<BlockDetail>(`/blocks/${slotOrHash}`);
  }

  async getLatestBlocks(limit = 5): Promise<BlockSummary[]> {
    return this.fetch<BlockSummary[]>(`/blocks/latest?limit=${limit}`);
  }

  // Transactions
  async getTransactions(
    page = 1,
    pageSize = 20,
    filters?: {
      type?: string;
      status?: string;
      from_slot?: number;
      to_slot?: number;
    }
  ): Promise<PaginatedResponse<TransactionSummary>> {
    const params = new URLSearchParams({
      page: page.toString(),
      page_size: pageSize.toString(),
    });

    if (filters?.type) params.set('type', filters.type);
    if (filters?.status) params.set('status', filters.status);
    if (filters?.from_slot) params.set('from_slot', filters.from_slot.toString());
    if (filters?.to_slot) params.set('to_slot', filters.to_slot.toString());

    return this.fetch<PaginatedResponse<TransactionSummary>>(
      `/transactions?${params.toString()}`
    );
  }

  async getTransaction(signature: string): Promise<TransactionDetail> {
    return this.fetch<TransactionDetail>(`/transactions/${signature}`);
  }

  async getLatestTransactions(limit = 5): Promise<TransactionSummary[]> {
    return this.fetch<TransactionSummary[]>(`/transactions/latest?limit=${limit}`);
  }

  // Accounts
  async getAccount(address: string): Promise<AccountDetail> {
    return this.fetch<AccountDetail>(`/accounts/${address}`);
  }

  async getAccountTransactions(
    address: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<AccountTransaction>> {
    return this.fetch<PaginatedResponse<AccountTransaction>>(
      `/accounts/${address}/transactions?page=${page}&page_size=${pageSize}`
    );
  }

  async getAccountTokens(address: string): Promise<TokenMint[]> {
    return this.fetch<TokenMint[]>(`/accounts/${address}/tokens`);
  }

  // Validators
  async getValidators(
    page = 1,
    pageSize = 20,
    status?: 'active' | 'delinquent' | 'inactive'
  ): Promise<PaginatedResponse<Validator>> {
    const params = new URLSearchParams({
      page: page.toString(),
      page_size: pageSize.toString(),
    });

    if (status) params.set('status', status);

    return this.fetch<PaginatedResponse<Validator>>(
      `/validators?${params.toString()}`
    );
  }

  async getValidator(identity: string): Promise<ValidatorDetail> {
    return this.fetch<ValidatorDetail>(`/validators/${identity}`);
  }

  // Tokens
  async getTokens(page = 1, pageSize = 20): Promise<PaginatedResponse<TokenMint>> {
    return this.fetch<PaginatedResponse<TokenMint>>(
      `/tokens?page=${page}&page_size=${pageSize}`
    );
  }

  async getToken(mint: string): Promise<TokenMint> {
    return this.fetch<TokenMint>(`/tokens/${mint}`);
  }

  async getTokenSupply(mint: string): Promise<TokenSupply> {
    return this.fetch<TokenSupply>(`/tokens/${mint}/supply`);
  }

  async getTokenHolders(
    mint: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<TokenHolder>> {
    return this.fetch<PaginatedResponse<TokenHolder>>(
      `/tokens/${mint}/holders?page=${page}&page_size=${pageSize}`
    );
  }

  // Search
  async search(query: string): Promise<SearchResult[]> {
    return this.fetch<SearchResult[]>(`/search?q=${encodeURIComponent(query)}`);
  }
}

export const api = new ApiClient();

// Export individual functions for convenience
export const getStats = () => api.getStats();
export const getBlocks = (page?: number, pageSize?: number) => api.getBlocks(page, pageSize);
export const getBlock = (id: string | number) => api.getBlock(id);
export const getLatestBlocks = (limit?: number) => api.getLatestBlocks(limit);
export const getTransactions = (
  page?: number,
  pageSize?: number,
  filters?: Parameters<typeof api.getTransactions>[2]
) => api.getTransactions(page, pageSize, filters);
export const getTransaction = (sig: string) => api.getTransaction(sig);
export const getLatestTransactions = (limit?: number) => api.getLatestTransactions(limit);
export const getAccount = (address: string) => api.getAccount(address);
export const getAccountTransactions = (address: string, page?: number, pageSize?: number) =>
  api.getAccountTransactions(address, page, pageSize);
export const getValidators = (
  page?: number,
  pageSize?: number,
  status?: Parameters<typeof api.getValidators>[2]
) => api.getValidators(page, pageSize, status);
export const getValidator = (id: string) => api.getValidator(id);
export const getTokens = (page?: number, pageSize?: number) => api.getTokens(page, pageSize);
export const getToken = (mint: string) => api.getToken(mint);
export const getTokenSupply = (mint: string) => api.getTokenSupply(mint);
export const getTokenHolders = (mint: string, page?: number, pageSize?: number) =>
  api.getTokenHolders(mint, page, pageSize);
export const search = (query: string) => api.search(query);
