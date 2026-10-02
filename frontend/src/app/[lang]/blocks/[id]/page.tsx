'use client';

import { useParams } from 'next/navigation';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useBlock } from '@/hooks/useApi';
import { L, useFmt, useT } from '@/i18n/client';

export default function BlockDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: block, error, isLoading, mutate } = useBlock(id);
  const { t } = useT();
  const fmt = useFmt();

  if (isLoading && !block) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title={t('block.notFound')}
        message={t('block.notFoundMessage', { id })}
        backHref="/blocks"
        backLabel={t('block.back')}
      />
    );
  }

  if (error || !block) {
    return <ErrorState message={t('block.loadError')} onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('block.title', { height: fmt.number(block.height) })}
        subtitle={
          <span className="inline-flex items-center gap-2">
            <Hash value={block.hash} full copyable />
          </span>
        }
      />

      <Panel title={t('block.overview')}>
        <DetailRow label={t('block.height')}>
          <span className="font-mono">{fmt.number(block.height)}</span>
        </DetailRow>
        <DetailRow label={t('block.hash')}>
          <Hash value={block.hash} full />
        </DetailRow>
        <DetailRow label={t('block.parent')}>
          {block.parent && !/^0+$/.test(block.parent) ? (
            <Hash value={block.parent} href={`/blocks/${block.parent}`} full />
          ) : (
            <span className="text-mute">{t('block.genesis')}</span>
          )}
        </DetailRow>
        <DetailRow label={t('block.proposer')}>
          <Hash value={block.proposer} href={`/validators/${block.proposer}`} full />
        </DetailRow>
        <DetailRow label={t('block.timestamp')}>
          <span>
            {fmt.dateTime(block.timestamp_ms)}{' '}
            <span className="text-mute">({fmt.ago(block.timestamp_ms)})</span>
          </span>
        </DetailRow>
        <DetailRow label={t('block.view')}>
          <span className="font-mono">{fmt.number(block.view)}</span>
        </DetailRow>
        <DetailRow label={t('block.justifyView')}>
          <span className="font-mono">{fmt.number(block.justify_view)}</span>
        </DetailRow>
        <DetailRow label={t('block.transactions')}>
          <span className="font-mono">{fmt.number(block.tx_count)}</span>
        </DetailRow>
        <DetailRow label={t('block.txRoot')}>
          <Hash value={block.tx_root} full />
        </DetailRow>
        <DetailRow label={t('block.stateRoot')}>
          <Hash value={block.state_root} full />
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="chip">
            {t('block.txHeading', { count: fmt.number(block.transactions.length) })}
          </h2>
          {block.height > 0 && (
            <L href={`/blocks/${block.height - 1}`} className="link text-sm">
              {t('block.previous')}
            </L>
          )}
        </div>
        <TransactionsTable
          transactions={block.transactions}
          hideColumns={['height']}
          emptyMessage={t('block.empty')}
        />
      </section>
    </div>
  );
}
