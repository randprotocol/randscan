'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { KeyPanel, type SubmittedKey } from '@/components/KeyPanel';
import { RoleBadge, SpentCell, type SpentState } from '@/components/OpenedNotes';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { PageHeader } from '@/components/States';
import { useTokens } from '@/hooks/useApi';
import * as api from '@/lib/api';
import { formatAmount, formatTokenAmount, formatNumber, tokenBalances, type TokenBalance } from '@/lib/utils';
import { keyInfo, openNote, type KeyInfo, type OpenedNote } from '@/lib/viewing';
import type { TokenInfo } from '@/types';

interface HistoryRow {
  leaf_index: number;
  cm: string;
  height: number;
  tx_hash: string | null;
  note: OpenedNote;
  spent: SpentState;
}

/** `tokens` resolves a received/sent note's `asset` index to its symbol and decimals — the same
 * registry a wallet's own `resolve_asset` reads through, so a zUSD note renders "12.50 zUSD"
 * rather than a bare unit count. */
function buildColumns(tokens: TokenInfo[] | undefined): Column<HistoryRow>[] {
  return [
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
      render: (r) => <span className="font-mono text-text">{formatTokenAmount(r.note.amount, r.note.asset, tokens)}</span>,
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
}

function buildBalanceColumns(tokens: TokenInfo[] | undefined): Column<TokenBalance>[] {
  return [
    {
      key: 'token',
      header: 'Token',
      render: (b) =>
        b.asset === 0 ? (
          <span className="text-text">RAND</span>
        ) : b.token ? (
          <Link href={`/tokens/${b.token.id_text}`} className="link">
            {b.token.symbol} <span className="text-mute">{b.token.name}</span>
          </Link>
        ) : (
          <span className="text-mute">asset #{b.asset}</span>
        ),
    },
    {
      key: 'balance',
      header: 'Spendable balance',
      render: (b) => <span className="font-mono text-text">{formatTokenAmount(b.units.toString(), b.asset, tokens)}</span>,
    },
    { key: 'notes', header: 'Unspent notes', render: (b) => <span className="font-mono">{formatNumber(b.notes)}</span> },
  ];
}

export default function ViewingPage() {
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<{ scanned: number; total: number } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<KeyInfo | null>(null);
  const [rows, setRows] = useState<HistoryRow[] | null>(null);
  const { data: tokenList } = useTokens();
  const columns = buildColumns(tokenList?.tokens);

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
      // Every received note's nullifier in one request per thousand: a lookup that fails fails the
      // scan, since a note of unknown status would silently drop out of the balances.
      const owned = found.filter((r) => r.note.role === 'received' && r.note.nullifier);
      for (let i = 0; i < owned.length; i += api.NULLIFIER_LOOKUP_MAX) {
        const chunk = owned.slice(i, i + api.NULLIFIER_LOOKUP_MAX);
        const spent = new Map((await api.lookupNullifiers(chunk.map((r) => r.note.nullifier!))).map((nf) => [nf.nullifier, nf]));
        for (const r of chunk) {
          const nf = spent.get(r.note.nullifier!);
          r.spent = nf ? { state: 'spent', nullifier: nf } : { state: 'unspent' };
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
  const balances = tokenBalances(
    received.filter((r) => r.spent.state === 'unspent').map((r) => r.note),
    tokenList?.tokens
  );
  const balance = balances[0].units;

  return (
    <div className="space-y-6">
      <PageHeader
        title="My history"
        subtitle="Scan the commitment tree with your viewing key. The explorer never sees the key; every envelope is opened in your browser."
      />

      <KeyPanel kinds={['viewing']} onOpen={run} busy={busy} title="Scan with a viewing key">
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
            <StatsCard title="Spendable balance" value={formatAmount(balance.toString())} subtitle="unspent notes received, RAND; tokens below" />
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
          <DataTable columns={buildBalanceColumns(tokenList?.tokens)} data={balances} keyExtractor={(b) => String(b.asset)} />
          <DataTable columns={columns} data={rows} keyExtractor={(r) => r.cm} emptyMessage="This key opens no note on the chain" />
          <p className="text-xs text-mute">
            Every row was decrypted in this browser and its commitment recomputed from the plaintext. A change note appears as received.
          </p>
        </>
      )}
    </div>
  );
}
