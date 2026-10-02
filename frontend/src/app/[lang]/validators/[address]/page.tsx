'use client';

import { useParams } from 'next/navigation';
import { useState } from 'react';
import { Column, DataTable } from '@/components/DataTable';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { Pagination } from '@/components/Pagination';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { BlocksTable, TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useTransactions, useValidator } from '@/hooks/useApi';
import { L, useFmt, useT, type Fmt } from '@/i18n/client';
import type { PendingStake } from '@/types';

const PAGE_SIZE = 10;

const pendingColumns = (t: (key: string) => string, fmt: Fmt): Column<PendingStake>[] => [
  {
    key: 'release_epoch',
    header: t('validator.releaseEpoch'),
    render: (p) => <span className="font-mono">{fmt.number(p.release_epoch)}</span>,
  },
  {
    key: 'amount',
    header: t('validator.amount'),
    render: (p) => <span className="text-text">{fmt.stake(p.amount)}</span>,
  },
];

export default function ValidatorDetailPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const [page, setPage] = useState(1);
  const { data: validator, error, isLoading, mutate } = useValidator(address);
  const { data: txs, isLoading: txsLoading } = useTransactions(page, PAGE_SIZE, null, {
    validator: error ? null : address,
  });
  const { t } = useT();
  const fmt = useFmt();

  if (isLoading && !validator) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title={t('validator.notFound')}
        message={t('validator.notFoundMessage', { address })}
        backHref="/validators"
        backLabel={t('validator.back')}
      />
    );
  }

  if (error || !validator) {
    return <ErrorState message={t('validator.loadError')} onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('validator.title')}
        subtitle={<Hash value={validator.address} full copyable />}
        actions={
          validator.active ? (
            <span className="badge badge-accent">{t('validator.active')}</span>
          ) : (
            <span className="badge badge-neutral">{t('validator.inactive')}</span>
          )
        }
      />

      <StatsRow columns={4}>
        <StatsCard title={t('validator.stake')} value={fmt.stake(validator.stake)} />
        <StatsCard
          title={t('validator.rewards')}
          value={fmt.stake(validator.rewards)}
          subtitle={t('validator.unwithdrawnFees')}
        />
        <StatsCard title={t('validator.share')} value={fmt.percentage(validator.share_percent)} />
        <StatsCard
          title={t('validator.blocksProposed')}
          value={fmt.number(validator.blocks_proposed)}
          subtitle={
            validator.last_proposed_timestamp_ms !== null
              ? t('validator.lastAgo', { ago: fmt.ago(validator.last_proposed_timestamp_ms) })
              : undefined
          }
        />
      </StatsRow>

      <Panel title={t('validator.registerEntry')}>
        <DetailRow label={t('validator.address')}>
          <Hash value={validator.address} full />
        </DetailRow>
        <DetailRow label={t('validator.stake')}>{fmt.stake(validator.stake)}</DetailRow>
        <DetailRow label={t('validator.rewards')}>{fmt.stake(validator.rewards)}</DetailRow>
        <DetailRow label={t('validator.payout')}>
          {validator.payout ? (
            <Hash value={validator.payout} full />
          ) : (
            <span className="text-mute">{t('validator.payoutUnknown')}</span>
          )}
        </DetailRow>
        <DetailRow label={t('validator.nonce')}>
          <span className="font-mono">{fmt.number(validator.nonce)}</span>
        </DetailRow>
        <DetailRow label={t('validator.share')}>
          {fmt.percentage(validator.share_percent)}
        </DetailRow>
        <DetailRow label={t('validator.sortIndex')}>
          <span className="font-mono">{fmt.number(validator.sort_index)}</span>
        </DetailRow>
        <DetailRow label={t('validator.lastProposed')}>
          {validator.last_proposed_height === null ? (
            <span className="text-mute">{t('validator.never')}</span>
          ) : (
            <span className="inline-flex flex-wrap items-center gap-2">
              <L href={`/blocks/${validator.last_proposed_height}`} className="link font-mono">
                #{fmt.number(validator.last_proposed_height)}
              </L>
              {validator.last_proposed_timestamp_ms !== null && (
                <span className="text-mute">
                  {fmt.dateTime(validator.last_proposed_timestamp_ms)}
                </span>
              )}
            </span>
          )}
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">{t('validator.unbonding')}</h2>
        <DataTable
          columns={pendingColumns(t, fmt)}
          data={validator.pending}
          keyExtractor={(p, ) => `${p.release_epoch}-${p.amount}`}
          emptyMessage={t('validator.unbondingEmpty')}
        />
      </section>

      <section className="space-y-4">
        <h2 className="chip">{t('validator.stakingTxs')}</h2>
        <TransactionsTable
          transactions={txs?.data ?? []}
          isLoading={txsLoading && !txs}
          emptyMessage={t('validator.stakingEmpty')}
        />
        {txs && txs.pagination.total_pages > 1 && (
          <Pagination
            currentPage={txs.pagination.page}
            totalPages={txs.pagination.total_pages}
            onPageChange={setPage}
          />
        )}
      </section>

      <section className="space-y-4">
        <h2 className="chip">{t('validator.recentBlocks')}</h2>
        <BlocksTable
          blocks={validator.recent_blocks}
          emptyMessage={t('validator.blocksEmpty')}
        />
      </section>
    </div>
  );
}
