'use client';

import { useEffect, useRef, useState } from 'react';
import { getWebSocketClient } from '@/lib/websocket';
import type { BlockSummary, NetworkStats, TransactionSummary } from '@/types';

/** Tracks the shared connection status of the `/ws` client. */
export function useWebSocket(): { isConnected: boolean } {
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const client = getWebSocketClient();
    setIsConnected(client.isConnected);
    const unsubscribe = client.onStatusChange(setIsConnected);
    client.connect();

    return unsubscribe;
  }, []);

  return { isConnected };
}

function useStableHandler<T>(handler: (value: T) => void) {
  const ref = useRef(handler);
  useEffect(() => {
    ref.current = handler;
  }, [handler]);
  return ref;
}

export function useNewBlocks(
  handler: (block: BlockSummary) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected } = useWebSocket();
  const ref = useStableHandler(handler);

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;
    return getWebSocketClient().onBlock((block) => ref.current(block));
  }, [enabled, ref]);

  return { isConnected };
}

export function useNewTransactions(
  handler: (tx: TransactionSummary) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected } = useWebSocket();
  const ref = useStableHandler(handler);

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;
    return getWebSocketClient().onTransaction((tx) => ref.current(tx));
  }, [enabled, ref]);

  return { isConnected };
}

export function useStatsUpdates(
  handler: (stats: NetworkStats) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected } = useWebSocket();
  const ref = useStableHandler(handler);

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;
    return getWebSocketClient().onStats((stats) => ref.current(stats));
  }, [enabled, ref]);

  return { isConnected };
}
