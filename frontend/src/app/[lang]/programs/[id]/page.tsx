'use client';

import { useParams } from 'next/navigation';
import Link from 'next/link';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useProgram, useTokens } from '@/hooks/useApi';
import { formatNumber, formatTokenAmount } from '@/lib/utils';

export default function ProgramDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: program, error, isLoading, mutate } = useProgram(id);
  const { data: tokenList } = useTokens();
  const tokens = tokenList?.tokens ?? [];

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
        <StatsCard
          title="Calls"
          value={formatNumber(program.call_count)}
          subtitle={
            program.invoke_count ? `and ${formatNumber(program.invoke_count)} invokes` : undefined
          }
        />
        <StatsCard title="Code size" value={`${formatNumber(program.words_len)} words`} />
        <StatsCard title="Deployed at" value={`#${formatNumber(program.deployed_at_height)}`} />
        <StatsCard
          title="Last called"
          value={
            lastActivity(program.last_called_height, program.last_invoked_height ?? null) === null
              ? '—'
              : `#${formatNumber(lastActivity(program.last_called_height, program.last_invoked_height ?? null))}`
          }
          subtitle={program.invoke_count ? 'call or invoke' : undefined}
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
        <DetailRow label="Public input">
          {program.public_words_len > 0 ? (
            <span className="font-mono">{formatNumber(program.public_words_len)} words</span>
          ) : (
            <span className="text-mute">None — calls are checked against the empty input</span>
          )}
        </DetailRow>
        {program.public_digest && (
          <DetailRow label="Public digest (H_PUB)">
            <Hash value={program.public_digest} full />
          </DetailRow>
        )}
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

      {program.program_state && (
        <>
          <Panel title="Vault">
            {program.program_state.vault.length === 0 ? (
              <p className="py-3 text-sm text-mute">
                The program holds nothing. Value enters its vault through an invoke&apos;s bundle
                and leaves it as the notes the invoke pays out.
              </p>
            ) : (
              program.program_state.vault.map((row) => (
                <DetailRow key={row.asset} label={row.asset === 0 ? 'RAND' : `Asset #${row.asset}`}>
                  <span className="font-mono text-strong">
                    {formatTokenAmount(row.amount, row.asset, tokens)}
                  </span>
                </DetailRow>
              ))
            )}
          </Panel>

          <section className="space-y-4">
            <h2 className="chip">State cells ({formatNumber(program.program_state.cells.length)}{program.program_state.cells_next ? '+' : ''})</h2>
            {program.program_state.cells.length === 0 ? (
              <p className="text-sm text-mute">No cell: nothing has been written, or every write was zeros.</p>
            ) : (
              <ul className="card divide-y divide-border-soft">
                {program.program_state.cells.map((c) => (
                  <li key={c.key} className="flex flex-col gap-1 px-6 py-3 text-xs md:flex-row md:items-center md:gap-6">
                    <span className="inline-flex items-center gap-2">
                      <span className="w-10 text-mute">key</span>
                      <Hash value={c.key} start={14} end={10} className="text-xs" />
                    </span>
                    <span className="inline-flex items-center gap-2">
                      <span className="w-10 text-mute">value</span>
                      <Hash value={c.value} start={14} end={10} className="text-xs" />
                    </span>
                  </li>
                ))}
              </ul>
            )}
            <p className="text-xs text-mute">
              Public by design: a program&apos;s state is a map of 64-hex keys to 64-hex values
              (eight words each), read live from the node in key order.
              {program.program_state.cells_next && (
                <> More cells follow; the API pages them at <code>/api/v1/programs/{program.id}/cells?after=…</code>.</>
              )}
            </p>
          </section>
        </>
      )}

      <section className="space-y-4">
        <h2 className="chip">{program.invoke_count ? 'Recent calls and invokes' : 'Recent calls'}</h2>
        <TransactionsTable
          transactions={program.recent_calls}
          hideColumns={['action']}
          emptyMessage="This program has not been called yet"
        />
      </section>
    </div>
  );
}

/** The later of the last call and the last invoke, or null when neither happened. */
function lastActivity(call: number | null, invoke: number | null): number | null {
  if (call === null) return invoke;
  if (invoke === null) return call;
  return Math.max(call, invoke);
}
