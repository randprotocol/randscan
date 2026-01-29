'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import { getWebSocketClient, WebSocketClient } from '@/lib/websocket';
import type {
  WebSocketChannel,
  WebSocketBlockUpdate,
  WebSocketTransactionUpdate,
  WebSocketStatsUpdate,
} from '@/types';

interface UseWebSocketOptions {
  autoConnect?: boolean;
}

interface UseWebSocketReturn {
  isConnected: boolean;
  subscribe: (
    channel: WebSocketChannel,
    handler: (data: unknown) => void,
    params?: Record<string, string>
  ) => () => void;
  subscribeToBlocks: (handler: (block: WebSocketBlockUpdate) => void) => () => void;
  subscribeToTransactions: (handler: (tx: WebSocketTransactionUpdate) => void) => () => void;
  subscribeToStats: (handler: (stats: WebSocketStatsUpdate) => void) => () => void;
  subscribeToAccount: (address: string, handler: (data: unknown) => void) => () => void;
  subscribeToValidator: (identity: string, handler: (data: unknown) => void) => () => void;
  connect: () => Promise<void>;
  disconnect: () => void;
}

export function useWebSocket(options: UseWebSocketOptions = {}): UseWebSocketReturn {
  const { autoConnect = true } = options;
  const [isConnected, setIsConnected] = useState(false);
  const clientRef = useRef<WebSocketClient | null>(null);

  useEffect(() => {
    // Only run on client
    if (typeof window === 'undefined') return;

    clientRef.current = getWebSocketClient();

    const removeConnectHandler = clientRef.current.onConnect(() => {
      setIsConnected(true);
    });

    const removeDisconnectHandler = clientRef.current.onDisconnect(() => {
      setIsConnected(false);
    });

    if (autoConnect) {
      clientRef.current.connect().catch(console.error);
    }

    return () => {
      removeConnectHandler();
      removeDisconnectHandler();
    };
  }, [autoConnect]);

  const subscribe = useCallback(
    (
      channel: WebSocketChannel,
      handler: (data: unknown) => void,
      params?: Record<string, string>
    ) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribe(channel, handler, params);
    },
    []
  );

  const subscribeToBlocks = useCallback(
    (handler: (block: WebSocketBlockUpdate) => void) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribeToBlocks(handler);
    },
    []
  );

  const subscribeToTransactions = useCallback(
    (handler: (tx: WebSocketTransactionUpdate) => void) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribeToTransactions(handler);
    },
    []
  );

  const subscribeToStats = useCallback(
    (handler: (stats: WebSocketStatsUpdate) => void) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribeToStats(handler);
    },
    []
  );

  const subscribeToAccount = useCallback(
    (address: string, handler: (data: unknown) => void) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribeToAccount(address, handler);
    },
    []
  );

  const subscribeToValidator = useCallback(
    (identity: string, handler: (data: unknown) => void) => {
      if (!clientRef.current) return () => {};
      return clientRef.current.subscribeToValidator(identity, handler);
    },
    []
  );

  const connect = useCallback(async () => {
    if (!clientRef.current) {
      clientRef.current = getWebSocketClient();
    }
    await clientRef.current.connect();
  }, []);

  const disconnect = useCallback(() => {
    if (clientRef.current) {
      clientRef.current.disconnect();
    }
  }, []);

  return {
    isConnected,
    subscribe,
    subscribeToBlocks,
    subscribeToTransactions,
    subscribeToStats,
    subscribeToAccount,
    subscribeToValidator,
    connect,
    disconnect,
  };
}

// Hook for subscribing to specific channels with automatic cleanup
export function useBlocksSubscription(
  handler: (block: WebSocketBlockUpdate) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected, subscribeToBlocks } = useWebSocket();
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return;

    const unsubscribe = subscribeToBlocks((block) => {
      handlerRef.current(block);
    });

    return unsubscribe;
  }, [enabled, subscribeToBlocks]);

  return { isConnected };
}

export function useTransactionsSubscription(
  handler: (tx: WebSocketTransactionUpdate) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected, subscribeToTransactions } = useWebSocket();
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return;

    const unsubscribe = subscribeToTransactions((tx) => {
      handlerRef.current(tx);
    });

    return unsubscribe;
  }, [enabled, subscribeToTransactions]);

  return { isConnected };
}

export function useStatsSubscription(
  handler: (stats: WebSocketStatsUpdate) => void,
  enabled = true
): { isConnected: boolean } {
  const { isConnected, subscribeToStats } = useWebSocket();
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return;

    const unsubscribe = subscribeToStats((stats) => {
      handlerRef.current(stats);
    });

    return unsubscribe;
  }, [enabled, subscribeToStats]);

  return { isConnected };
}

export function useAccountSubscription(
  address: string | null,
  handler: (data: unknown) => void
): { isConnected: boolean } {
  const { isConnected, subscribeToAccount } = useWebSocket();
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!address) return;

    const unsubscribe = subscribeToAccount(address, (data) => {
      handlerRef.current(data);
    });

    return unsubscribe;
  }, [address, subscribeToAccount]);

  return { isConnected };
}
