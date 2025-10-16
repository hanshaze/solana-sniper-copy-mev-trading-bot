import WebSocket from 'ws';
import axios from 'axios';

export class PumpPortalClient {
  constructor(apiKey, wsUrl) {
    this.apiKey = apiKey;
    this.wsUrl = wsUrl;
    this.ws = null;
    this.isConnected = false;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 1000;
    this.subscriptions = new Set();
  }

  async connect() {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.wsUrl, {
          headers: {
            'Authorization': `Bearer ${this.apiKey}`,
            'User-Agent': 'Solana-Sniper-Bot/1.0.0'
          }
        });

        this.ws.on('open', () => {
          console.log('✅ Connected to PumpPortal WebSocket');
          this.isConnected = true;
          this.reconnectAttempts = 0;
          resolve();
        });

        this.ws.on('message', (data) => {
          try {
            const message = JSON.parse(data.toString());
            this.handleMessage(message);
          } catch (error) {
            console.error('❌ Error parsing WebSocket message:', error);
          }
        });

        this.ws.on('close', (code, reason) => {
          console.log(`🔌 WebSocket closed: ${code} - ${reason}`);
          this.isConnected = false;
          this.handleReconnect();
        });

        this.ws.on('error', (error) => {
          console.error('❌ WebSocket error:', error);
          this.isConnected = false;
          reject(error);
        });

      } catch (error) {
        reject(error);
      }
    });
  }

  handleMessage(message) {
    console.log('📨 Received message:', JSON.stringify(message, null, 2));
    
    // Handle different message types
    switch (message.type) {
      case 'mint':
        this.handleMintEvent(message.data);
        break;
      case 'pong':
        console.log('🏓 Pong received');
        break;
      case 'error':
        console.error('❌ PumpPortal error:', message.error);
        break;
      default:
        console.log('📋 Unknown message type:', message.type);
    }
  }

  handleMintEvent(data) {
    console.log('🪙 Mint event detected:', {
      mint: data.mint,
      creator: data.creator,
      timestamp: new Date().toISOString(),
      metadata: data.metadata
    });

    // Emit mint event for the main bot to handle
    if (this.onMint) {
      this.onMint(data);
    }
  }

  async subscribeToMints() {
    if (!this.isConnected) {
      throw new Error('WebSocket not connected');
    }

    const subscription = {
      type: 'subscribe',
      channel: 'mints',
      params: {
        // Subscribe to all new mints on PumpFun
        platform: 'pumpfun'
      }
    };

    this.ws.send(JSON.stringify(subscription));
    this.subscriptions.add('mints');
    console.log('📡 Subscribed to mint events');
  }

  async subscribeToToken(mintAddress) {
    if (!this.isConnected) {
      throw new Error('WebSocket not connected');
    }

    const subscription = {
      type: 'subscribe',
      channel: 'token',
      params: {
        mint: mintAddress
      }
    };

    this.ws.send(JSON.stringify(subscription));
    this.subscriptions.add(`token:${mintAddress}`);
    console.log(`📡 Subscribed to token: ${mintAddress}`);
  }

  async unsubscribeFromToken(mintAddress) {
    if (!this.isConnected) {
      return;
    }

    const unsubscription = {
      type: 'unsubscribe',
      channel: 'token',
      params: {
        mint: mintAddress
      }
    };

    this.ws.send(JSON.stringify(unsubscription));
    this.subscriptions.delete(`token:${mintAddress}`);
    console.log(`📡 Unsubscribed from token: ${mintAddress}`);
  }

  handleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('❌ Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    
    console.log(`🔄 Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
    
    setTimeout(() => {
      this.connect().then(() => {
        // Re-subscribe to all previous subscriptions
        this.resubscribe();
      }).catch(error => {
        console.error('❌ Reconnection failed:', error);
        this.handleReconnect();
      });
    }, delay);
  }

  resubscribe() {
    console.log('🔄 Re-subscribing to channels...');
    for (const subscription of this.subscriptions) {
      if (subscription === 'mints') {
        this.subscribeToMints();
      } else if (subscription.startsWith('token:')) {
        const mintAddress = subscription.split(':')[1];
        this.subscribeToToken(mintAddress);
      }
    }
  }

  async ping() {
    if (!this.isConnected) {
      throw new Error('WebSocket not connected');
    }

    const ping = {
      type: 'ping',
      timestamp: Date.now()
    };

    this.ws.send(JSON.stringify(ping));
  }

  async disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
      this.isConnected = false;
      console.log('🔌 Disconnected from PumpPortal');
    }
  }

  // Set event handlers
  setOnMint(handler) {
    this.onMint = handler;
  }
}
