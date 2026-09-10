'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useRef, useState } from 'react';
import { useSWRConfig } from 'swr';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { useApiKeys, useMe } from '@/hooks/useApi';
import * as api from '@/lib/api';
import { copyToClipboard, formatDateTime, formatNumber } from '@/lib/utils';
import type { ApiKey, CreatedApiKey } from '@/types';

function when(iso: string | null): string {
  return iso ? formatDateTime(Date.parse(iso)) : '—';
}

export default function DashboardPage() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { data: me, isLoading: meLoading, error: meError } = useMe();
  const { data: keys, error: keysError, mutate: refreshKeys } = useApiKeys(!!me);

  const [name, setName] = useState('');
  const [created, setCreated] = useState<CreatedApiKey | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const signingOut = useRef(false);

  useEffect(() => {
    if (!meLoading && me === null && !signingOut.current) router.replace('/login');
  }, [me, meLoading, router]);

  if (meError) return <ErrorState onRetry={() => void mutate('me')} />;
  if (!me) return <DetailSkeleton />;

  const onCreate = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const key = await api.createKey(name);
      setCreated(key);
      setCopied(false);
      setName('');
      await refreshKeys();
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
    } finally {
      setBusy(false);
    }
  };

  const onRevoke = async (key: ApiKey) => {
    setError(null);
    try {
      await api.revokeKey(key.id);
      if (created?.id === key.id) setCreated(null);
      await refreshKeys();
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
    }
  };

  const onLogout = async () => {
    signingOut.current = true;
    try {
      await api.logout();
      await mutate('me', null, { revalidate: false });
      router.push('/');
    } catch (err) {
      signingOut.current = false;
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
    }
  };

  const active = (keys ?? []).filter((k) => !k.revoked_at);
  const revoked = (keys ?? []).filter((k) => k.revoked_at);

  return (
    <div className="space-y-8">
      <PageHeader
        label="Account"
        title="API keys"
        subtitle={me.email}
        actions={
          <button type="button" onClick={onLogout} className="btn-secondary">
            Sign out
          </button>
        }
      />

      {created && (
        <Panel title="New key">
          <div className="py-3">
            <p className="text-sm text-soft">
              Copy your key now. For security it is shown only once; if you lose it, revoke it and
              create another.
            </p>
            <div className="mt-3 flex flex-wrap items-center gap-3">
              <code className="break-all rounded border border-border bg-surface-2 px-3 py-2 font-mono text-sm text-strong">
                {created.key}
              </code>
              <button
                type="button"
                className="btn-secondary"
                onClick={async () => setCopied(await copyToClipboard(created.key))}
              >
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
            <pre className="mt-4 overflow-x-auto rounded border border-border bg-surface-2 p-3 font-mono text-xs text-soft">
{`curl -H "Authorization: Bearer ${created.key}" https://randscan.org/api/v1/stats`}
            </pre>
          </div>
        </Panel>
      )}

      <Panel title="Create a key">
        <form onSubmit={onCreate} className="flex flex-wrap items-end gap-3 py-3">
          <div className="min-w-[16rem] flex-1">
            <label htmlFor="key-name" className="field-label">
              Name
            </label>
            <input
              id="key-name"
              className="input"
              placeholder="e.g. exchange deposit watcher"
              maxLength={64}
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <button type="submit" disabled={busy || active.length >= 10} className="btn-primary disabled:opacity-60">
            New key
          </button>
        </form>
        {error && (
          <div className="form-error mb-3" role="alert">
            {error}
          </div>
        )}
        <p className="pb-3 text-xs text-mute">
          Up to 10 active keys. Keyed requests get 600 requests per minute; anonymous traffic gets
          60 per IP. See the{' '}
          <Link href="https://github.com/randprotocol/randscan/blob/main/docs/api.md" className="link">
            API guide
          </Link>
          .
        </p>
      </Panel>

      <Panel title={`Active keys (${active.length})`}>
        {keysError && <ErrorState message="Could not load your keys." onRetry={() => void refreshKeys()} />}
        {!keysError && active.length === 0 && (
          <p className="py-4 text-sm text-mute">No active keys yet.</p>
        )}
        {active.map((k) => (
          <DetailRow key={k.id} label={k.name}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-sm">
                <span className="font-mono text-strong">rsk_{k.prefix}…</span>
                <span className="ml-3 text-mute">
                  created {when(k.created_at)} · last used {when(k.last_used_at)} ·{' '}
                  {formatNumber(k.request_count)} requests
                </span>
              </div>
              <button type="button" onClick={() => onRevoke(k)} className="btn-secondary text-accent-3">
                Revoke
              </button>
            </div>
          </DetailRow>
        ))}
      </Panel>

      {revoked.length > 0 && (
        <Panel title={`Revoked keys (${revoked.length})`}>
          {revoked.map((k) => (
            <DetailRow key={k.id} label={k.name}>
              <span className="font-mono text-mute">rsk_{k.prefix}…</span>
              <span className="ml-3 text-sm text-mute">
                revoked {when(k.revoked_at)} · {formatNumber(k.request_count)} requests
              </span>
            </DetailRow>
          ))}
        </Panel>
      )}

      <Panel title="Account">
        <DetailRow label="Email">{me.email}</DetailRow>
        <DetailRow label="Member since">{when(me.created_at)}</DetailRow>
        <DetailRow label="Last sign-in">{when(me.last_login_at)}</DetailRow>
      </Panel>
    </div>
  );
}
