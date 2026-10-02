'use client';

import { useParams } from 'next/navigation';
import { Hash } from '@/components/Hash';
import { DetailSkeleton } from '@/components/Loading';
import { StatsCard, StatsRow } from '@/components/StatsCard';
import { TransactionsTable } from '@/components/Tables';
import { DetailRow, ErrorState, NotFoundState, PageHeader, Panel } from '@/components/States';
import { isNotFound, useProgram, useTokens } from '@/hooks/useApi';
import { L, useFmt, useT } from '@/i18n/client';

export default function ProgramDetailPage() {
  const params = useParams<{ id: string }>();
  const id = decodeURIComponent(params.id);
  const { data: program, error, isLoading, mutate } = useProgram(id);
  const { data: tokenList } = useTokens();
  const tokens = tokenList?.tokens ?? [];
  const { t, tp, rich } = useT();
  const fmt = useFmt();

  if (isLoading && !program) {
    return <DetailSkeleton />;
  }

  if (error && isNotFound(error)) {
    return (
      <NotFoundState
        title={t('program.notFound')}
        message={t('program.notFoundMessage', { id })}
        backHref="/programs"
        backLabel={t('program.back')}
      />
    );
  }

  if (error || !program) {
    return <ErrorState message={t('program.loadError')} onRetry={() => void mutate()} />;
  }

  return (
    <div className="space-y-6">
      <PageHeader title={t('program.title')} subtitle={<Hash value={program.id} full copyable />} />

      <StatsRow columns={4}>
        <StatsCard
          title={t('program.calls')}
          value={fmt.number(program.call_count)}
          subtitle={
            program.invoke_count ? tp('program.andInvokes', program.invoke_count) : undefined
          }
        />
        <StatsCard title={t('program.codeSize')} value={tp('program.words', program.words_len)} />
        <StatsCard title={t('program.deployedAt')} value={`#${fmt.number(program.deployed_at_height)}`} />
        <StatsCard
          title={t('program.lastCalled')}
          value={
            lastActivity(program.last_called_height, program.last_invoked_height ?? null) === null
              ? '—'
              : `#${fmt.number(lastActivity(program.last_called_height, program.last_invoked_height ?? null))}`
          }
          subtitle={program.invoke_count ? t('program.callOrInvoke') : undefined}
        />
      </StatsRow>

      <Panel title={t('program.details')}>
        <DetailRow label={t('program.programId')}>
          <Hash value={program.id} full />
        </DetailRow>
        <DetailRow label={t('program.deployTx')}>
          <Hash value={program.deploy_tx} href={`/transactions/${program.deploy_tx}`} full />
        </DetailRow>
        <DetailRow label={t('program.deployedAtHeight')}>
          <L href={`/blocks/${program.deployed_at_height}`} className="link font-mono">
            #{fmt.number(program.deployed_at_height)}
          </L>
        </DetailRow>
        <DetailRow label={t('program.basePc')}>
          <span className="font-mono">{fmt.number(program.base_pc)}</span>
        </DetailRow>
        <DetailRow label={t('program.wordsLength')}>
          <span className="font-mono">{tp('program.words', program.words_len)}</span>
        </DetailRow>
        <DetailRow label={t('program.codeHash')}>
          <Hash value={program.code_hash} full />
        </DetailRow>
        <DetailRow label={t('program.publicInput')}>
          {program.public_words_len > 0 ? (
            <span className="font-mono">{tp('program.words', program.public_words_len)}</span>
          ) : (
            <span className="text-mute">{t('program.noPublicInput')}</span>
          )}
        </DetailRow>
        {program.public_digest && (
          <DetailRow label={t('program.publicDigest')}>
            <Hash value={program.public_digest} full />
          </DetailRow>
        )}
        <DetailRow label={t('program.callCount')}>
          <span className="font-mono">{fmt.number(program.call_count)}</span>
        </DetailRow>
        <DetailRow label={t('program.lastCalled')}>
          {program.last_called_height === null ? (
            <span className="text-mute">{t('program.never')}</span>
          ) : (
            <L href={`/blocks/${program.last_called_height}`} className="link font-mono">
              #{fmt.number(program.last_called_height)}
            </L>
          )}
        </DetailRow>
      </Panel>

      {program.program_state && (
        <>
          <Panel title={t('program.vault')}>
            {program.program_state.vault.length === 0 ? (
              <p className="py-3 text-sm text-mute">
                {t('program.vaultEmpty')}
              </p>
            ) : (
              program.program_state.vault.map((row) => (
                <DetailRow key={row.asset} label={row.asset === 0 ? 'RAND' : t('program.asset', { index: row.asset })}>
                  <span className="font-mono text-strong">
                    {fmt.tokenAmount(row.amount, row.asset, tokens)}
                  </span>
                </DetailRow>
              ))
            )}
          </Panel>

          <section className="space-y-4">
            <h2 className="chip">
              {t('program.stateCells', {
                count: `${fmt.number(program.program_state.cells.length)}${program.program_state.cells_next ? '+' : ''}`,
              })}
            </h2>
            {program.program_state.cells.length === 0 ? (
              <p className="text-sm text-mute">{t('program.noCells')}</p>
            ) : (
              <ul className="card divide-y divide-border-soft">
                {program.program_state.cells.map((c) => (
                  <li key={c.key} className="flex flex-col gap-1 px-6 py-3 text-xs md:flex-row md:items-center md:gap-6">
                    <span className="inline-flex items-center gap-2">
                      <span className="w-10 text-mute">{t('program.key')}</span>
                      <Hash value={c.key} start={14} end={10} className="text-xs" />
                    </span>
                    <span className="inline-flex items-center gap-2">
                      <span className="w-10 text-mute">{t('program.value')}</span>
                      <Hash value={c.value} start={14} end={10} className="text-xs" />
                    </span>
                  </li>
                ))}
              </ul>
            )}
            <p className="text-xs text-mute">
              {t('program.statePublic')}
              {program.program_state.cells_next && (
                <> {rich('program.moreCells', { id: program.id })}</>
              )}
            </p>
          </section>
        </>
      )}

      <section className="space-y-4">
        <h2 className="chip">{program.invoke_count ? t('program.recentCallsInvokes') : t('program.recentCalls')}</h2>
        <TransactionsTable
          transactions={program.recent_calls}
          hideColumns={['action']}
          emptyMessage={t('program.empty')}
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
