'use client';

import { useEffect, useState } from 'react';
import { Panel } from '@/components/States';
import { detectKeyKind, forgetKey, looksLikeKey, recallKey, rememberKey, type KeyKind } from '@/lib/viewing';

export interface SubmittedKey {
  kind: KeyKind;
  key: string;
}

interface KeyPanelProps {
  /** Which readings of a 64-hex key to offer. */
  kinds?: Exclude<KeyKind, 'file'>[];
  onOpen: (key: SubmittedKey) => void;
  busy?: boolean;
  /** What the parent renders once it has tried the key. */
  children?: React.ReactNode;
  title?: string;
}

const LABELS: Record<Exclude<KeyKind, 'file'>, string> = {
  viewing: 'Viewing key (nk)',
  spend: 'Spend key',
  tx: 'Transaction key',
  call: 'Call key',
};

/**
 * A key pasted here is handed to WebAssembly in this page and nowhere else: no request carries
 * it, and a spend key is reduced to its viewing key locally before anything is opened.
 */
export function KeyPanel({
  kinds = ['viewing', 'spend', 'tx'],
  onOpen,
  busy,
  children,
  title = 'Open with a key',
}: KeyPanelProps) {
  const [input, setInput] = useState('');
  const [chosen, setChosen] = useState<Exclude<KeyKind, 'file'>>(kinds[0]);
  const [remember, setRemember] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const saved = recallKey();
    if (saved) {
      setInput(saved.key);
      if (saved.kind !== 'file' && kinds.includes(saved.kind)) setChosen(saved.kind);
      setRemember(true);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const kind = detectKeyKind(input, chosen);
  const secret = kind === 'spend' || kind === 'file';

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const key = input.trim();
    if (!looksLikeKey(key)) {
      setError('A key is 64 hex characters, or the contents of a wallet key file (wallet.key.json).');
      return;
    }
    setError(null);
    if (remember) rememberKey(kind, key);
    else forgetKey();
    onOpen({ kind, key });
  };

  return (
    <Panel title={title}>
      <p className="py-3 text-sm text-soft">
        Decryption runs in your browser with WebAssembly; the key never leaves this page. A
        viewing key opens every note you sent or received, a transaction key opens one
        transaction, a call key opens one call&apos;s inputs. The wallet key file
        (<code className="font-mono text-xs">wallet.key.json</code>) is accepted too and only its
        derived viewing key is used.
      </p>
      <form onSubmit={submit} className="space-y-3 pb-3">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          rows={3}
          spellCheck={false}
          autoComplete="off"
          placeholder="64 hex characters, or paste wallet.key.json"
          className="w-full rounded border border-border bg-surface px-3 py-2 font-mono text-xs text-strong focus:border-accent focus:outline-none"
        />
        <div className="flex flex-wrap items-center gap-4 text-sm">
          {kinds.map((k) => (
            <label key={k} className="inline-flex items-center gap-1.5 text-soft">
              <input
                type="radio"
                name="key-kind"
                value={k}
                checked={chosen === k}
                disabled={kind === 'file'}
                onChange={() => setChosen(k)}
              />
              {LABELS[k]}
            </label>
          ))}
          {kind === 'file' && <span className="text-mute">key file detected</span>}
        </div>
        <div className="flex flex-wrap items-center gap-4">
          <button type="submit" disabled={busy || input.trim() === ''} className="btn-primary">
            {busy ? 'Opening…' : 'Open'}
          </button>
          <label className="inline-flex items-center gap-1.5 text-sm text-mute">
            <input type="checkbox" checked={remember} onChange={(e) => setRemember(e.target.checked)} />
            remember for this tab
          </label>
        </div>
        {secret && (
          <p className="text-xs text-accent-3">
            Prefer a viewing key: a spend key can move funds. This page derives the viewing key
            locally and keeps only that in memory.
          </p>
        )}
        {error && <p className="text-xs text-accent-3">{error}</p>}
      </form>
      {children}
    </Panel>
  );
}
