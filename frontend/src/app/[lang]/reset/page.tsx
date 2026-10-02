import type { Metadata } from 'next';
import { Suspense } from 'react';
import { PageLoading } from '@/components/Loading';
import { ResetPasswordForm } from '@/components/PasswordForms';

export const metadata: Metadata = { title: 'Reset password — RandScan' };

export default function ResetPage() {
  return (
    <Suspense fallback={<PageLoading />}>
      <ResetPasswordForm />
    </Suspense>
  );
}
