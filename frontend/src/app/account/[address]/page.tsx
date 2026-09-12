'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { PageHeader, Panel } from '@/components/States';
import { useValidator } from '@/hooks/useApi';

/**
 * The shielded chain has no accounts. Old links (and old habits) land here; explain, and point
 * at the validator page when the address is one.
 */
export default function NoAccountPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const { data: validator } = useValidator(address);

  return (
    <div className="space-y-6">
      <PageHeader title="No accounts on this chain" subtitle={<Hash value={address} full copyable />} />

      <Panel>
        <div className="space-y-3 py-3 text-sm text-soft">
          <p>
            SHRUGG is a fully shielded chain. A balance is a set of notes only its owner&apos;s
            viewing key can open, and a transaction carries no sender, recipient or amount: only
            commitments, nullifiers, sealed envelopes, the fee and a proof.
          </p>
          <p>
            There is therefore nothing the explorer can show for an address. What is public is
            the validator register (stake, rewards, unbonding queue), the commitment tree and the
            nullifier set, program deployments, call receipts and the bridge&apos;s own state.
          </p>
          {validator ? (
            <p>
              This address is a validator:{' '}
              <Link href={`/validators/${address}`} className="link">
                view its register entry →
              </Link>
            </p>
          ) : (
            <p className="flex flex-wrap gap-4">
              <Link href="/validators" className="link">
                Validators →
              </Link>
              <Link href="/notes" className="link">
                Notes (commitment tree) →
              </Link>
            </p>
          )}
        </div>
      </Panel>
    </div>
  );
}
