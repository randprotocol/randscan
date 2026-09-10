// RandScan API types — mirrors the `/api/v1` contract served by randscan-api.
// All amounts are decimal strings of smallest units (SHRUGG has 9 decimals).
// All timestamps are `timestamp_ms`: milliseconds since the Unix epoch.

export type TransactionKind = 'transfer' | 'mint' | 'deploy' | 'call';

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';

export type AccountRole = 'sender' | 'recipient';

export type SearchResultType =
  | 'block'
  | 'transaction'
  | 'account'
  | 'validator'
  | 'program';

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

export interface PaginationMeta {
  page: number;
  limit: number;
  total: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

export interface Paginated<T> {
  data: T[];
  pagination: PaginationMeta;
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

export interface BlockSummary {
  hash: string;
  height: number;
  view: number;
  parent: string;
  proposer: string;
  timestamp_ms: number;
  tx_count: number;
  justify_view: number;
}

export interface BlockDetail extends BlockSummary {
  tx_root: string;
  state_root: string;
  transactions: TransactionSummary[];
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

export interface TransactionSummary {
  hash: string;
  height: number;
  block_hash: string;
  tx_index: number;
  sender: string;
  nonce: number;
  fee: string;
  kind: TransactionKind;
  timestamp_ms: number;
  to: string | null;
  amount: string | null;
  program: string | null;
}

export interface ReceiptEffect {
  to: string;
  amount: string;
}

export interface Receipt {
  tx: string;
  program: string;
  tier: number;
  outputs: number[];
  effect: ReceiptEffect | null;
  height: number;
  index: number;
}

export interface TransactionDetail extends TransactionSummary {
  chain_id: number;
  base_pc: number | null;
  words_len: number | null;
  proof_len: number | null;
  recipients: string[];
  receipt: Receipt | null;
}

export interface AccountTransaction extends TransactionSummary {
  role: AccountRole;
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

export interface AccountDetail {
  address: string;
  balance: string;
  nonce: number;
  tx_count: number;
  first_seen_height: number;
  last_seen_height: number;
  is_validator: boolean;
  stake: string | null;
  programs_deployed: number;
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

export interface Validator {
  address: string;
  stake: string;
  share_percent: number;
  blocks_proposed: number;
  last_proposed_height: number | null;
  last_proposed_timestamp_ms: number | null;
  sort_index: number;
}

export interface ValidatorDetail extends Validator {
  recent_blocks: BlockSummary[];
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

export interface ProgramSummary {
  id: string;
  deployer: string;
  deploy_tx: string;
  deployed_at_height: number;
  base_pc: number;
  words_len: number;
  code_hash: string;
  call_count: number;
  last_called_height: number | null;
}

export interface ProgramDetail extends ProgramSummary {
  recent_calls: TransactionSummary[];
}

// ---------------------------------------------------------------------------
// Network / health
// ---------------------------------------------------------------------------

export interface NetworkStats {
  chain_id: number;
  symbol: string;
  decimals: number;
  height: number;
  view: number;
  total_transactions: number;
  total_accounts: number;
  validator_count: number;
  total_stake: string;
  total_supply: string;
  program_count: number;
  avg_block_time_ms: number;
  peer_count: number;
  mempool_size: number;
  node_syncing: boolean;
  faucet: boolean;
  confidential: boolean;
  current_leader: string | null;
  updated_at: string;
}

export interface IndexerHealth {
  connected: boolean;
  synced: boolean;
  current_height: number;
  node_height: number;
  lag: number;
}

export interface Health {
  status: HealthStatus;
  version: string;
  database: string;
  indexer: IndexerHealth;
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

export type NodeRole = 'validator' | 'observer' | 'peer';

export interface GeoInfo {
  lat: number;
  lon: number;
  city: string | null;
  region: string | null;
  country: string | null;
  country_code: string | null;
  org: string | null;
}

export interface NodeInfo {
  peer_id: string;
  ip: string | null;
  port: number | null;
  connected_secs: number | null;
  /** True for the node this explorer runs alongside. */
  is_self: boolean;
  role: NodeRole;
  /** Null while a geo lookup is pending, or for private/unroutable addresses. */
  geo: GeoInfo | null;
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

export interface SearchResult {
  type: SearchResultType;
  id: string;
  title: string;
  subtitle?: string | null;
  url: string;
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

export interface ApiErrorBody {
  error: string;
  message: string;
  code?: string | number | null;
}

// ---------------------------------------------------------------------------
// WebSocket
// ---------------------------------------------------------------------------

export type WebSocketChannel = 'blocks' | 'transactions' | 'stats';

export interface ClientSubscribeMessage {
  type: 'subscribe' | 'unsubscribe';
  channel: WebSocketChannel;
}

export interface ClientPingMessage {
  type: 'ping';
}

export type ClientMessage = ClientSubscribeMessage | ClientPingMessage;

export interface ServerSubscribedMessage {
  type: 'subscribed';
  channel: WebSocketChannel;
  subscription_id: string;
}

export interface ServerUnsubscribedMessage {
  type: 'unsubscribed';
  channel: WebSocketChannel;
  subscription_id?: string;
}

export interface ServerPongMessage {
  type: 'pong';
}

export interface ServerErrorMessage {
  type: 'error';
  message?: string;
  error?: string;
}

export interface ServerNewBlockMessage {
  type: 'new_block';
  block: BlockSummary;
}

export interface ServerNewTransactionMessage {
  type: 'new_transaction';
  transaction: TransactionSummary;
}

export interface ServerStatsUpdateMessage {
  type: 'stats_update';
  stats: NetworkStats;
}

export type ServerMessage =
  | ServerSubscribedMessage
  | ServerUnsubscribedMessage
  | ServerPongMessage
  | ServerErrorMessage
  | ServerNewBlockMessage
  | ServerNewTransactionMessage
  | ServerStatsUpdateMessage;

// ---------------------------------------------------------------------------
// Accounts and API keys
// ---------------------------------------------------------------------------

export interface User {
  id: number;
  email: string;
  created_at: string;
  last_login_at: string | null;
}

export interface ApiKey {
  id: number;
  name: string;
  prefix: string;
  created_at: string;
  last_used_at: string | null;
  request_count: number;
  revoked_at: string | null;
}

/** Returned once at creation; `key` is the full secret. */
export interface CreatedApiKey extends ApiKey {
  key: string;
}
