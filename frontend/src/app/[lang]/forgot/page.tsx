import type { Metadata } from 'next';
import { ForgotPasswordForm } from '@/components/PasswordForms';
import { serverT } from '@/i18n/server';

type P = { params: Promise<{ lang: string }> };

export async function generateMetadata({ params }: P): Promise<Metadata> {
  const { t } = await serverT(params);
  return { title: t('auth.forgot.metaTitle') };
}

export default function ForgotPage() {
  return <ForgotPasswordForm />;
}
