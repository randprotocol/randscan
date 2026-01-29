// Network Statistics
export interface NetworkStats {
  block_height: number;
  slot: number;
  epoch: number;
  epoch_progress: number;
  tps: number;
  average_tps: number;
  total_transactions: number;
  active_validators: number;
  total_stake: string;
  atlas_price?: number;
  shrug_price?: number;
}

// Block Types
export interface BlockSummary {
  slot: number;
  blockhash: string;
  parent_slot: number;
  timestamp: number;
  transaction_count: number;
  leader: string;
  rewards: string;
}

export interface BlockDetail {
  slot: number;
  blockhash: string;
  parent_slot: number;
  parent_blockhash: string;
  timestamp: number;
  transaction_count: number;
  leader: string;
  leader_identity?: string;
  rewards: string;
  fee_rewards: string;
  transactions: TransactionSummary[];
  qc_hash?: string;
  qc_slot?: number;
  previous_blockhash: string;
}

// Transaction Types
export type TransactionStatus = 'success' | 'failed' | 'pending';
export type TransactionType = 'transfer' | 'stake' | 'vote' | 'program' | 'system' | 'unknown';

export interface TransactionSummary {
  signature: string;
  slot: number;
  timestamp: number;
  status: TransactionStatus;
  type: TransactionType;
  fee: string;
  signer: string;
  is_private: boolean;
}

export interface TransactionInstruction {
  program_id: string;
  program_name?: string;
  accounts: string[];
  data: string;
  decoded?: Record<string, unknown>;
}

export interface TransactionDetail {
  signature: string;
  slot: number;
  block_time: number;
  status: TransactionStatus;
  type: TransactionType;
  fee: string;
  signer: string;
  signers: string[];
  recent_blockhash: string;
  instructions: TransactionInstruction[];
  log_messages: string[];
  pre_balances: string[];
  post_balances: string[];
  pre_token_balances: TokenBalance[];
  post_token_balances: TokenBalance[];
  is_private: boolean;
  privacy_level?: 'public' | 'shielded' | 'confidential';
  compute_units_consumed?: number;
  error?: string;
}

// Account Types
export interface TokenBalance {
  mint: string;
  owner: string;
  amount: string;
  decimals: number;
  ui_amount: number;
}

export interface TokenAccount {
  address: string;
  mint: string;
  mint_name?: string;
  mint_symbol?: string;
  balance: string;
  decimals: number;
  ui_balance: number;
}

export interface AccountDetail {
  address: string;
  lamports: string;
  atlas_balance: string;
  shrug_balance: string;
  owner: string;
  executable: boolean;
  rent_epoch: number;
  data_size: number;
  token_accounts: TokenAccount[];
  is_validator: boolean;
  validator_identity?: string;
  stake_accounts: StakeAccount[];
}

export interface StakeAccount {
  address: string;
  stake: string;
  voter: string;
  activation_epoch: number;
  deactivation_epoch?: number;
  status: 'active' | 'activating' | 'deactivating' | 'inactive';
}

export interface AccountTransaction {
  signature: string;
  slot: number;
  timestamp: number;
  status: TransactionStatus;
  type: TransactionType;
  fee: string;
  direction: 'in' | 'out' | 'self';
  amount?: string;
  counterparty?: string;
}

// Validator Types
export interface Validator {
  identity: string;
  vote_account: string;
  name?: string;
  website?: string;
  icon_url?: string;
  stake: string;
  commission: number;
  last_vote: number;
  root_slot: number;
  credits: number;
  epoch_credits: number;
  status: 'active' | 'delinquent' | 'inactive';
  uptime_percentage: number;
  skip_rate: number;
}

export interface ValidatorDetail extends Validator {
  description?: string;
  keybase_username?: string;
  activated_stake: string;
  stake_history: StakeHistoryEntry[];
  recent_blocks: BlockSummary[];
  epoch_vote_account: boolean;
  version?: string;
  feature_set?: number;
}

export interface StakeHistoryEntry {
  epoch: number;
  effective_stake: string;
  activating_stake: string;
  deactivating_stake: string;
}

// Token Types
export interface TokenMint {
  mint: string;
  name?: string;
  symbol?: string;
  decimals: number;
  supply: string;
  ui_supply: number;
  mint_authority?: string;
  freeze_authority?: string;
  is_initialized: boolean;
  holder_count: number;
  icon_url?: string;
}

export interface TokenSupply {
  mint: string;
  total_supply: string;
  circulating_supply: string;
  decimals: number;
  ui_total_supply: number;
  ui_circulating_supply: number;
}

export interface TokenHolder {
  address: string;
  balance: string;
  ui_balance: number;
  percentage: number;
}

// Search Types
export type SearchResultType = 'block' | 'transaction' | 'account' | 'validator' | 'token';

export interface SearchResult {
  type: SearchResultType;
  id: string;
  label: string;
  description?: string;
  url: string;
}

// Pagination
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

// WebSocket Types
export type WebSocketChannel = 'blocks' | 'transactions' | 'account' | 'validator' | 'stats';

export interface WebSocketMessage {
  channel: WebSocketChannel;
  action: 'subscribe' | 'unsubscribe' | 'update';
  data?: unknown;
  params?: Record<string, string>;
}

export interface WebSocketBlockUpdate {
  slot: number;
  blockhash: string;
  timestamp: number;
  transaction_count: number;
  leader: string;
}

export interface WebSocketTransactionUpdate {
  signature: string;
  slot: number;
  status: TransactionStatus;
  type: TransactionType;
  signer: string;
}

export interface WebSocketStatsUpdate {
  block_height: number;
  tps: number;
  total_transactions: number;
}

// API Error
export interface ApiError {
  error: string;
  message: string;
  status: number;
}
