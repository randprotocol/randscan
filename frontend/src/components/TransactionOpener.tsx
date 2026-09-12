'use client';

import { useState } from 'react';
import { KeyPanel, type SubmittedKey } from '@/components/KeyPanel';
import { OpenedCallBlock, OpenedNoteBlock, type OpenedNoteRow, type SpentState } from '@/components/OpenedNotes';
import * as api from '@/lib/api';
import { openCall, openNote, type OpenedCall, type OpenedNote } from '@/lib/viewing';
import type { TransactionKind } from '@/types';

async function spentState(nullifier: string): Promise<SpentState> {
  try {
    const nf = await api.getNullifier(nullifier);
    return { state: 'spent', nullifier: nf };
  } catch (e) {
    return api.isNotFoundError(e) ? { state: 'unspent' } : { state: 'unknown' };
  }
}

/** The "Open with a key" panel of a transaction page. */
export function TransactionOpener({ hash, kind }: { hash: string; kind: TransactionKind }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rows, setRows] = useState<OpenedNoteRow[] | null>(null);
  const [call, setCall] = useState<OpenedCall | null | undefined>(undefined);
  const [hIn, setHIn] = useState<string | null>(null);
  const [hasCallEnvelope, setHasCallEnvelope] = useState(false);

  const run = async ({ kind: keyKind, key }: SubmittedKey) => {
    setBusy(true);
    setError(null);
    try {
      const env = await api.getTransactionEnvelopes(hash);
      const out: OpenedNoteRow[] = [];
      for (const n of env.notes) {
        let opened: OpenedNote | null | undefined;
        if (!n.envelope) {
          opened = undefined;
        } else if (keyKind === 'call') {
          opened = null;
        } else {
          opened = await openNote(n.cm, n.envelope, keyKind, key);
        }
        const row: OpenedNoteRow = { leaf_index: n.leaf_index, cm: n.cm, height: n.height, opened };
        if (opened && opened.nullifier) row.spent = await spentState(opened.nullifier);
        out.push(row);
      }
      setRows(out);
      setHIn(env.h_in);
      setHasCallEnvelope(!!env.call_envelope);
      if (env.kind === 'call' && env.call_envelope && env.h_in) {
        const callKind = keyKind === 'tx' ? 'call' : keyKind;
        setCall(await openCall(env.h_in, env.call_envelope, callKind, key));
      } else {
        setCall(undefined);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setRows(null);
    } finally {
      setBusy(false);
    }
  };

  const kinds: SubmittedKey['kind'][] = kind === 'call' ? ['viewing', 'spend', 'tx', 'call'] : ['viewing', 'spend', 'tx'];

  return (
    <KeyPanel kinds={kinds.filter((k) => k !== 'file') as ('viewing' | 'spend' | 'tx' | 'call')[]} onOpen={run} busy={busy}>
      {error && <p className="border-t border-border-soft py-3 text-sm text-accent-3">{error}</p>}
      {rows && rows.length === 0 && (
        <p className="border-t border-border-soft py-3 text-sm text-mute">
          This transaction created no notes with an envelope (nothing a key could open).
        </p>
      )}
      {rows?.map((row) => (
        <OpenedNoteBlock key={row.cm} row={row} />
      ))}
      {rows && kind === 'call' && (hasCallEnvelope ? <OpenedCallBlock call={call} hIn={hIn} /> : <OpenedCallBlock call={undefined} hIn={hIn} />)}
    </KeyPanel>
  );
}
