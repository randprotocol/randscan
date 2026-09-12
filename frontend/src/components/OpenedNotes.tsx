'use client';

import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailRow } from '@/components/States';
import { formatAssetAmount, formatNumber } from '@/lib/utils';
import type { OpenedCall, OpenedNote } from '@/lib/viewing';
import type { Nullifier } from '@/types';

export type SpentState = { state: 'unspent' } | { state: 'spent'; nullifier: Nullifier } | { state: 'unknown' };

export interface OpenedNoteRow {
  leaf_index: number;
  cm: string;
  height: number;
  /** null: the key does not open it; undefined: no envelope indexed. */
  opened: OpenedNote | null | undefined;
  spent?: SpentState;
}

export function RoleBadge({ role }: { role: OpenedNote['role'] }) {
  const cls = role === 'received' ? 'badge badge-mint' : role === 'sent' ? 'badge badge-transfer' : 'badge badge-neutral';
  const label = role === 'received' ? 'Received' : role === 'sent' ? 'Sent' : 'Opened with tx key';
  return <span className={cls}>{label}</span>;
}

export function SpentCell({ spent }: { spent?: SpentState }) {
  if (!spent || spent.state === 'unknown') return <span className="text-mute">—</span>;
  if (spent.state === 'unspent') return <span className="text-accent">unspent</span>;
  return (
    <span className="inline-flex flex-wrap items-center gap-1.5">
      <span className="text-soft">spent in</span>
      <Hash value={spent.nullifier.tx_hash} href={`/transactions/${spent.nullifier.tx_hash}`} start={8} end={6} />
      <span className="text-mute">(#{formatNumber(spent.nullifier.height)})</span>
    </span>
  );
}

/** One note of a transaction, as opened (or not) by the pasted key. */
export function OpenedNoteBlock({ row }: { row: OpenedNoteRow }) {
  const title = (
    <span className="inline-flex flex-wrap items-center gap-2">
      <Link href={`/notes/${row.cm}`} className="link font-mono">
        leaf #{formatNumber(row.leaf_index)}
      </Link>
      <Hash value={row.cm} start={10} end={6} />
    </span>
  );
  if (row.opened === undefined) {
    return (
      <div className="border-t border-border-soft py-3">
        {title}
        <p className="mt-1 text-sm text-mute">envelope not indexed yet</p>
      </div>
    );
  }
  if (row.opened === null) {
    return (
      <div className="border-t border-border-soft py-3">
        {title}
        <p className="mt-1 text-sm text-mute">not opened by this key</p>
      </div>
    );
  }
  const n = row.opened;
  return (
    <div className="border-t border-border-soft py-2">
      <div className="flex flex-wrap items-center justify-between gap-2 py-2">
        {title}
        <RoleBadge role={n.role} />
      </div>
      <DetailRow label="Amount">
        <span className="text-base font-semibold text-strong">{formatAssetAmount(n.amount, n.asset)}</span>
      </DetailRow>
      <DetailRow label="Owner (pk)">
        <Hash value={n.pk} full />
      </DetailRow>
      <DetailRow label="Created by (pk)">
        <Hash value={n.from} full />
      </DetailRow>
      <DetailRow label="Note time">
        <span className="font-mono">{formatNumber(n.time)}</span>
      </DetailRow>
      <DetailRow label="Commitment">
        <span className="text-accent">verified ✓ (recomputed from the plaintext)</span>
      </DetailRow>
      {n.nullifier && (
        <>
          <DetailRow label="Nullifier">
            <Hash value={n.nullifier} full />
          </DetailRow>
          <DetailRow label="Status">
            <SpentCell spent={row.spent} />
          </DetailRow>
        </>
      )}
    </div>
  );
}

/** A call's opened input transcript. */
export function OpenedCallBlock({ call, hIn }: { call: OpenedCall | null | undefined; hIn: string | null }) {
  if (call === undefined) {
    return <p className="border-t border-border-soft py-3 text-sm text-mute">This call published no input transcript.</p>;
  }
  if (call === null) {
    return <p className="border-t border-border-soft py-3 text-sm text-mute">The input transcript is not opened by this key.</p>;
  }
  const role = call.role === 'caller' ? 'Caller' : call.role === 'auditor' ? 'Auditor' : 'Opened with call key';
  return (
    <div className="border-t border-border-soft py-2">
      <div className="flex flex-wrap items-center justify-between gap-2 py-2">
        <span className="text-sm text-strong">Call inputs</span>
        <span className="badge badge-call">{role}</span>
      </div>
      <DetailRow label="Private inputs">
        <span className="font-mono break-all">[{call.inputs.join(', ')}]</span>
      </DetailRow>
      <DetailRow label="Salt">
        <span className="font-mono">[{call.salt.join(', ')}]</span>
      </DetailRow>
      <DetailRow label="H_IN">
        {call.faithful ? (
          <span className="text-accent">faithful to H_IN ✓ {hIn ? <Hash value={hIn} start={8} end={6} /> : null}</span>
        ) : (
          <span className="text-accent-3">does NOT match H_IN: the caller published a false transcript</span>
        )}
      </DetailRow>
    </div>
  );
}
