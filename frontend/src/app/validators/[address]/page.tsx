'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { BlocksTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useValidator } from '@/hooks/useApi';
import {
  formatDateTime,
  formatNumber,
  formatPercentage,
  formatStake,
  formatTimestamp,
} from '@/lib/utils';

export default function ValidatorDetailPage() {
  const params = useParams<{ address: string }>();
  const address = decodeURIComponent(params.address);
  const { data: validator, error, isLoading, mutate } = useValidator(address);

  if (isLoading && !validator) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Validator not found"
        message={`"${address}" is not part of the current validator set.`}
        backHref="/validators"
        backLabel="Back to validators"
      />
    );
  }

  if (error || !validator) {
    return <ErrorState message="Could not load this validator." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Validator"
        subtitle={<Hash value={validator.address} full copyable />}
        actions={
          <Link href={`/account/${validator.address}`} className="link text-sm">
            View account →
          </Link>
        }
      />

      <StatsRow columns={4}>
        <StatsCard title="Stake" value={formatStake(validator.stake)} />
        <StatsCard title="Share of stake" value={formatPercentage(validator.share_percent)} />
        <StatsCard title="Blocks proposed" value={formatNumber(validator.blocks_proposed)} />
        <StatsCard
          title="Last proposed"
          value={
            validator.last_proposed_height === null
              ? '—'
              : `#${formatNumber(validator.last_proposed_height)}`
          }
          subtitle={
            validator.last_proposed_timestamp_ms !== null
              ? formatTimestamp(validator.last_proposed_timestamp_ms)
              : undefined
          }
        />
      </StatsRow>

      <Panel title="Details">
        <DetailRow label="Address">
          <Hash value={validator.address} full />
        </DetailRow>
        <DetailRow label="Stake">{formatStake(validator.stake)}</DetailRow>
        <DetailRow label="Share of total stake">
          {formatPercentage(validator.share_percent)}
        </DetailRow>
        <DetailRow label="Sort index">
          <span className="font-mono">{formatNumber(validator.sort_index)}</span>
        </DetailRow>
        <DetailRow label="Blocks proposed">
          <span className="font-mono">{formatNumber(validator.blocks_proposed)}</span>
        </DetailRow>
        <DetailRow label="Last proposed block">
          {validator.last_proposed_height === null ? (
            <span className="text-mute">Never</span>
          ) : (
            <span className="inline-flex flex-wrap items-center gap-2">
              <Link href={`/blocks/${validator.last_proposed_height}`} className="link font-mono">
                #{formatNumber(validator.last_proposed_height)}
              </Link>
              {validator.last_proposed_timestamp_ms !== null && (
                <span className="text-mute">
                  {formatDateTime(validator.last_proposed_timestamp_ms)}
                </span>
              )}
            </span>
          )}
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">Recent blocks</h2>
        <BlocksTable
          blocks={validator.recent_blocks}
          emptyMessage="This validator has not proposed any blocks yet"
        />
      </section>
    </div>
  );
}
