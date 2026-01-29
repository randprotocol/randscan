import type {
  WebSocketChannel,
  WebSocketMessage,
  WebSocketBlockUpdate,
  WebSocketTransactionUpdate,
  WebSocketStatsUpdate,
} from '@/types';

type MessageHandler = (data: unknown) => void;
type ConnectionHandler = () => void;
type ErrorHandler = (error: Event) => void;

interface Subscription {
  channel: WebSocketChannel;
  params?: Record<string, string>;
  handler: MessageHandler;
}

class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private subscriptions: Map<string, Subscription> = new Map();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private isConnecting = false;
  private onConnectHandlers: Set<ConnectionHandler> = new Set();
  private onDisconnectHandlers: Set<ConnectionHandler> = new Set();
  private onErrorHandlers: Set<ErrorHandler> = new Set();

  constructor(url?: string) {
    const wsUrl = url || process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3000';
    this.url = `${wsUrl}/ws`;
  }

  private getSubscriptionKey(channel: WebSocketChannel, params?: Record<string, string>): string {
    if (!params || Object.keys(params).length === 0) {
      return channel;
    }
    const sortedParams = Object.entries(params)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([k, v]) => `${k}=${v}`)
      .join('&');
    return `${channel}:${sortedParams}`;
  }

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        resolve();
        return;
      }

      if (this.isConnecting) {
        const checkConnection = setInterval(() => {
          if (this.ws?.readyState === WebSocket.OPEN) {
            clearInterval(checkConnection);
            resolve();
          }
        }, 100);
        return;
      }

      this.isConnecting = true;

      try {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.onConnectHandlers.forEach(handler => handler());

          // Resubscribe to all channels
          this.subscriptions.forEach(sub => {
            this.sendSubscribe(sub.channel, sub.params);
          });

          resolve();
        };

        this.ws.onclose = () => {
          this.isConnecting = false;
          this.onDisconnectHandlers.forEach(handler => handler());
          this.handleReconnect();
        };

        this.ws.onerror = (error) => {
          this.isConnecting = false;
          this.onErrorHandlers.forEach(handler => handler(error));
          reject(error);
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  private handleMessage(data: string): void {
    try {
      const message: WebSocketMessage = JSON.parse(data);
      const key = this.getSubscriptionKey(message.channel, message.params as Record<string, string>);
      const subscription = this.subscriptions.get(key);

      if (subscription && message.action === 'update') {
        subscription.handler(message.data);
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }

  private handleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    setTimeout(() => {
      this.connect().catch(console.error);
    }, delay);
  }

  private sendSubscribe(channel: WebSocketChannel, params?: Record<string, string>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      const message: WebSocketMessage = {
        channel,
        action: 'subscribe',
        params,
      };
      this.ws.send(JSON.stringify(message));
    }
  }

  private sendUnsubscribe(channel: WebSocketChannel, params?: Record<string, string>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      const message: WebSocketMessage = {
        channel,
        action: 'unsubscribe',
        params,
      };
      this.ws.send(JSON.stringify(message));
    }
  }

  subscribe(
    channel: WebSocketChannel,
    handler: MessageHandler,
    params?: Record<string, string>
  ): () => void {
    const key = this.getSubscriptionKey(channel, params);

    this.subscriptions.set(key, { channel, params, handler });

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.sendSubscribe(channel, params);
    } else {
      this.connect().catch(console.error);
    }

    // Return unsubscribe function
    return () => {
      this.subscriptions.delete(key);
      this.sendUnsubscribe(channel, params);
    };
  }

  // Convenience methods for typed subscriptions
  subscribeToBlocks(handler: (block: WebSocketBlockUpdate) => void): () => void {
    return this.subscribe('blocks', handler as MessageHandler);
  }

  subscribeToTransactions(handler: (tx: WebSocketTransactionUpdate) => void): () => void {
    return this.subscribe('transactions', handler as MessageHandler);
  }

  subscribeToStats(handler: (stats: WebSocketStatsUpdate) => void): () => void {
    return this.subscribe('stats', handler as MessageHandler);
  }

  subscribeToAccount(
    address: string,
    handler: MessageHandler
  ): () => void {
    return this.subscribe('account', handler, { address });
  }

  subscribeToValidator(
    identity: string,
    handler: MessageHandler
  ): () => void {
    return this.subscribe('validator', handler, { identity });
  }

  onConnect(handler: ConnectionHandler): () => void {
    this.onConnectHandlers.add(handler);
    return () => this.onConnectHandlers.delete(handler);
  }

  onDisconnect(handler: ConnectionHandler): () => void {
    this.onDisconnectHandlers.add(handler);
    return () => this.onDisconnectHandlers.delete(handler);
  }

  onError(handler: ErrorHandler): () => void {
    this.onErrorHandlers.add(handler);
    return () => this.onErrorHandlers.delete(handler);
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.subscriptions.clear();
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

// Singleton instance
let wsClient: WebSocketClient | null = null;

export function getWebSocketClient(): WebSocketClient {
  if (!wsClient) {
    wsClient = new WebSocketClient();
  }
  return wsClient;
}

export { WebSocketClient };
