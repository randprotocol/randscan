'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { KindBadge } from '@/components/KindBadge';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useTransaction } from '@/hooks/useApi';
import {
  formatAmount,
  formatAssetAmount,
  formatBridgeAddress,
  formatBridgeChain,
  formatBytes,
  formatDateTime,
  formatNumber,
  formatTimestamp,
  getKindLabel,
} from '@/lib/utils';
import type { Bundle, Receipt, TransactionDetail } from '@/types';

export default function TransactionDetailPage() {
  const params = useParams<{ hash: string }>();
  const hash = decodeURIComponent(params.hash);
  const { data: tx, error, isLoading, mutate } = useTransaction(hash);

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
              <span>No: signed by validator</span>
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

      {tx.bundle && <BundlePanel title="Bundle" bundle={tx.bundle} />}

      <KindPanel tx={tx} />

      {tx.kind === 'bridge_burn' && tx.asset_bundle && (
        <BundlePanel title="Asset bundle" bundle={tx.asset_bundle} asset />
      )}

      {tx.kind === 'call' && <ReceiptPanel receipt={tx.receipt} />}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Bundle
// ---------------------------------------------------------------------------

function BundlePanel({ title, bundle, asset }: { title: string; bundle: Bundle; asset?: boolean }) {
  return (
    <Panel title={title}>
      <DetailRow label="Anchor">
        <Hash value={bundle.anchor} full />
      </DetailRow>
      {bundle.nullifiers.map((nf, i) => (
        <DetailRow key={`nf-${i}`} label={`Nullifier ${i + 1}`}>
          <Hash value={nf} full />
        </DetailRow>
      ))}
      {bundle.commitments.map((cm, i) => (
        <DetailRow key={`cm-${i}`} label={`Commitment ${i + 1}`}>
          <Hash value={cm} href={`/notes/${cm}`} full />
        </DetailRow>
      ))}
      <DetailRow label="Fee">
        {asset ? (
          <span className="text-mute">0 (the fee bundle pays)</span>
        ) : (
          formatAmount(bundle.fee)
        )}
      </DetailRow>
      <DetailRow label="Burn">
        {bundle.burn === '0' ? (
          <span className="text-mute">0</span>
        ) : (
          <span className="font-semibold text-strong">
            {formatAssetAmount(bundle.burn, bundle.asset)}
          </span>
        )}
      </DetailRow>
      <DetailRow label="Asset">
        <span className="font-mono">
          {bundle.asset === 0 ? '0 (SHRUGG)' : `#${formatNumber(bundle.asset)} (bridged)`}
        </span>
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
          {formatBytes(bundle.envelope_len[0])} · {formatBytes(bundle.envelope_len[1])}
        </span>
      </DetailRow>
      <p className="px-4 py-3 text-xs text-mute">
        The proof and the two note envelopes are reported by size only. Nothing in a bundle
        reveals the sender, the receiver or the amount; a dummy input or output looks like a real
        one.
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

function KindPanel({ tx }: { tx: TransactionDetail }) {
  const title = `${getKindLabel(tx.kind)} details`;

  if (tx.kind === 'transfer') {
    return (
      <Panel title={title}>
        <p className="px-4 py-3 text-sm text-mute">
          A plain shielded transfer: up to two notes spent, up to two created, and the fee. Who
          paid whom, and how much, is known only to the two parties and to whoever they hand a
          viewing key.
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
          <span className="font-mono">
            {tx.asset_index === null ? '—' : `#${formatNumber(tx.asset_index)}`}
          </span>
        </DetailRow>
        <DetailRow label="Amount burned">
          <span className="text-base font-semibold text-strong">
            {formatAssetAmount(tx.amount, tx.asset_index)}
          </span>
        </DetailRow>
        <DetailRow label="Relayer fee">
          <span>{formatAssetAmount(tx.relayer_fee, tx.asset_index)}</span>
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
        <p className="px-4 py-3 text-xs text-mute">
          The one two-bundle transaction: the fee bundle above pays in SHRUGG and the asset bundle
          below burns the bridged notes. Bridged amounts are in the asset&apos;s own smallest unit.
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
          <span className="font-mono">
            {tx.asset_index === null ? '—' : `#${formatNumber(tx.asset_index)}`}
          </span>
        </DetailRow>
        <DetailRow label="Amount">
          {tx.amount === null ? (
            <span className="text-mute">Guardian-set rotation, no deposit</span>
          ) : (
            <span className="text-base font-semibold text-strong">
              {formatAssetAmount(tx.amount, tx.asset_index)}
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
        <p className="px-4 py-3 text-xs text-mute">
          A guardian-signed message that deposits a bridged asset as a note for the recipient.
          The amount is public here and nowhere else.
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
