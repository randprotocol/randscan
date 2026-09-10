'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useSWRConfig } from 'swr';
import * as api from '@/lib/api';

interface AuthFormProps {
  mode: 'login' | 'signup';
}

const copy = {
  login: {
    title: 'Sign in',
    button: 'Sign in',
    alt: 'No account yet?',
    altLink: '/signup',
    altLabel: 'Create one',
  },
  signup: {
    title: 'Create an account',
    button: 'Create account',
    alt: 'Already registered?',
    altLink: '/login',
    altLabel: 'Sign in',
  },
} as const;

/** Email + password form used by /login and /signup. Redirects to /dashboard on success. */
export function AuthForm({ mode }: AuthFormProps) {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const text = copy[mode];

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const user = mode === 'login' ? await api.login(email, password) : await api.signup(email, password);
      await mutate('me', user, { revalidate: false });
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-md">
      <h1 className="font-serif text-3xl font-medium tracking-tight text-strong">{text.title}</h1>
      <p className="mt-2 text-sm text-soft">
        An account lets you create API keys with a higher request quota.
      </p>
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
        <div>
          <label htmlFor="password" className="field-label">
            Password
          </label>
          <input
            id="password"
            type="password"
            autoComplete={mode === 'login' ? 'current-password' : 'new-password'}
            required
            minLength={10}
            maxLength={128}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="input"
          />
          {mode === 'signup' && <p className="mt-1.5 text-xs text-mute">At least 10 characters.</p>}
        </div>
        {error && <div className="form-error">{error}</div>}
        <button type="submit" disabled={busy} className="btn-primary w-full justify-center disabled:opacity-60">
          {busy ? 'Please wait…' : text.button}
        </button>
        <p className="text-center text-sm text-soft">
          {text.alt}{' '}
          <Link href={text.altLink} className="link">
            {text.altLabel}
          </Link>
        </p>
      </form>
    </div>
  );
}
