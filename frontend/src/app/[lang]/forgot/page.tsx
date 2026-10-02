import type { Metadata } from 'next';
import { ForgotPasswordForm } from '@/components/PasswordForms';

export const metadata: Metadata = { title: 'Forgot password — RandScan' };

export default function ForgotPage() {
  return <ForgotPasswordForm />;
}
