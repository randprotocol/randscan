'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useProgram } from '@/hooks/useApi';
import { formatNumber } from '@/lib/utils';

export default function ProgramDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: program, error, isLoading, mutate } = useProgram(id);

  if (isLoading && !program) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title="Program not found"
        message={`No program matches "${id}". Program ids are 64-character hashes.`}
        backHref="/programs"
        backLabel="Back to programs"
      />
    );
  }

  if (error || !program) {
    return <ErrorState message="Could not load this program." onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Program" subtitle={<Hash value={program.id} full copyable />} />

      <StatsRow columns={4}>
        <StatsCard title="Calls" value={formatNumber(program.call_count)} />
        <StatsCard title="Code size" value={`${formatNumber(program.words_len)} words`} />
        <StatsCard title="Deployed at" value={`#${formatNumber(program.deployed_at_height)}`} />
        <StatsCard
          title="Last called"
          value={
            program.last_called_height === null
              ? '—'
              : `#${formatNumber(program.last_called_height)}`
          }
        />
      </StatsRow>

      <Panel title="Details">
        <DetailRow label="Program id">
          <Hash value={program.id} full />
        </DetailRow>
        <DetailRow label="Deploy transaction">
          <Hash value={program.deploy_tx} href={`/transactions/${program.deploy_tx}`} full />
        </DetailRow>
        <DetailRow label="Deployed at height">
          <Link href={`/blocks/${program.deployed_at_height}`} className="link font-mono">
            #{formatNumber(program.deployed_at_height)}
          </Link>
        </DetailRow>
        <DetailRow label="Base PC">
          <span className="font-mono">{formatNumber(program.base_pc)}</span>
        </DetailRow>
        <DetailRow label="Words length">
          <span className="font-mono">{formatNumber(program.words_len)} words</span>
        </DetailRow>
        <DetailRow label="Code hash">
          <Hash value={program.code_hash} full />
        </DetailRow>
        <DetailRow label="Call count">
          <span className="font-mono">{formatNumber(program.call_count)}</span>
        </DetailRow>
        <DetailRow label="Last called">
          {program.last_called_height === null ? (
            <span className="text-mute">Never</span>
          ) : (
            <Link href={`/blocks/${program.last_called_height}`} className="link font-mono">
              #{formatNumber(program.last_called_height)}
            </Link>
          )}
        </DetailRow>
      </Panel>

      <section className="space-y-4">
        <h2 className="chip">Recent calls</h2>
        <TransactionsTable
          transactions={program.recent_calls}
          hideColumns={['action']}
          emptyMessage="This program has not been called yet"
        />
      </section>
    </div>
  );
}
