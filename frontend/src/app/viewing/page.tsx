'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { KeyPanel, type SubmittedKey } from '@/components/KeyPanel';
import { RoleBadge, SpentCell, type SpentState } from '@/components/OpenedNotes';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { PageHeader } from '@/components/States';
import * as api from '@/lib/api';
import { formatAmount, formatAssetAmount, formatNumber } from '@/lib/utils';
import { keyInfo, openNote, type KeyInfo, type OpenedNote } from '@/lib/viewing';

interface HistoryRow {
  leaf_index: number;
  cm: string;
  height: number;
  tx_hash: string | null;
  note: OpenedNote;
  spent: SpentState;
}

const columns: Column<HistoryRow>[] = [
  {
    key: 'leaf',
    header: 'Leaf',
    render: (r) => (
      <Link href={`/notes/${r.cm}`} className="link font-mono">
        #{formatNumber(r.leaf_index)}
      </Link>
    ),
  },
  {
    key: 'height',
    header: 'Height',
    render: (r) => (
      <Link href={`/blocks/${r.height}`} className="link font-mono">
        {formatNumber(r.height)}
      </Link>
    ),
  },
  {
    key: 'tx',
    header: 'Transaction',
    render: (r) =>
      r.tx_hash ? <Hash value={r.tx_hash} href={`/transactions/${r.tx_hash}`} /> : <span className="text-mute">{r.height === 0 ? 'genesis' : 'deposit'}</span>,
  },
  { key: 'role', header: 'Role', render: (r) => <RoleBadge role={r.note.role} /> },
  {
    key: 'amount',
    header: 'Amount',
    render: (r) => <span className="font-mono text-text">{formatAssetAmount(r.note.amount, r.note.asset)}</span>,
  },
  {
    key: 'counterparty',
    header: 'Counterparty (pk)',
    render: (r) => <Hash value={r.note.role === 'received' ? r.note.from : r.note.pk} start={8} end={6} />,
  },
  {
    key: 'status',
    header: 'Status',
    render: (r) => (r.note.role === 'received' ? <SpentCell spent={r.spent} /> : <span className="text-mute">—</span>),
  },
];

export default function ViewingPage() {
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<{ scanned: number; total: number } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<KeyInfo | null>(null);
  const [rows, setRows] = useState<HistoryRow[] | null>(null);

  const run = async ({ kind, key }: SubmittedKey) => {
    setBusy(true);
    setError(null);
    setRows(null);
    try {
      const ki = await keyInfo(key, kind);
      setInfo(ki);
      const found: HistoryRow[] = [];
      let from = 0;
      let scanned = 0;
      for (;;) {
        const page = await api.getEnvelopePage(from, 1000);
        for (const n of page.notes) {
          scanned += 1;
          if (!n.envelope) continue;
          const opened = await openNote(n.cm, n.envelope, kind, key);
          if (opened) {
            found.push({ leaf_index: n.leaf_index, cm: n.cm, height: n.height, tx_hash: n.tx_hash, note: opened, spent: { state: 'unknown' } });
          }
        }
        setProgress({ scanned, total: page.total_leaves });
        if (page.next_leaf === null) break;
        from = page.next_leaf;
      }
      for (const r of found) {
        if (r.note.role === 'received' && r.note.nullifier) {
          try {
            const nf = await api.getNullifier(r.note.nullifier);
            r.spent = { state: 'spent', nullifier: nf };
          } catch (e) {
            r.spent = api.isNotFoundError(e) ? { state: 'unspent' } : { state: 'unknown' };
          }
        }
      }
      setRows(found.sort((a, b) => b.leaf_index - a.leaf_index));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const received = rows?.filter((r) => r.note.role === 'received') ?? [];
  const balance = received
    .filter((r) => r.spent.state === 'unspent' && r.note.asset === 0)
    .reduce((sum, r) => sum + BigInt(r.note.amount), BigInt(0));

  return (
    <div className="space-y-6">
      <PageHeader
        title="My history"
        subtitle="Scan the commitment tree with your viewing key. The explorer never sees the key; every envelope is opened in your browser."
      />

      <KeyPanel kinds={['viewing', 'spend']} onOpen={run} busy={busy} title="Scan with a viewing key">
        {progress && (
          <p className="border-t border-border-soft py-3 text-sm text-mute">
            scanned {formatNumber(progress.scanned)} of {formatNumber(progress.total)} leaves
            {busy ? '…' : ''}
          </p>
        )}
        {error && <p className="border-t border-border-soft py-3 text-sm text-accent-3">{error}</p>}
      </KeyPanel>

      {info && rows && (
        <>
          <StatsRow columns={4}>
            <StatsCard title="Spendable balance" value={formatAmount(balance.toString())} subtitle="unspent notes received, SHRUGG only" />
            <StatsCard title="Notes received" value={formatNumber(received.length)} subtitle={`${formatNumber(received.filter((r) => r.spent.state === 'unspent').length)} unspent`} />
            <StatsCard title="Notes sent" value={formatNumber(rows.filter((r) => r.note.role === 'sent').length)} />
            <StatsCard
              title="Address"
              value={info.address ? `${info.address.slice(0, 14)}…${info.address.slice(-6)}` : '—'}
              subtitle={info.pk ? `pk ${info.pk.slice(0, 12)}…` : undefined}
            />
          </StatsRow>
          {info.address && (
            <p className="text-xs text-mute">
              Shielded address: <Hash value={info.address} start={20} end={10} />
            </p>
          )}
          <DataTable columns={columns} data={rows} keyExtractor={(r) => r.cm} emptyMessage="This key opens no note on the chain" />
          <p className="text-xs text-mute">
            Every row was decrypted in this browser and its commitment recomputed from the plaintext. A change note appears as received.
          </p>
        </>
      )}
    </div>
  );
}
