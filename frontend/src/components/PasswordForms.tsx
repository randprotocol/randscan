'use client';

import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { useState } from 'react';
import { useSWRConfig } from 'swr';
import * as api from '@/lib/api';

function errorMessage(err: unknown): string {
  return err instanceof api.ApiError ? err.message : 'Request failed';
}

/** /forgot: asks for an email and always reports success (no account enumeration). */
export function ForgotPasswordForm() {
  const [email, setEmail] = useState('');
  const [sent, setSent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await api.forgotPassword(email);
      setSent(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-md">
      <h1 className="font-serif text-3xl font-medium tracking-tight text-strong">Forgot your password?</h1>
      <p className="mt-2 text-sm text-soft">
        Enter your account email and we will send a link to choose a new password. The link works
        for one hour.
      </p>
      {sent ? (
        <div className="card-padded mt-6 space-y-3">
          <p className="text-sm text-text">
            If an account exists for <span className="font-mono">{email.trim()}</span>, a reset link is
            on its way. Check your spam folder if it does not arrive within a few minutes.
          </p>
          <Link href="/login" className="link text-sm">
            Back to sign in
          </Link>
        </div>
      ) : (
        <form onSubmit={onSubmit} className="card-padded mt-6 space-y-4">
          <div>
            <label htmlFor="email" className="field-label">
              Email
            </label>
            <input
              id="email"
              type="email"
              autoComplete="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="input"
            />
          </div>
          {error && (
            <div className="form-error" role="alert">
              {error}
            </div>
          )}
          <button type="submit" disabled={busy} className="btn-primary w-full justify-center disabled:opacity-60">
            {busy ? 'Please wait…' : 'Send reset link'}
          </button>
          <p className="text-center text-sm text-soft">
            Remembered it?{' '}
            <Link href="/login" className="link">
              Sign in
            </Link>
          </p>
        </form>
      )}
    </div>
  );
}

/** /reset?token=…: sets a new password with the emailed token, then signs the user in. */
export function ResetPasswordForm() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const token = (useSearchParams().get('token') ?? '').trim();
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    if (password !== confirm) {
      setError('The two passwords do not match.');
      return;
    }
    setBusy(true);
    try {
      const user = await api.resetPassword(token, password);
      await mutate('me', user, { revalidate: false });
      router.push('/dashboard');
    } catch (err) {
      setError(errorMessage(err));
      setBusy(false);
    }
  };

  if (!token) {
    return (
      <div className="mx-auto max-w-md">
        <h1 className="font-serif text-3xl font-medium tracking-tight text-strong">Reset password</h1>
        <div className="card-padded mt-6 space-y-3">
          <p className="text-sm text-text">This link is missing its token. Open the link from the email, or request a new one.</p>
          <Link href="/forgot" className="link text-sm">
            Request a new link
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-md">
      <h1 className="font-serif text-3xl font-medium tracking-tight text-strong">Choose a new password</h1>
      <p className="mt-2 text-sm text-soft">You will be signed out of every other device.</p>
      <form onSubmit={onSubmit} className="card-padded mt-6 space-y-4">
        <div>
          <label htmlFor="password" className="field-label">
            New password
          </label>
          <input
            id="password"
            type="password"
            autoComplete="new-password"
            required
            minLength={10}
            maxLength={128}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="input"
          />
          <p className="mt-1.5 text-xs text-mute">At least 10 characters.</p>
        </div>
        <div>
          <label htmlFor="confirm" className="field-label">
            Confirm new password
          </label>
          <input
            id="confirm"
            type="password"
            autoComplete="new-password"
            required
            minLength={10}
            maxLength={128}
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            className="input"
          />
        </div>
        {error && (
          <div className="form-error" role="alert">
            {error}
          </div>
        )}
        <button type="submit" disabled={busy} className="btn-primary w-full justify-center disabled:opacity-60">
          {busy ? 'Please wait…' : 'Set new password'}
        </button>
        <p className="text-center text-sm text-soft">
          Link expired?{' '}
          <Link href="/forgot" className="link">
            Request a new one
          </Link>
        </p>
      </form>
    </div>
  );
}

/** Dashboard panel body: change the password of the signed-in user. */
export function ChangePasswordForm() {
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setDone(false);
    setBusy(true);
    try {
      await api.changePassword(current, next);
      setCurrent('');
      setNext('');
      setDone(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4 py-3">
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label htmlFor="current-password" className="field-label">
            Current password
          </label>
          <input
            id="current-password"
            type="password"
            autoComplete="current-password"
            required
            value={current}
            onChange={(e) => setCurrent(e.target.value)}
            className="input"
          />
        </div>
        <div>
          <label htmlFor="new-password" className="field-label">
            New password
          </label>
          <input
            id="new-password"
            type="password"
            autoComplete="new-password"
            required
            minLength={10}
            maxLength={128}
            value={next}
            onChange={(e) => setNext(e.target.value)}
            className="input"
          />
        </div>
      </div>
      {error && (
        <div className="form-error" role="alert">
          {error}
        </div>
      )}
      {done && <p className="text-sm text-live">Password changed. Other devices have been signed out.</p>}
      <button type="submit" disabled={busy} className="btn-secondary disabled:opacity-60">
        {busy ? 'Please wait…' : 'Change password'}
      </button>
    </form>
  );
}
