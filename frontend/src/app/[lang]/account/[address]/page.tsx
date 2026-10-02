'use client';

import { useParams } from 'next/navigation';
import { Hash } from '@/components/Hash';
import { PageHeader, Panel } from '@/components/States';
import { useValidator } from '@/hooks/useApi';
import { L, useT } from '@/i18n/client';

/**
 * The shielded chain has no accounts. Old links (and old habits) land here; explain, and point
 * at the validator page when the address is one.
 */
export default function NoAccountPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const { data: validator } = useValidator(address);
  const { t, rich } = useT();

  return (
    <div className="space-y-6">
      <PageHeader title={t('account.title')} subtitle={<Hash value={address} full copyable />} />

      <Panel>
        <div className="space-y-3 py-3 text-sm text-soft">
          <p>{t('account.shielded')}</p>
          <p>{t('account.nothingToShow')}</p>
          {validator ? (
            <p>{rich('account.isValidator', { href: `/validators/${address}` })}</p>
          ) : (
            <p className="flex flex-wrap gap-4">
              <L href="/validators" className="link">
                {t('account.validators')}
              </L>
              <L href="/notes" className="link">
                {t('account.notes')}
              </L>
            </p>
          )}
        </div>
      </Panel>
    </div>
  );
}
