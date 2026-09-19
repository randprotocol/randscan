'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { KindBadge } from '@/components/KindBadge';
import { TransactionOpener } from '@/components/TransactionOpener';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useTokens, useTransaction } from '@/hooks/useApi';
import {
  formatAmount,
  formatBridgeAddress,
  formatBridgeChain,
  formatBytes,
  formatDateTime,
  formatNumber,
  formatTimestamp,
  formatTokenAmount,
  getKindLabel,
  resolveToken,
} from '@/lib/utils';
import type { Bundle, Receipt, TokenInfo, TransactionDetail } from '@/types';

export default function TransactionDetailPage() {
  const params = useParams<{ hash: string }>();
  const hash = decodeURIComponent(params.hash);
  const { data: tx, error, isLoading, mutate } = useTransaction(hash);
  const { data: tokenList } = useTokens();
  const tokens = tokenList?.tokens ?? [];

  if (isLoading && !tx) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Transaction not found"
        message={`No transaction matches "${hash}". Only committed transactions are indexed.`}
        backHref="/transactions"
        backLabel="Back to transactions"
      />
    );
  }

  if (error || !tx) {
    return <ErrorState message="Could not load this transaction." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Transaction"
        subtitle={<Hash value={tx.hash} full copyable />}
        actions={<KindBadge kind={tx.kind} />}
      />

      <Panel title="Overview">
        <DetailRow label="Hash">
          <Hash value={tx.hash} full />
        </DetailRow>
        <DetailRow label="Kind">
          <KindBadge kind={tx.kind} />
        </DetailRow>
        <DetailRow label="Block">
          <span className="inline-flex flex-wrap items-center gap-2">
            <Link href={`/blocks/${tx.height}`} className="link font-mono">
              #{formatNumber(tx.height)}
            </Link>
            <span className="text-mute">·</span>
            <Hash value={tx.block_hash} href={`/blocks/${tx.block_hash}`} />
          </span>
        </DetailRow>
        <DetailRow label="Index in block">
          <span className="font-mono">{formatNumber(tx.tx_index)}</span>
        </DetailRow>
        <DetailRow label="Timestamp">
          <span>
            {formatDateTime(tx.timestamp_ms)}{' '}
            <span className="text-mute">({formatTimestamp(tx.timestamp_ms)})</span>
          </span>
        </DetailRow>
        <DetailRow label="Fee">
          {tx.has_bundle ? (
            formatAmount(tx.fee)
          ) : (
            <span className="text-mute">None (no bundle)</span>
          )}
        </DetailRow>
        <DetailRow label="Bundle">
          {tx.has_bundle ? (
            <span>Yes: paid from the sender&apos;s own notes, proved with a STARK</span>
          ) : (
            <span className="inline-flex flex-wrap items-center gap-2">
              <span>No: signed by validator or guardian quorum</span>
              {tx.validator ? (
                <Hash value={tx.validator} href={`/validators/${tx.validator}`} />
              ) : (
                <span className="text-mute">—</span>
              )}
            </span>
          )}
        </DetailRow>
        <DetailRow label="Chain ID">
          <span className="font-mono">{formatNumber(tx.chain_id)}</span>
        </DetailRow>
      </Panel>

      {tx.bundle && <BundlePanel bundle={tx.bundle} tokens={tokens} />}

      <KindPanel tx={tx} tokens={tokens} />

      {tx.kind === 'call' && <ReceiptPanel receipt={tx.receipt} />}

      {tx.kind !== 'other' && <TransactionOpener hash={tx.hash} kind={tx.kind} />}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Bundle
// ---------------------------------------------------------------------------

function BundlePanel({ bundle, tokens }: { bundle: Bundle; tokens: TokenInfo[] }) {
  const burned = bundle.burn_a !== '0' || bundle.burn_r !== '0';
  return (
    <Panel title="Bundle">
      <DetailRow label="Anchor">
        <Hash value={bundle.anchor} full />
      </DetailRow>
      {bundle.nullifiers.map((nf, i) => (
        <DetailRow key={`nf-${i}`} label={`Nullifier ${i + 1}${i < 2 ? ' (asset)' : ' (RAND)'}`}>
          <Hash value={nf} full />
        </DetailRow>
      ))}
      {bundle.commitments.map((cm, i) => (
        <DetailRow key={`cm-${i}`} label={`Commitment ${i + 1}${i < 2 ? ' (asset)' : ' (RAND)'}`}>
          <Hash value={cm} href={`/notes/${cm}`} full />
        </DetailRow>
      ))}
      <DetailRow label="Fee">{formatAmount(bundle.fee)}</DetailRow>
      <DetailRow label="Burned">
        {!burned ? (
          <span className="text-mute">Nothing burned</span>
        ) : (
          <span className="font-semibold text-strong">
            {bundle.burn_r !== '0' && <span>{formatAmount(bundle.burn_r)} (RAND)</span>}
            {bundle.burn_a !== '0' && (
              <span>{formatTokenAmount(bundle.burn_a, bundle.burn_asset, tokens)}</span>
            )}
          </span>
        )}
      </DetailRow>
      <DetailRow label="Time (target height)">
        <Link href={`/blocks/${bundle.time}`} className="link font-mono">
          #{formatNumber(bundle.time)}
        </Link>
      </DetailRow>
      <DetailRow label="Proof size">
        <span className="font-mono">{formatBytes(bundle.proof_len)}</span>
      </DetailRow>
      <DetailRow label="Envelope sizes">
        <span className="font-mono">
          {bundle.envelope_len.map((n) => formatBytes(n)).join(' · ')}
        </span>
      </DetailRow>
      <p className="px-4 py-3 text-xs text-mute">
        Four slots, dummies included: slots 1–2 carry a private asset (RAND or any RPL token) and
        slots 3–4 always RAND. There is no public asset field — a transfer of RAND and a transfer
        of any RPL token are the same shape, field for field. The proof and the envelopes are
        reported by size only.
      </p>
    </Panel>
  );
}

// ---------------------------------------------------------------------------
// Kind-specific detail
// ---------------------------------------------------------------------------

function ValidatorRow({ label, address }: { label: string; address: string | null }) {
  return (
    <DetailRow label={label}>
      {address ? (
        <Hash value={address} href={`/validators/${address}`} full />
      ) : (
        <span className="text-mute">—</span>
      )}
    </DetailRow>
  );
}

function PqSignersRow({ signers }: { signers: number[] | null }) {
  if (!signers || signers.length === 0) return null;
  return (
    <DetailRow label="PQ guardian co-signers">
      <span className="font-mono">{signers.map((i) => `#${i}`).join(', ')}</span>
    </DetailRow>
  );
}

/** A resolved token's symbol as a link to its page, or "asset #N" when this build's cached
 * registry does not (yet) know the index. */
function AssetLink({ index, tokens }: { index: number | null; tokens: TokenInfo[] }) {
  if (index === null) return <span className="text-mute">—</span>;
  if (index === 0) return <span className="font-mono">RAND</span>;
  const token = resolveToken(tokens, index);
  if (!token) return <span className="font-mono">asset #{index}</span>;
  return (
    <Link href={`/tokens/${token.index}`} className="link font-mono">
      {token.symbol} (#{index})
    </Link>
  );
}

function KindPanel({ tx, tokens }: { tx: TransactionDetail; tokens: TokenInfo[] }) {
  const title = `${getKindLabel(tx.kind)} details`;

  if (tx.kind === 'transfer') {
    return (
      <Panel title={title}>
        <p className="px-4 py-3 text-sm text-mute">
          A plain shielded transfer of RAND or of any RPL token — the asset is private, so this
          page cannot say which one moved. Up to two notes spent, up to two created, and the fee.
          Who paid whom, how much and in what asset is known only to the two parties and to
          whoever they hand a viewing key or a transaction key.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'mint') {
    return (
      <Panel title={title}>
        <DetailRow label="Deposit note">
          {tx.cm ? <Hash value={tx.cm} href={`/notes/${tx.cm}`} full /> : <span className="text-mute">—</span>}
        </DetailRow>
        <DetailRow label="Amount">
          <span className="text-base font-semibold text-strong">{formatAmount(tx.amount)}</span>
        </DetailRow>
        <ValidatorRow label="Minted by" address={tx.validator} />
        <p className="px-4 py-3 text-xs text-mute">
          A testnet faucet deposit. The amount is public in this one transaction; the note it
          created is indistinguishable from any other afterwards.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'deploy') {
    return (
      <Panel title={title}>
        <DetailRow label="Program">
          {tx.program ? (
            <Hash value={tx.program} href={`/programs/${tx.program}`} full />
          ) : (
            <span className="text-mute">—</span>
          )}
        </DetailRow>
        <DetailRow label="Words length">
          <span className="font-mono">
            {tx.words_len === null ? '—' : `${formatNumber(tx.words_len)} words`}
          </span>
        </DetailRow>
      </Panel>
    );
  }

  if (tx.kind === 'call') {
    return (
      <Panel title={title}>
        <DetailRow label="Program">
          {tx.program ? (
            <Hash value={tx.program} href={`/programs/${tx.program}`} full />
          ) : (
            <span className="text-mute">—</span>
          )}
        </DetailRow>
        <DetailRow label="Call proof size">
          <span className="font-mono">{formatBytes(tx.call_proof_len)}</span>
        </DetailRow>
        <DetailRow label="Input transcript">
          {tx.input_envelope_len === null ? (
            <span className="text-mute">None published (the caller chose --no-envelope)</span>
          ) : (
            <span className="font-mono">{formatBytes(tx.input_envelope_len)} sealed</span>
          )}
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          The call&apos;s private inputs never leave the wallet in the clear. When a transcript is
          published it opens only for the caller&apos;s viewing key, a per-call key, or the
          auditor the caller named.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'bond' || tx.kind === 'unbond' || tx.kind === 'withdraw') {
    return (
      <Panel title={title}>
        <ValidatorRow label="Validator" address={tx.validator} />
        <DetailRow label="Amount">
          <span className="text-base font-semibold text-strong">{formatAmount(tx.amount)}</span>
        </DetailRow>
        {tx.kind === 'bond' && (
          <DetailRow label="Registration">
            {tx.registered ? (
              <span className="badge badge-accent">New validator</span>
            ) : (
              <span className="text-mute">Existing validator</span>
            )}
          </DetailRow>
        )}
        {tx.kind !== 'bond' && (
          <DetailRow label="Register nonce">
            <span className="font-mono">
              {tx.action_nonce === null ? '—' : formatNumber(tx.action_nonce)}
            </span>
          </DetailRow>
        )}
        <p className="px-4 py-3 text-xs text-mute">
          {tx.kind === 'bond'
            ? 'Value left the pool (the bundle’s burn) into the validator’s public stake.'
            : tx.kind === 'unbond'
              ? 'Stake moved to the unbonding queue, signed by the validator’s key.'
              : 'Released stake and rewards became one new note of public amount at the validator’s payout address.'}
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'bridge_burn') {
    return (
      <Panel title={title}>
        <DetailRow label="Asset">
          <AssetLink index={tx.asset_index} tokens={tokens} />
        </DetailRow>
        <DetailRow label="Amount burned">
          <span className="text-base font-semibold text-strong">
            {formatTokenAmount(tx.amount, tx.asset_index, tokens)}
          </span>
        </DetailRow>
        <DetailRow label="Relayer fee">
          <span>{formatTokenAmount(tx.relayer_fee, tx.asset_index, tokens)}</span>
        </DetailRow>
        <DetailRow label="Destination chain">
          <span>{formatBridgeChain(tx.to_chain)}</span>
        </DetailRow>
        <DetailRow label="Destination address">
          {tx.bridge_to ? (
            <span className="font-mono break-all" title={tx.bridge_to}>
              {formatBridgeAddress(tx.bridge_to)}
            </span>
          ) : (
            <span className="text-mute">—</span>
          )}
        </DetailRow>
        <DetailRow label="Coin redeemed">
          {tx.bridge_token ? (
            <span className="font-mono break-all" title={tx.bridge_token}>
              {formatBridgeAddress(tx.bridge_token)}
            </span>
          ) : (
            <span className="text-mute">—</span>
          )}
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          One hidden-asset bundle carries the whole burn: its <code>burn_a</code> and{' '}
          <code>burn_asset</code> are this action&apos;s asset and amount, and its fee pays the
          bridge fee in RAND.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'bridge_attest') {
    return (
      <Panel title={title}>
        <DetailRow label="Attestation size">
          <span className="font-mono">{formatBytes(tx.attestation_len)}</span>
        </DetailRow>
        <DetailRow label="Recipient">
          {tx.recipient ? (
            <Hash value={tx.recipient} full />
          ) : (
            <span className="text-mute">—</span>
          )}
        </DetailRow>
        <DetailRow label="Asset">
          <AssetLink index={tx.asset_index} tokens={tokens} />
        </DetailRow>
        <DetailRow label="Amount">
          {tx.amount === null ? (
            <span className="text-mute">Guardian-set rotation, no deposit</span>
          ) : (
            <span className="text-base font-semibold text-strong">
              {formatTokenAmount(tx.amount, tx.asset_index, tokens)}
            </span>
          )}
        </DetailRow>
        <DetailRow label="Note time">
          {tx.note_time === null ? (
            <span className="text-mute">—</span>
          ) : (
            <Link href={`/blocks/${tx.note_time}`} className="link font-mono">
              #{formatNumber(tx.note_time)}
            </Link>
          )}
        </DetailRow>
        <DetailRow label="Deposit commitment">
          {tx.commitment ? (
            <Hash value={tx.commitment} href={`/notes/${tx.commitment}`} full />
          ) : (
            <span className="text-mute">— (rotation)</span>
          )}
        </DetailRow>
        <PqSignersRow signers={tx.pq_signers} />
        <p className="px-4 py-3 text-xs text-mute">
          A guardian-signed message that deposits a bridged asset as a note for the recipient.
          The amount is public here and nowhere else.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'register_token') {
    const action = tx.token_action?.kind === 'register_token' ? tx.token_action : null;
    return (
      <Panel title={title}>
        <DetailRow label="Token">
          <Link href={`/tokens/${tx.asset_index}`} className="link">
            {action?.symbol ?? `#${tx.asset_index}`}
          </Link>
        </DetailRow>
        <DetailRow label="Name">{action?.name ?? '—'}</DetailRow>
        <DetailRow label="Decimals">
          <span className="font-mono">{action ? formatNumber(action.decimals) : '—'}</span>
        </DetailRow>
        <DetailRow label="Authority">
          <span className="font-mono">{action?.authority ?? '—'}</span>
        </DetailRow>
        <DetailRow label="Registry index">
          <span className="font-mono">#{formatNumber(tx.asset_index)}</span>
        </DetailRow>
        <DetailRow label="Initial mint">
          {action?.initial ? (
            <span className="text-base font-semibold text-strong">
              {formatTokenAmount(action.initial.amount, tx.asset_index, tokens)} to{' '}
              <Hash value={action.initial.recipient} />
            </span>
          ) : (
            <span className="text-mute">None</span>
          )}
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          A token&apos;s registration and its initial mint are public by design, as a bridge
          deposit is (spec §4) — only a later transfer of the token&apos;s notes is shielded.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'token_mint') {
    const action = tx.token_action?.kind === 'token_mint' ? tx.token_action : null;
    return (
      <Panel title={title}>
        <DetailRow label="Asset">
          <AssetLink index={tx.asset_index} tokens={tokens} />
        </DetailRow>
        <DetailRow label="Amount">
          <span className="text-base font-semibold text-strong">
            {formatTokenAmount(tx.amount, tx.asset_index, tokens)}
          </span>
        </DetailRow>
        <DetailRow label="Recipient">
          {tx.recipient ? <Hash value={tx.recipient} full /> : <span className="text-mute">—</span>}
        </DetailRow>
        <DetailRow label="Mint nonce">
          <span className="font-mono">{tx.action_nonce === null ? '—' : formatNumber(tx.action_nonce)}</span>
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          Every word of the minted note is here (its sender is the chain&apos;s fixed mint tag),
          so the recipient rebuilds it with nothing decrypted, whatever envelope{action ? '' : ''}{' '}
          the minter published.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'set_authority') {
    const action = tx.token_action?.kind === 'set_authority' ? tx.token_action : null;
    return (
      <Panel title={title}>
        <DetailRow label="Asset">
          <AssetLink index={tx.asset_index} tokens={tokens} />
        </DetailRow>
        <DetailRow label="Nonce">
          <span className="font-mono">{tx.action_nonce === null ? '—' : formatNumber(tx.action_nonce)}</span>
        </DetailRow>
        <DetailRow label="New authority">
          {action?.new_authority ? (
            <Hash value={action.new_authority} href={`/validators/${action.new_authority}`} full />
          ) : (
            <span className="text-mute">None (renounced — this token can never be minted again)</span>
          )}
        </DetailRow>
      </Panel>
    );
  }

  if (tx.kind === 'token_burn') {
    return (
      <Panel title={title}>
        <DetailRow label="Asset">
          <AssetLink index={tx.asset_index} tokens={tokens} />
        </DetailRow>
        <DetailRow label="Amount burned">
          <span className="text-base font-semibold text-strong">
            {formatTokenAmount(tx.amount, tx.asset_index, tokens)}
          </span>
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          A holder burn: public by design, since it is what makes the token&apos;s total supply
          auditable. A transfer of the same token is a plain shielded transfer — its asset stays
          private.
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'pause_mints' || tx.kind === 'unpause_mints') {
    const action = tx.bridge_governance;
    const nonce = action && 'nonce' in action ? action.nonce : null;
    const signers = action && 'pq_signers' in action ? action.pq_signers : null;
    return (
      <Panel title={title}>
        <DetailRow label="Nonce">
          <span className="font-mono">{nonce === null ? '—' : formatNumber(nonce)}</span>
        </DetailRow>
        <PqSignersRow signers={signers ?? null} />
        <p className="px-4 py-3 text-xs text-mute">
          {tx.kind === 'pause_mints'
            ? 'While paused, every transfer attest is refused; burns and guardian-set rotations stay open. Signed by the genesis pause key alone.'
            : 'Lifting the pause needs the PQ guardian quorum — the pause key can never unpause by itself.'}
        </p>
      </Panel>
    );
  }

  if (tx.kind === 'register_bridged_token' || tx.kind === 'list_backing') {
    const action = tx.bridge_governance;
    return (
      <Panel title={title}>
        {action?.kind === 'register_bridged_token' && (
          <>
            <DetailRow label="Name">{action.name}</DetailRow>
            <DetailRow label="Symbol">{action.symbol}</DetailRow>
            <DetailRow label="Source chain">{formatBridgeChain(action.chain)}</DetailRow>
            <DetailRow label="Source token">
              <span className="font-mono break-all">{formatBridgeAddress(action.token)}</span>
            </DetailRow>
            <DetailRow label="Source decimals">
              <span className="font-mono">{formatNumber(action.decimals)}</span>
            </DetailRow>
          </>
        )}
        {action?.kind === 'list_backing' && (
          <>
            <DetailRow label="Token">
              <AssetLink index={action.token_index} tokens={tokens} />
            </DetailRow>
            <DetailRow label="Source chain">{formatBridgeChain(action.chain)}</DetailRow>
            <DetailRow label="Source token">
              <span className="font-mono break-all">{formatBridgeAddress(action.token)}</span>
            </DetailRow>
            <DetailRow label="Source decimals">
              <span className="font-mono">{formatNumber(action.decimals)}</span>
            </DetailRow>
          </>
        )}
        <PqSignersRow signers={tx.pq_signers} />
        <p className="px-4 py-3 text-xs text-mute">
          Authorised by the PQ guardian quorum, on a RAND fee bundle its submitter pays.
        </p>
      </Panel>
    );
  }

  // other
  return (
    <Panel title={title}>
      <p className="px-4 py-3 text-sm text-mute">
        This transaction kind is newer than this explorer build. It was indexed with its hash,
        bundle, fee and block; decoded fields will appear after the explorer is updated.
      </p>
    </Panel>
  );
}

// ---------------------------------------------------------------------------
// Receipt
// ---------------------------------------------------------------------------

function ReceiptPanel({ receipt }: { receipt: Receipt | null }) {
  if (!receipt) {
    return (
      <Panel title="Receipt">
        <div className="py-6 text-sm text-mute">
          No receipt has been indexed for this call.
        </div>
      </Panel>
    );
  }

  return (
    <Panel title="Receipt">
      <DetailRow label="Program">
        <Hash value={receipt.program} href={`/programs/${receipt.program}`} full />
      </DetailRow>
      <DetailRow label="Tier">
        <span className="font-mono">{formatNumber(receipt.tier)}</span>
      </DetailRow>
      <DetailRow label="Position">
        <span className="inline-flex flex-wrap items-center gap-2">
          <Link href={`/blocks/${receipt.height}`} className="link font-mono">
            #{formatNumber(receipt.height)}
          </Link>
          <span className="text-mute">· index {formatNumber(receipt.index)}</span>
        </span>
      </DetailRow>
      <DetailRow label="Input commitment (H_IN)">
        {receipt.h_in ? <Hash value={receipt.h_in} full /> : <span className="text-mute">—</span>}
      </DetailRow>
      <DetailRow label="Outputs">
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          {receipt.outputs.map((output, index) => (
            <div key={index} className="rounded border border-border bg-bg-soft px-3 py-2">
              <div className="text-[10px] font-medium uppercase tracking-wide text-mute">
                out {index}
              </div>
              <div className="truncate font-mono text-sm text-text" title={String(output)}>
                {formatNumber(output)}
              </div>
            </div>
          ))}
        </div>
      </DetailRow>
      <p className="px-4 py-3 text-xs text-mute">
        The eight public output words are recorded and nothing else moves: value moves only
        through the bundle that paid for the call. H_IN is a salted digest of the private inputs
        and discloses nothing on its own.
      </p>
    </Panel>
  );
}
