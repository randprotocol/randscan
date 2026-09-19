// RandScan API types — mirrors the `/api/v1` contract served by randscan-api.
// All amounts are decimal strings of smallest units (RAND has 9 decimals).
// All timestamps are `timestamp_ms`: milliseconds since the Unix epoch.

/**
 * `other` is any kind the node serves that this explorer build does not decode.
 *
 * Chain 14 (the hidden-asset bundle, RPL tokens, bridge hardening): there is no
 * `token_transfer` — a transfer of any asset (RAND or an RPL token) is `transfer`
 * (the node's `none`), indistinguishable on the public page.
 */
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
  | 'register_token'
  | 'token_mint'
  | 'set_authority'
  | 'token_burn'
  | 'pause_mints'
  | 'unpause_mints'
  | 'register_bridged_token'
  | 'list_backing'
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

/**
 * The public fields of the chain-14 hidden-asset bundle: four input slots and four output slots
 * (dummies included). **There is no `asset` field** — slots 0-1 carry a private asset (RAND or
 * any RPL token) and slots 2-3 always RAND, so nothing public says which asset slots 0-1 moved.
 * A transfer of RAND and a transfer of any RPL token are the same shape, field for field.
 * `burn_a`/`burn_r`/`burn_asset` are the bundle's only public statement about value leaving the
 * pool: `burn_a`/`burn_asset` non-zero only on a `token_burn` or a `bridge_burn`; `burn_r`
 * (RAND) non-zero only on a `bond` or a `register_aggregator`.
 */
export interface Bundle {
  anchor: string;
  nullifiers: [string, string, string, string];
  commitments: [string, string, string, string];
  /** Units of RAND. */
  fee: string;
  /** The private asset burned, in that asset's own smallest unit. */
  burn_a: string;
  /** RAND burned. */
  burn_r: string;
  /** 0 when nothing was burned; otherwise the registry index `burn_a` names. */
  burn_asset: number;
  /** Block height the sender targeted. */
  time: number;
  proof_len: number;
  envelope_len: [number, number, number, number];
}

/**
 * There is no sender, recipient or nonce: a shielded transaction has none. `amount` is the one
 * public amount an action carries (a deposit's note value, a staking move, or an RPL
 * registration/mint/burn); `asset_index` is the token registry index it names — the key to
 * resolve a symbol through `/api/v1/tokens`. `null` for a plain `transfer`: its asset is private.
 */
export interface TransactionSummary {
  hash: string;
  height: number;
  block_hash: string;
  tx_index: number;
  kind: TransactionKind;
  /** RAND fee paid by the bundle ("0" for a validator-signed or bundle-less action). */
  fee: string;
  timestamp_ms: number;
  /** False for mint / unbond / withdraw / pause_mints / unpause_mints / register_bridged_token /
   * list_backing, which carry no bundle. */
  has_bundle: boolean;
  /** deploy: deployed program id; call: called program id */
  program: string | null;
  /** bond / unbond / withdraw: the validator; mint: the minting validator */
  validator: string | null;
  /** mint, bond, unbond, withdraw: RAND units; bridge_attest, bridge_burn, token_mint,
   * token_burn: the named asset's own units */
  amount: string | null;
  /** the token registry index this action names (see the interface doc) */
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

/** `register_token`'s optional initial mint: every word of the note the chain computes for it. */
export interface InitialMint {
  amount: string;
  recipient: string;
  time: number;
  r: string;
}

/** The bare authority tag `tx_json` renders on an action (contrast `TokenAuthority`, the full
 * form `/api/v1/tokens` serves). */
export type AuthorityKind = 'none' | 'key' | 'bridge' | 'program';

/** The RPL token actions' public fields (spec §4/§6): a token's registration and mints are
 * public by design, as a bridge deposit is — only a later transfer of the token's notes is
 * shielded. */
export type TokenAction =
  | {
      kind: 'register_token';
      name: string;
      symbol: string;
      decimals: number;
      authority: AuthorityKind;
      /** The registry index this registration was given. */
      index: number;
      initial_amount: string | null;
      initial: InitialMint | null;
    }
  | { kind: 'token_mint'; asset: number; amount: string; recipient: string; time: number; r: string; nonce: number }
  | { kind: 'set_authority'; asset: number; nonce: number; new_authority: string | null }
  | { kind: 'token_burn'; asset: number; amount: string };

/** Bridge hardening B1/B4's governance actions: bundle-less, and — apart from the pause key's
 * own `pause_mints` — authorised by the PQ guardian quorum, like a `bridge_attest`. */
export type BridgeGovernanceAction =
  | { kind: 'pause_mints'; nonce: number }
  | { kind: 'unpause_mints'; nonce: number; pq_signers: number[] }
  | {
      kind: 'register_bridged_token';
      name: string;
      symbol: string;
      salt: string;
      chain: number;
      token: string;
      decimals: number;
      nonce: number;
      asset_id: string;
      pq_signers: number[];
    }
  | {
      kind: 'list_backing';
      token_index: number;
      chain: number;
      token: string;
      decimals: number;
      nonce: number;
      pq_signers: number[];
    };

export interface TransactionDetail extends TransactionSummary {
  chain_id: number;
  /** The bundle; null for a bundle-less signed action. */
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
  /** unbond / withdraw / token_mint / set_authority: the nonce it signed */
  action_nonce: number | null;
  /** bridge_attest: size of the guardian-signed message */
  attestation_len: number | null;
  /** bridge_attest / token_mint / register_token (initial mint): the recipient's shielded
   * address (rand1…) */
  recipient: string | null;
  /** bridge_attest / token_mint / register_token (initial mint): the deposit/minted note's
   * time word */
  note_time: number | null;
  /** bridge_burn: relayer fee in bridged units */
  relayer_fee: string | null;
  /** bridge_burn: destination chain id (1 Rand, 2 Ethereum, 3 BSC, 4 Tron, 5 Solana) */
  to_chain: number | null;
  /** bridge_burn: destination address, 32 bytes hex */
  bridge_to: string | null;
  /** bridge_burn: the coin being redeemed (a backing's source-chain token address, 32 bytes hex) */
  bridge_token: string | null;
  /** bridge_attest / token_mint / register_token (initial mint): the note's blinding, hex —
   * public, and with the fields above every word of the note, so it can be rebuilt with nothing
   * decrypted. */
  deposit_r: string | null;
  /** bridge_attest only: the leaf the chain appended for the deposit. `null` for a rotation and
   * for every other kind — a token_mint's or register_token's note commitment is not published
   * directly; the explorer resolves it by finding the note among the transaction's own notes
   * (`GET /transactions/:hash/envelopes`). */
  commitment: string | null;
  /** bridge_attest / unpause_mints / register_bridged_token / list_backing: the PQ guardians who
   * co-signed, by index. */
  pq_signers: number[] | null;
  /** register_token / token_mint / set_authority / token_burn. */
  token_action: TokenAction | null;
  /** pause_mints / unpause_mints / register_bridged_token / list_backing. */
  bridge_governance: BridgeGovernanceAction | null;
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

/** A call's sealed input transcript (`rand_getCallEnvelope`), hex fields. */
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

/**
 * `decimals`/`locked`/`minted_today`/`mint_day` are bridge hardening B1 — one row per *backing*
 * (a token with several source coins lists once per backing, all sharing `index`); `null` on a
 * node predating B1.
 */
export interface BridgeAsset {
  index: number;
  chain: number;
  token: string;
  asset_id: string;
  decimals: number | null;
  locked: string | null;
  minted_today: string | null;
  mint_day: number | null;
}

/**
 * `mint_paused`/`pause_nonce`/`list_nonce`/`pause_key`/`pq_guardians`/`registration_fee` are
 * bridge hardening B1/B3/B4 — defaulted/empty on a node predating them.
 */
export interface BridgeState {
  enabled: boolean;
  emitter: string | null;
  /** source chain id -> the emitter address trusted there */
  emitters: Record<string, string>;
  guardian_set_index: number | null;
  guardians: string[];
  /** The genesis PQ (Dilithium2) guardian set, hex. Never moves on a rotation. */
  pq_guardians: string[];
  /** B1: while true, every transfer attest is refused (burns and rotations stay open). */
  mint_paused: boolean;
  /** B1: what the next pause_mints / unpause_mints must carry. */
  pause_nonce: number | null;
  /** B4: what the next list_backing / register_bridged_token must carry. */
  list_nonce: number | null;
  /** B1: the one Dilithium2 key that may pause minting (it can never unpause), hex. */
  pause_key: string | null;
  /** B4: what a register_bridged_token owes past the bundle base. A number, not a decimal
   * string. */
  registration_fee: number | null;
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
  deposits: number;
  deposited: string;
  burns: number;
  burned: string;
  outstanding: string;
  first_height: number | null;
  last_height: number | null;
  /** This backing's own figures straight from the registry (bridge hardening B1); `outstanding`
   * above is the indexer's own reconciliation from indexed flows — the two should agree. */
  mint_cap_per_day: string | null;
}

// ---------------------------------------------------------------------------
// RPL token registry
// ---------------------------------------------------------------------------

/** A source-chain coin behind a bridged token: its chain, its source token address, its
 * **source** decimals (the token itself is always eight decimals on Rand) and what this backing
 * has locked/minted through it. */
export interface TokenBacking {
  chain: number;
  token: string;
  decimals: number;
  locked: string;
  minted_today: string | null;
  mint_day: number | null;
  mint_cap_per_day: string | null;
}

/** A token's mint authority in full (contrast `AuthorityKind`, the bare tag on an action). */
export type TokenAuthority =
  | { kind: 'none' }
  | { kind: 'key'; key: string; address: string }
  | { kind: 'bridge'; backings: TokenBacking[] }
  | { kind: 'program'; program: string };

/** One registry row (`GET /api/v1/tokens` lists them, `GET /api/v1/tokens/:id` returns one).
 * `index` is the `asset` word a note of this token carries — 0 is RAND and never appears here.
 * `id_text` is the checksummed `rpl1…` text form. */
export interface TokenInfo {
  index: number;
  id: string;
  id_text: string;
  name: string;
  symbol: string;
  decimals: number;
  authority: TokenAuthority;
  mint_nonce: number;
  total_supply: string;
  registered_at: number;
}

export interface TokenList {
  enabled: boolean;
  registration_fee: number | null;
  next_index: number | null;
  tokens: TokenInfo[];
}

export interface TokenSupply {
  total_supply: string;
  backings: TokenBacking[];
}

/** One point of a token's public supply history — a register_token (its initial mint),
 * token_mint or token_burn naming it. `delta` is signed: positive for a mint, negative for a
 * burn, in the token's own smallest unit. */
export interface TokenSupplyEvent {
  tx_hash: string;
  height: number;
  timestamp_ms: number;
  kind: 'register_token' | 'token_mint' | 'token_burn';
  delta: string;
}

/** `GET /api/v1/tokens/:id`: the registry row plus its deploy transaction and supply history. */
export interface TokenDetail extends TokenInfo {
  deploy_tx: string | null;
  /** Oldest first. */
  supply_history: TokenSupplyEvent[];
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
  /** Units of RAND. */
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
