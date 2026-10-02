import type { Metadata } from 'next';
import { Suspense } from 'react';
import { PageLoading } from '@/components/Loading';
import { ResetPasswordForm } from '@/components/PasswordForms';
import { serverT } from '@/i18n/server';

type P = { params: Promise<{ lang: string }> };

export async function generateMetadata({ params }: P): Promise<Metadata> {
  const { t } = await serverT(params);
  return { title: t('auth.reset.metaTitle') };
}

export default function ResetPage() {
  return (
    <Suspense fallback={<PageLoading />}>
      <ResetPasswordForm />
    </Suspense>
  );
}
