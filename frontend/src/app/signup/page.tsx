import type { Metadata } from 'next';
import { AuthForm } from '@/components/AuthForm';

export const metadata: Metadata = { title: 'Create account — RandScan' };

export default function SignupPage() {
  return <AuthForm mode="signup" />;
}
