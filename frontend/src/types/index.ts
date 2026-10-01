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
  | 'invoke'
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
  /**
   * Split authorisation (chain 17+): the commitment the bundle proof publishes and a second proof
   * over the spend key must match, hex. All zeros on a chain without an auth guest; `null` when
   * the node reported none.
   */
  auth_commit: string | null;
  /** The auth proof's size in bytes; 0 without an auth guest. */
  auth_proof_len: number;
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
  /**
   * The program's deploy-time public digest the proof was checked against. `null` means the
   * program has no public input and the proof was checked against the empty one's digest.
   */
  h_pub: string | null;
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

/** One program-state cell as an invoke read or wrote it (RPL-2): Word8 key and value, 64 hex
 * each; a written value of 64 zeros deletes the cell, a read of 64 zeros is of none. */
export interface ProgramCell {
  key: string;
  value: string;
}

/** One note an invoke paid out of its program's vault (`pays`) or minted of the program's own
 * token (`mints`): every word of the chain-computed note, `time` the bundle's, `cm` the leaf. */
export interface Payout {
  /** registry index of the asset (0 is RAND) */
  asset: number;
  /** decimal string in the asset's own units */
  amount: string;
  recipient: string;
  time: number;
  r: string;
  cm: string;
}

/** The state transition an invoke declared and the ledger applied (RPL-2). What came in is the
 * bundle's: `burn_r` RAND into the vault, `burn_a` of `burn_asset` what `inflow` names. */
export interface Transition {
  reads: ProgramCell[];
  writes: ProgramCell[];
  inflow: 'none' | 'deposit' | 'burn';
  pays: Payout[];
  mints: Payout[];
}

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
  /** invoke (RPL-2): the transition the proof vouched for and the ledger applied. */
  transition?: Transition | null;
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
  /** Audit v6 (a node after v0.6.7): under bridge rules v2, what this backing minted in the
   * rolling window and the window's length (`null` on a day-counter chain), and `mint_headroom`,
   * the largest deposit the caps admit to it right now — the per-backing cap less what it
   * minted, and no more than the registry-wide window has left. Absent on an older API. */
  minted_in_window?: string | null;
  mint_window_secs?: number | null;
  mint_headroom?: string | null;
}

/**
 * One source-chain endpoint the chain mints from: a row of `emitters` as its own chain prints
 * it. Derived by the explorer, not sent by the node. Since the 2026-09-30 endpoint redeploy
 * (chain 19) there are two generations of bridge contracts, and the 32-byte word alone cannot
 * be compared with what Etherscan or Tronscan shows.
 */
export interface BridgeEndpoint {
  /** bridge chain id (2 Ethereum, 3 BSC, 4 Tron, 5 Solana) */
  chain: number;
  chain_name: string | null;
  /** the emitter as the node serves it: 32 bytes, hex */
  emitter: string;
  /** `0x…`, Tron base58check or Solana base58; `null` when the word is not such an address */
  address: string | null;
  explorer_url: string | null;
  /** The lowest sequence a lock from this endpoint may carry to be minted (the replay floor). */
  min_inbound_sequence: number | null;
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
  /** Bridge rules v2: what the next PQ-set or pause-key rotation must carry. */
  rotation_nonce?: number | null;
  /** Bridge rules v2: the cap on what every backing of every token together may mint per window. */
  rules_v2?: {
    global_mint_cap_per_window: string;
    cap_window_secs: number;
    /** Audit v6: what every backing together minted in the window, and what it has left. */
    global_minted_in_window?: string | null;
    global_mint_headroom?: string | null;
  } | null;
  /** The genesis replay floor: source chain id -> the lowest sequence a lock may carry. */
  min_inbound_sequence?: Record<string, number> | null;
  assets: BridgeAsset[];
  /** `emitters`, readable; absent from an API older than chain 19's. */
  endpoints?: BridgeEndpoint[];
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
  /** How many backings (source coins) the token at `index` has. */
  backings: number;
  /**
   * This backing's deposits. `null` when the token has several backings: a deposit publishes the
   * token it minted, not the coin that was locked for it. Use `token_deposits` for the token.
   */
  deposits: number | null;
  deposited: string | null;
  /** Burns that redeemed this backing; a burn names its coin, so these are exact per row. */
  burns: number;
  burned: string;
  /** What this backing holds for the chain now: the registry's `locked`. */
  outstanding: string;
  /**
   * The whole token's tallies, identical on every row of the token. Sum them once per `index`,
   * never per row (see `bridgeTokenTotals`).
   */
  token_deposits: number;
  token_deposited: string;
  token_burns: number;
  token_burned: string;
  /** First and last height of any bridge activity of the token, not of this backing alone. */
  first_height: number | null;
  last_height: number | null;
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
  /** Audit v6 (a node after v0.6.7): under bridge rules v2, what this backing minted in the
   * rolling window and the window's length (`null` on a day-counter chain), and `mint_headroom`,
   * the largest deposit the caps admit to it right now — the per-backing cap less what it
   * minted, and no more than the registry-wide window has left. Absent on an older API. */
  minted_in_window?: string | null;
  mint_window_secs?: number | null;
  mint_headroom?: string | null;
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
  /** Genesis vesting (chain 17+); `null` on an older node, "0" on a chain without a vesting section. */
  vesting_issued: string | null;
  vesting_released: string | null;
  vesting_in_register: string | null;
  vesting_locked: string | null;
  /** RPL-2: RAND invokes paid out of program vaults (pool side) and what the vaults hold
   * (register side); "0" without the section, absent on an older API. */
  program_rand_out?: string | null;
  program_rand_held?: string | null;
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
  /** The length of the public input fixed at deploy (0 without one). */
  public_words_len: number;
  /** Its digest: what every call's proof is checked against (a receipt's `h_pub`). */
  public_digest: string | null;
  call_count: number;
  last_called_height: number | null;
  /** RPL-2: invokes that moved this program's state, and the last height one did. */
  invoke_count?: number;
  last_invoked_height?: number | null;
}

/** One row of a program's vault (RPL-2): what it holds of one asset, in that asset's units. */
export interface VaultRow {
  asset: number;
  amount: string;
}

/** A program's public state on a chain with the `program_state` section, live from the node. */
export interface ProgramState {
  vault: VaultRow[];
  /** the program's cells in key order, first page */
  cells: ProgramCell[];
  /** the last key served when more follow (`GET /programs/:id/cells?after=`), else null */
  cells_next: string | null;
}

export interface ProgramCellsPage {
  cells: ProgramCell[];
  next: string | null;
}

export interface ProgramDetail extends ProgramSummary {
  /** the latest calls and invokes */
  recent_calls: TransactionSummary[];
  /** null on a chain without a `program_state` section (and on an older API). */
  program_state?: ProgramState | null;
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
  /** The genesis auth guest (split authorisation, chain 17+); `null` without one. */
  hc_auth: string | null;
  /** The tip's live gas prices, refreshed every commit; `null` on a chain without a gas section. */
  gas_prices: GasPrices | null;
  epoch: number | null;
  epoch_blocks: number | null;
  /** Block 0's hash: a chain id alone does not tell two cuts apart. */
  genesis_hash: string | null;
  node_version: string | null;
  /** The commit the node was built from, `-dirty` appended when its tree was not clean. */
  node_git_sha: string | null;
  fri_profile: string | null;
  /** The chain's call limits; `null` on a node without `rand_getLimits`. */
  limits: ChainLimits | null;
  updated_at: string;
}

/**
 * What a wallet needs from the chain's genesis (`rand_getLimits`): the size caps (constants
 * before chain 13) and, since chains 16–18, the envelope format, the v0.6 switch, the auth guest
 * and the gas section. The fields past the five caps are `null`/`false` on a chain without the
 * setting and on a node too old to report it.
 */
export interface ChainLimits {
  max_program_words: number;
  max_proof_bytes: number;
  max_block_bytes: number;
  max_call_envelope_bytes: number;
  /** 0 means no program on this chain can have a public input. */
  max_program_public_words: number;
  /** Every note envelope's exact size: 1860 where notes carry an encrypted memo (chain 18). */
  envelope_bytes: number | null;
  /** The v0.6 validity rules as consensus rules (chains 16+). */
  hardening_v6: boolean;
  /** The auth guest the genesis pins (split authorisation, chain 17+), hex. */
  hc_auth: string | null;
  /** RAND units per gas, decimal string: the chain's section (`circuit`) or this node's policy (`header`). */
  gas_price: string | null;
  /** RAND units per KiB of call proof and input envelope, decimal string. */
  byte_price: string | null;
  /** `circuit` under a chain's own gas section, `header` under a node's policy, `null` with neither. */
  gas_metering: 'circuit' | 'header' | null;
  /** The flat gas every bundle proof declares (20 479 on chain 18); `null` without a section. */
  bundle_gas_limit: number | null;
  /** The dynamic controller's per-block step in basis points; `null` when prices never move. */
  adjust_bps: number | null;
  /** RPL-2 (a v0.6.8 node): the `program_state` section's invoke limits; `null` without it. */
  program_state?: {
    /** RAND units an invoke's fee floor gains per cell it creates, decimal string */
    cell_fee: string;
    max_reads: number;
    max_writes: number;
    max_payouts: number;
  } | null;
}

/** The tip's live gas prices under a chain's gas section, decimal strings. */
export interface GasPrices {
  gas_price: string;
  byte_price: string;
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
// Delegated provers
// ---------------------------------------------------------------------------

/** A delegated prover's public `prover_info` (the ML-KEM key left out). */
export interface ProverInfoReply {
  version: string | null;
  kem_fingerprint: string | null;
  hc_bundles: string[];
  /** `cpu` or `cuda` */
  backend: string | null;
  witness_kinds: string[];
  /** Behind a pool, the sums over the members that answered the router's last poll. */
  queue: { depth: number; max: number; proving: number } | null;
  /** `null` for a free prover; otherwise RAND base units per bundle and the address paid. */
  fee: { amount: string; address: string } | null;
}

/** A host that proves for a prover endpoint: its label and where it is (never its address). */
export interface ProverMember {
  label: string;
  geo: GeoInfo | null;
}

export interface ProverView {
  name: string;
  url: string;
  operator: string;
  /** The pairing fingerprint a wallet pins. */
  fingerprint: string;
  pairing_url: string | null;
  /** Whether the endpoint answered `prover_info` on the explorer's last poll. */
  up: boolean;
  info: ProverInfoReply | null;
  fingerprint_matches: boolean | null;
  error: string | null;
  checked_at_ms: number | null;
  last_up_ms: number | null;
  members: ProverMember[];
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
