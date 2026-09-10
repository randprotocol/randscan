import type {
  AccountDetail,
  AccountTransaction,
  ApiErrorBody,
  ApiKey,
  BlockDetail,
  BlockSummary,
  CreatedApiKey,
  Health,
  NetworkStats,
  NodeInfo,
  Paginated,
  ProgramDetail,
  ProgramSummary,
  SearchResult,
  TransactionDetail,
  TransactionKind,
  TransactionSummary,
  User,
  Validator,
  ValidatorDetail,
} from '@/types';

/**
 * Requests always go to the same origin (`/api/v1/...`). In production Caddy proxies them;
 * in development `next.config.js` rewrites them to `NEXT_PUBLIC_API_URL`. Same-origin is what
 * lets the session cookie work.
 */
export const API_BASE_URL = '';

const API_PREFIX = '/api/v1';

export class ApiError extends Error {
  readonly status: number;
  readonly error: string;
  readonly code?: string | number | null;

  constructor(status: number, body: Partial<ApiErrorBody> = {}) {
    super(body.message || body.error || `Request failed with status ${status}`);
    this.name = 'ApiError';
    this.status = status;
    this.error = body.error ?? 'error';
    this.code = body.code ?? null;
  }

  get isNotFound(): boolean {
    return this.status === 404 || this.error === 'not_found';
  }
}

export function isNotFoundError(err: unknown): boolean {
  return err instanceof ApiError && err.isNotFound;
}

type QueryValue = string | number | boolean | null | undefined;

function buildQuery(params: Record<string, QueryValue>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === null || value === undefined || value === '') continue;
    search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : '';
}

/**
 * Performs the fetch shared by `request()` and `requestVoid()`: builds the URL, sends the
 * request same-origin, maps a thrown fetch error to `ApiError(0, ...)`, and throws
 * `ApiError(response.status, ...)` on a non-2xx response.
 */
async function send(path: string, init?: RequestInit): Promise<Response> {
  const url = `${API_BASE_URL}${API_PREFIX}${path}`;

  let response: Response;
  try {
    response = await fetch(url, {
      ...init,
      credentials: 'same-origin',
      headers: { Accept: 'application/json', ...init?.headers },
    });
  } catch (cause) {
    throw new ApiError(0, {
      error: 'network_error',
      message: cause instanceof Error ? cause.message : 'Network request failed',
    });
  }

  if (!response.ok) {
    let body: Partial<ApiErrorBody> = {};
    try {
      body = (await response.json()) as Partial<ApiErrorBody>;
    } catch {
      // Response had no JSON body; fall back to the status-derived message.
    }
    throw new ApiError(response.status, body);
  }

  return response;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  return (await (await send(path, init)).json()) as T;
}

// ---------------------------------------------------------------------------
// Health & stats
// ---------------------------------------------------------------------------

export function getHealth(): Promise<Health> {
  return request<Health>('/health');
}

export function getStats(): Promise<NetworkStats> {
  return request<NetworkStats>('/stats');
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

export function getBlocks(
  page = 1,
  limit = 25,
  proposer?: string
): Promise<Paginated<BlockSummary>> {
  return request<Paginated<BlockSummary>>(
    `/blocks${buildQuery({ page, limit, proposer })}`
  );
}

export function getLatestBlocks(limit = 10): Promise<BlockSummary[]> {
  return request<BlockSummary[]>(`/blocks/latest${buildQuery({ limit })}`);
}

/** `id` is a block height or a 64-hex block hash. */
export function getBlock(id: string | number): Promise<BlockDetail> {
  return request<BlockDetail>(`/blocks/${encodeURIComponent(String(id))}`);
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

export interface TransactionListParams {
  page?: number;
  limit?: number;
  kind?: TransactionKind | null;
  sender?: string | null;
  height?: number | null;
}

export function getTransactions(
  params: TransactionListParams = {}
): Promise<Paginated<TransactionSummary>> {
  const { page = 1, limit = 25, kind, sender, height } = params;
  return request<Paginated<TransactionSummary>>(
    `/transactions${buildQuery({ page, limit, kind, sender, height })}`
  );
}

export function getLatestTransactions(limit = 10): Promise<TransactionSummary[]> {
  return request<TransactionSummary[]>(
    `/transactions/latest${buildQuery({ limit })}`
  );
}

export function getTransaction(hash: string): Promise<TransactionDetail> {
  return request<TransactionDetail>(`/transactions/${encodeURIComponent(hash)}`);
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

export function getAccount(address: string): Promise<AccountDetail> {
  return request<AccountDetail>(`/accounts/${encodeURIComponent(address)}`);
}

export function getAccountTransactions(
  address: string,
  page = 1,
  limit = 25
): Promise<Paginated<AccountTransaction>> {
  return request<Paginated<AccountTransaction>>(
    `/accounts/${encodeURIComponent(address)}/transactions${buildQuery({ page, limit })}`
  );
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

export function getValidators(): Promise<Validator[]> {
  return request<Validator[]>('/validators');
}

export function getValidator(address: string): Promise<ValidatorDetail> {
  return request<ValidatorDetail>(`/validators/${encodeURIComponent(address)}`);
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

export function getPrograms(
  page = 1,
  limit = 25
): Promise<Paginated<ProgramSummary>> {
  return request<Paginated<ProgramSummary>>(
    `/programs${buildQuery({ page, limit })}`
  );
}

export function getProgram(id: string): Promise<ProgramDetail> {
  return request<ProgramDetail>(`/programs/${encodeURIComponent(id)}`);
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

export function getNodes(): Promise<NodeInfo[]> {
  return request<NodeInfo[]>('/nodes');
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

export function search(q: string): Promise<SearchResult[]> {
  return request<SearchResult[]>(`/search${buildQuery({ q })}`);
}

// ---------------------------------------------------------------------------
// Accounts and API keys (cookie session; same-origin only)
// ---------------------------------------------------------------------------

function jsonInit(method: string, body?: unknown): RequestInit {
  return {
    method,
    headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  };
}

async function requestVoid(path: string, init: RequestInit): Promise<void> {
  await send(path, init);
}

export async function signup(email: string, password: string): Promise<User> {
  const res = await request<{ user: User }>('/auth/signup', jsonInit('POST', { email, password }));
  return res.user;
}

export async function login(email: string, password: string): Promise<User> {
  const res = await request<{ user: User }>('/auth/login', jsonInit('POST', { email, password }));
  return res.user;
}

export function logout(): Promise<void> {
  return requestVoid('/auth/logout', jsonInit('POST'));
}

export async function getMe(): Promise<User> {
  const res = await request<{ user: User }>('/auth/me');
  return res.user;
}

export function listKeys(): Promise<ApiKey[]> {
  return request<ApiKey[]>('/keys');
}

export function createKey(name: string): Promise<CreatedApiKey> {
  return request<CreatedApiKey>('/keys', jsonInit('POST', { name }));
}

export function revokeKey(id: number): Promise<void> {
  return requestVoid(`/keys/${id}`, jsonInit('DELETE'));
}

// ---------------------------------------------------------------------------
// Password reset and change
// ---------------------------------------------------------------------------

/** Always resolves (202) whether or not the address is registered; 503 when email is disabled. */
export function forgotPassword(email: string): Promise<void> {
  return requestVoid('/auth/forgot', jsonInit('POST', { email }));
}

/** Consumes the emailed token, sets the password and signs the user in. */
export async function resetPassword(token: string, password: string): Promise<User> {
  const res = await request<{ user: User }>('/auth/reset', jsonInit('POST', { token, password }));
  return res.user;
}

export function changePassword(currentPassword: string, newPassword: string): Promise<void> {
  return requestVoid(
    '/auth/password',
    jsonInit('POST', { current_password: currentPassword, new_password: newPassword })
  );
}
