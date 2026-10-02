'use client';

import { useParams } from 'next/navigation';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useNote } from '@/hooks/useApi';
import { L, useFmt, useT } from '@/i18n/client';

export default function NoteDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: note, error, isLoading, mutate } = useNote(id);
  const { t, rich } = useT();
  const fmt = useFmt();

  if (isLoading && !note) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title={t('note.notFound')}
        message={t('note.notFoundMessage', { id })}
        backHref="/notes"
        backLabel={t('note.back')}
      />
    );
  }

  if (error || !note) {
    return <ErrorState message={t('note.loadError')} onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('note.title', { index: fmt.number(note.leaf_index) })}
        subtitle={<Hash value={note.cm} full copyable />}
      />

      <Panel title={t('note.leaf')}>
        <DetailRow label={t('note.leafIndex')}>
          <span className="font-mono">{fmt.number(note.leaf_index)}</span>
        </DetailRow>
        <DetailRow label={t('note.commitment')}>
          <Hash value={note.cm} full />
        </DetailRow>
        <DetailRow label={t('note.createdAt')}>
          <L href={`/blocks/${note.height}`} className="link font-mono">
            #{fmt.number(note.height)}
          </L>
        </DetailRow>
        <DetailRow label={t('note.createdBy')}>
          {note.tx_hash ? (
            <Hash value={note.tx_hash} href={`/transactions/${note.tx_hash}`} full />
          ) : (
            <span className="text-mute">
              {note.height === 0 ? t('note.genesisDeposit') : t('note.depositOffWire')}
            </span>
          )}
        </DetailRow>
        <p className="px-4 py-3 text-xs text-mute">
          {note.tx_hash && (
            <>
              {rich('note.openWithKey', { href: `/transactions/${note.tx_hash}` })} <br />
            </>
          )}
          {t('note.sealed')}
        </p>
      </Panel>
    </div>
  );
}
