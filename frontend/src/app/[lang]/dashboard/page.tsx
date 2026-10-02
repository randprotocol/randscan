'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useRef, useState } from 'react';
import { useSWRConfig } from 'swr';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { ChangePasswordForm } from '@/components/PasswordForms';
import { useApiKeys, useMe } from '@/hooks/useApi';
import * as api from '@/lib/api';
import { copyToClipboard } from '@/lib/utils';
import { useFmt, useLocalizePath, useT } from '@/i18n/client';
import type { ApiKey, CreatedApiKey } from '@/types';

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
  const { t, tp, rich } = useT();
  const fmt = useFmt();
  const localize = useLocalizePath();
  const when = (iso: string | null): string => (iso ? fmt.dateTime(Date.parse(iso)) : '—');

  useEffect(() => {
    if (!meLoading && me === null && !signingOut.current) router.replace(localize('/login'));
  }, [me, meLoading, router, localize]);

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
      setError(err instanceof api.ApiError ? err.message : t('dashboard.requestFailed'));
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
      setError(err instanceof api.ApiError ? err.message : t('dashboard.requestFailed'));
    }
  };

  const onLogout = async () => {
    signingOut.current = true;
    try {
      await api.logout();
      await mutate('me', null, { revalidate: false });
      router.push(localize('/'));
    } catch (err) {
      signingOut.current = false;
      setError(err instanceof api.ApiError ? err.message : t('dashboard.requestFailed'));
    }
  };

  const active = (keys ?? []).filter((k) => !k.revoked_at);
  const revoked = (keys ?? []).filter((k) => k.revoked_at);

  return (
    <div className="space-y-8">
      <PageHeader
        label={t('dashboard.account')}
        title={t('dashboard.title')}
        subtitle={me.email}
        actions={
          <button type="button" onClick={onLogout} className="btn-secondary">
            {t('dashboard.signOut')}
          </button>
        }
      />

      {created && (
        <Panel title={t('dashboard.newKey.title')}>
          <div className="py-3">
            <p className="text-sm text-soft">
              {t('dashboard.newKey.copyNow')}
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
                {copied ? t('common.copied') : t('common.copy')}
              </button>
            </div>
            <pre className="mt-4 overflow-x-auto rounded border border-border bg-surface-2 p-3 font-mono text-xs text-soft">
{`curl -H "Authorization: Bearer ${created.key}" https://randscan.org/api/v1/stats`}
            </pre>
          </div>
        </Panel>
      )}

      <Panel title={t('dashboard.create.title')}>
        <form onSubmit={onCreate} className="flex flex-wrap items-end gap-3 py-3">
          <div className="min-w-[16rem] flex-1">
            <label htmlFor="key-name" className="field-label">
              {t('dashboard.create.name')}
            </label>
            <input
              id="key-name"
              className="input"
              placeholder={t('dashboard.create.placeholder')}
              maxLength={64}
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <button type="submit" disabled={busy || active.length >= 10} className="btn-primary disabled:opacity-60">
            {t('dashboard.create.submit')}
          </button>
        </form>
        {error && (
          <div className="form-error mb-3" role="alert">
            {error}
          </div>
        )}
        <p className="pb-3 text-xs text-mute">
          {rich('dashboard.create.limits')}
        </p>
      </Panel>

      <Panel title={t('dashboard.activeKeys', { count: fmt.number(active.length) })}>
        {keysError && <ErrorState message={t('dashboard.keysError')} onRetry={() => void refreshKeys()} />}
        {!keysError && active.length === 0 && (
          <p className="py-4 text-sm text-mute">{t('dashboard.noActiveKeys')}</p>
        )}
        {active.map((k) => (
          <DetailRow key={k.id} label={k.name}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-sm">
                <span className="font-mono text-strong">rsk_{k.prefix}…</span>
                <span className="ms-3 text-mute">
                  {tp('dashboard.activeKeyMeta', k.request_count, {
                    created: when(k.created_at),
                    lastUsed: when(k.last_used_at),
                  })}
                </span>
              </div>
              <button type="button" onClick={() => onRevoke(k)} className="btn-secondary text-negative">
                {t('dashboard.revoke')}
              </button>
            </div>
          </DetailRow>
        ))}
      </Panel>

      {revoked.length > 0 && (
        <Panel title={t('dashboard.revokedKeys', { count: fmt.number(revoked.length) })}>
          {revoked.map((k) => (
            <DetailRow key={k.id} label={k.name}>
              <span className="font-mono text-mute">rsk_{k.prefix}…</span>
              <span className="ms-3 text-sm text-mute">
                {tp('dashboard.revokedKeyMeta', k.request_count, { revoked: when(k.revoked_at) })}
              </span>
            </DetailRow>
          ))}
        </Panel>
      )}

      <Panel title={t('dashboard.changePassword')}>
        <ChangePasswordForm />
      </Panel>

      <Panel title={t('dashboard.account')}>
        <DetailRow label={t('dashboard.email')}>{me.email}</DetailRow>
        <DetailRow label={t('dashboard.memberSince')}>{when(me.created_at)}</DetailRow>
        <DetailRow label={t('dashboard.lastSignIn')}>{when(me.last_login_at)}</DetailRow>
      </Panel>
    </div>
  );
}
