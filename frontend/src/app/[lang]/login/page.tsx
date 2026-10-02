import type { Metadata } from 'next';
import { AuthForm } from '@/components/AuthForm';
import { serverT } from '@/i18n/server';

type P = { params: Promise<{ lang: string }> };

export async function generateMetadata({ params }: P): Promise<Metadata> {
  const { t } = await serverT(params);
  return { title: t('auth.login.metaTitle') };
}

export default function LoginPage() {
  return <AuthForm mode="login" />;
}
