// RandScan API types — mirrors the `/api/v1` contract served by randscan-api.
// All amounts are decimal strings of smallest units (SHRUGG has 9 decimals).
// All timestamps are `timestamp_ms`: milliseconds since the Unix epoch.

/** `other` is any kind the node serves that this explorer build does not decode. */
export type TransactionKind =
  | 'transfer'
  | 'mint'
  | 'deploy'
  | 'call'
  | 'bond'
  | 'unbond'
  | 'withdraw'
  | 'bridge_attest'
  | 'bridge_burn'
  | 'other';

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';

export type SearchResultType =
  | 'block'
  | 'transaction'
  | 'validator'
  | 'program'
  | 'note'
  | 'nullifier';

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

/** The public fields of a shielded bundle: what every observer sees of a transfer. */
export interface Bundle {
  anchor: string;
  nullifiers: [string, string];
  commitments: [string, string];
  /** Units of SHRUGG. */
  fee: string;
  /** Units leaving the pool into the action (bond, bridge burn). */
  burn: string;
  /** 0 = SHRUGG; otherwise the bridge registry index of the balanced asset. */
  asset: number;
  /** Block height the sender targeted. */
  time: number;
  proof_len: number;
  envelope_len: [number, number];
}

/**
 * There is no sender, recipient or nonce: a shielded transaction has none. `amount` is the one
 * public amount an action carries (a deposit's note value or a staking move).
 */
export interface TransactionSummary {
  hash: string;
  height: number;
  block_hash: string;
  tx_index: number;
  kind: TransactionKind;
  /** SHRUGG fee paid by the bundle ("0" for a validator-signed action without one). */
  fee: string;
  timestamp_ms: number;
  /** False for mint / unbond / withdraw, which are signed by a validator instead. */
  has_bundle: boolean;
  /** deploy: deployed program id; call: called program id */
  program: string | null;
  /** bond / unbond / withdraw: the validator; mint: the minting validator */
  validator: string | null;
  /** mint, bond, unbond, withdraw: SHRUGG units; bridge_attest, bridge_burn: bridged units */
  amount: string | null;
  /** bridge_attest / bridge_burn: the bridged asset's registry index */
  asset_index: number | null;
}

export interface Receipt {
  tx: string;
  program: string;
  tier: number;
  outputs: number[];
  height: number;
  index: number;
  /** The proof's salted commitment to the call's private inputs (zkVM M4.1). */
  h_in: string;
}

export interface TransactionDetail extends TransactionSummary {
  chain_id: number;
  /** The fee bundle; null for a validator-signed action. */
  bundle: Bundle | null;
  /** deploy: program length in words */
  words_len: number | null;
  /** call: size of the call's own proof */
  call_proof_len: number | null;
  /** call: size of the sealed input transcript, null when the caller published none */
  input_envelope_len: number | null;
  receipt: Receipt | null;
  /** mint: the deposit note's commitment */
  cm: string | null;
  /** bond: whether this bond registered a new validator */
  registered: boolean | null;
  /** unbond / withdraw: the register nonce the validator signed */
  action_nonce: number | null;
  /** bridge_attest: size of the guardian-signed message */
  attestation_len: number | null;
  /** bridge_attest: the depositor's shielded address (shrugg1…) */
  recipient: string | null;
  /** bridge_attest: the deposit note's time word */
  note_time: number | null;
  /** bridge_burn: relayer fee in bridged units */
  relayer_fee: string | null;
  /** bridge_burn: destination chain id (1 Rand, 2 Ethereum, 3 BSC, 4 Tron, 5 Solana) */
  to_chain: number | null;
  /** bridge_burn: destination address, 32 bytes hex */
  bridge_to: string | null;
  /** bridge_burn: the second bundle, which burns the bridged asset */
  asset_bundle: Bundle | null;
}

// ---------------------------------------------------------------------------
// Notes, nullifiers, bridge, supply
// ---------------------------------------------------------------------------

/** One leaf of the commitment tree. `tx_hash` is null for genesis, withdraw and bridge deposits. */
export interface Note {
  leaf_index: number;
  cm: string;
  height: number;
  tx_hash: string | null;
}

export interface Nullifier {
  nullifier: string;
  tx_hash: string;
  height: number;
  tx_index: number;
}

/** A note envelope as the node publishes it: hex fields, public, opened only by a key. */
export interface EnvelopeHex {
  kem_ct: string;
  to_receiver: string;
  to_sender: string;
  body: string;
}

/** A call's sealed input transcript (`shrugg_getCallEnvelope`), hex fields. */
export interface CallEnvelopeHex {
  kem_ct: string;
  to_sender: string;
  to_auditor: string;
  body: string;
}

/** A tree leaf with its envelope; `envelope` is null for a leaf indexed before envelopes were stored. */
export interface NoteEnvelope {
  leaf_index: number;
  cm: string;
  height: number;
  tx_hash: string | null;
  envelope: EnvelopeHex | null;
}

/** Everything a key can be tried against for one transaction. */
export interface TransactionEnvelopes {
  hash: string;
  kind: TransactionKind;
  notes: NoteEnvelope[];
  h_in: string | null;
  call_envelope: CallEnvelopeHex | null;
}

/** A page of leaves with envelopes, oldest first. */
export interface NoteEnvelopePage {
  from_leaf: number;
  next_leaf: number | null;
  total_leaves: number;
  notes: NoteEnvelope[];
}

export interface BridgeAsset {
  index: number;
  chain: number;
  token: string;
  asset_id: string;
}

export interface BridgeState {
  enabled: boolean;
  emitter: string | null;
  /** source chain id -> the emitter address trusted there */
  emitters: Record<string, string>;
  guardian_set_index: number | null;
  guardians: string[];
  burn_sequence: number | null;
  next_index: number | null;
  assets: BridgeAsset[];
}

/**
 * A registry row with everything the chain has seen of the asset (`GET /bridge/assets`).
 * Amounts are decimal strings of bridge units: always 8 decimals, whatever the token's own
 * decimals on its home chain. `outstanding` is `deposited - burned`, the supply now held in
 * shielded notes.
 */
export interface BridgeAssetActivity extends BridgeAsset {
  /** set when (chain, token) is on the approved list */
  symbol: string | null;
  name: string | null;
  /** the token's decimals on its home chain */
  decimals: number | null;
  deposits: number;
  deposited: string;
  burns: number;
  burned: string;
  outstanding: string;
  first_height: number | null;
  last_height: number | null;
}

export type TokenStatus = 'allowed' | 'discontinued';

/** A token the bridge accepts, one entry per (token, home chain) (`GET /bridge/tokens`). */
export interface ApprovedToken {
  symbol: string;
  name: string;
  /** bridge chain id (2 Ethereum, 3 BSC, 4 Tron, 5 Solana) */
  chain: number;
  chain_name: string;
  /** ERC-20, BEP-20, TRC-20, SPL */
  standard: string;
  /** the contract address (or mint) as the chain's explorers print it */
  address: string;
  /** the same address as the registry stores it: 32 bytes hex */
  token: string;
  /** decimals on the home chain; on Rand every bridged amount has 8 */
  decimals: number;
  status: TokenStatus;
  explorer_url: string;
  note?: string;
}

/** The node's supply audit; every amount is a unit string. */
export interface Supply {
  height: number;
  genesis_deposited: string;
  genesis_staked: string;
  faucet_minted: string;
  withdraw_deposited: string;
  fees_paid: string;
  burned: string;
  pool_value: string;
  register_total: string;
  total_supply: string;
  invariant_holds: boolean;
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

export interface PendingStake {
  release_epoch: number;
  amount: string;
}

/** An entry of the public validator register. */
export interface Validator {
  address: string;
  /** Units of SHRUGG. */
  stake: string;
  /** Bundle fees credited as proposer, not yet withdrawn (units). */
  rewards: string;
  pending: PendingStake[];
  payout: string | null;
  nonce: number;
  /** In the set running the current epoch. */
  active: boolean;
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
  /** Leaves in the commitment tree. */
  notes: number;
  /** Nullifiers published. */
  nullifiers: number;
  validator_count: number;
  active_validator_count: number;
  total_stake: string;
  /** The supply audit's total, or "0" on a node without one. */
  total_supply: string;
  pool_value: string | null;
  program_count: number;
  avg_block_time_ms: number;
  peer_count: number;
  mempool_size: number;
  node_syncing: boolean;
  faucet: boolean;
  confidential: boolean;
  current_leader: string | null;
  tree_root: string | null;
  hc_bundle: string | null;
  epoch: number | null;
  epoch_blocks: number | null;
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
