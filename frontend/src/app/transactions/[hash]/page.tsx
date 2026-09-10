'use client';

import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { KindBadge } from '@/components/KindBadge';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useTransaction } from '@/hooks/useApi';
import {
  formatAmount,
  formatBytes,
  formatDateTime,
  formatNumber,
  formatTimestamp,
  getKindLabel,
} from '@/lib/utils';
import type { Receipt, TransactionDetail } from '@/types';

export default function TransactionDetailPage({ params }: { params: { hash: string } }) {
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
        <DetailRow label="Sender">
          <Hash value={tx.sender} href={`/account/${tx.sender}`} full />
        </DetailRow>
        <DetailRow label="Nonce">
          <span className="font-mono">{formatNumber(tx.nonce)}</span>
        </DetailRow>
        <DetailRow label="Fee">{formatAmount(tx.fee)}</DetailRow>
        <DetailRow label="Chain ID">
          <span className="font-mono">{formatNumber(tx.chain_id)}</span>
        </DetailRow>
      </Panel>

      <KindPanel tx={tx} />

      {tx.kind === 'call' && <ReceiptPanel receipt={tx.receipt} />}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Kind-specific detail
// ---------------------------------------------------------------------------

function KindPanel({ tx }: { tx: TransactionDetail }) {
  const title = `${getKindLabel(tx.kind)} details`;

  if (tx.kind === 'transfer' || tx.kind === 'mint') {
    return (
      <Panel title={title}>
        <DetailRow label={tx.kind === 'mint' ? 'Minted to' : 'Recipient'}>
          {tx.to ? <Hash value={tx.to} href={`/account/${tx.to}`} full /> : <span className="text-mute">—</span>}
        </DetailRow>
        <DetailRow label="Amount">
          <span className="text-base font-semibold text-strong">{formatAmount(tx.amount)}</span>
        </DetailRow>
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
        <DetailRow label="Base PC">
          <span className="font-mono">{tx.base_pc === null ? '—' : formatNumber(tx.base_pc)}</span>
        </DetailRow>
        <DetailRow label="Words length">
          <span className="font-mono">
            {tx.words_len === null ? '—' : `${formatNumber(tx.words_len)} words`}
          </span>
        </DetailRow>
      </Panel>
    );
  }

  // call
  return (
    <Panel title={title}>
      <DetailRow label="Program">
        {tx.program ? (
          <Hash value={tx.program} href={`/programs/${tx.program}`} full />
        ) : (
          <span className="text-mute">—</span>
        )}
      </DetailRow>
      <DetailRow label="Proof size">
        <span className="font-mono">{formatBytes(tx.proof_len)}</span>
      </DetailRow>
      <DetailRow label={`Recipients (${tx.recipients.length})`}>
        {tx.recipients.length === 0 ? (
          <span className="text-mute">None declared</span>
        ) : (
          <ul className="space-y-1.5">
            {tx.recipients.map((recipient, index) => (
              <li key={`${recipient}-${index}`} className="flex items-center gap-2">
                <span className="w-6 flex-shrink-0 text-right font-mono text-xs text-mute">
                  {index}
                </span>
                <Hash value={recipient} href={`/account/${recipient}`} full />
              </li>
            ))}
          </ul>
        )}
      </DetailRow>
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
      <DetailRow label="Effect">
        {receipt.effect ? (
          <span className="inline-flex flex-wrap items-center gap-2">
            <Hash value={receipt.effect.to} href={`/account/${receipt.effect.to}`} />
            <span className="text-mute">received</span>
            <span className="font-semibold text-strong">{formatAmount(receipt.effect.amount)}</span>
          </span>
        ) : (
          <span className="text-mute">No effect</span>
        )}
      </DetailRow>
    </Panel>
  );
}
