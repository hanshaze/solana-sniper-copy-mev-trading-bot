import { PumpPortalClient } from './pumpportal.js';
import { SolanaTrader } from './solana-trader.js';

export class SniperBot {
  constructor(config) {
    this.config = config;
    this.pumpPortal = new PumpPortalClient(
      config.pumpportal.apiKey,
      config.pumpportal.wsUrl
    );
    this.trader = new SolanaTrader(
      config.solana.rpcUrl,
      config.solana.privateKey
    );
    this.activeTrades = new Map();
    this.isRunning = false;
    this.stats = {
      totalMints: 0,
      successfulBuys: 0,
      successfulSells: 0,
      failedTrades: 0,
      totalProfit: 0
    };
  }

  async start() {
    try {
      console.log('🚀 Starting Solana Sniper Bot...');
      
      // Check wallet balance
      const balance = await this.trader.getWalletBalance();
      console.log(`💰 Wallet balance: ${balance} SOL`);
      
      if (balance < this.config.trading.maxSolAmount) {
        throw new Error(`Insufficient balance. Need at least ${this.config.trading.maxSolAmount} SOL`);
      }

      // Connect to PumpPortal
      await this.pumpPortal.connect();
      
      // Set up event handlers
      this.pumpPortal.setOnMint(this.handleMintEvent.bind(this));
      
      // Subscribe to mint events
      await this.pumpPortal.subscribeToMints();
      
      this.isRunning = true;
      console.log('✅ Sniper bot started successfully!');
      
      // Start ping interval to keep connection alive
      this.startPingInterval();
      
    } catch (error) {
      console.error('❌ Failed to start sniper bot:', error);
      throw error;
    }
  }

  async stop() {
    try {
      console.log('🛑 Stopping sniper bot...');
      this.isRunning = false;
      
      // Close all active trades
      for (const [mintAddress, trade] of this.activeTrades) {
        await this.closeTrade(mintAddress, 'Bot stopping');
      }
      
      // Disconnect from PumpPortal
      await this.pumpPortal.disconnect();
      
      console.log('✅ Sniper bot stopped');
    } catch (error) {
      console.error('❌ Error stopping sniper bot:', error);
    }
  }

  async handleMintEvent(mintData) {
    try {
      this.stats.totalMints++;
      const mintAddress = mintData.mint;
      
      console.log(`🪙 New mint detected: ${mintAddress}`);
      console.log(`📊 Creator: ${mintData.creator}`);
      console.log(`📈 Metadata:`, mintData.metadata);
      
      // Validate the token
      if (!await this.validateToken(mintData)) {
        console.log(`❌ Token validation failed for ${mintAddress}`);
        return;
      }
      
      // Check if we already have an active trade for this token
      if (this.activeTrades.has(mintAddress)) {
        console.log(`⚠️ Already trading ${mintAddress}, skipping`);
        return;
      }
      
      // Execute buy order
      await this.executeBuy(mintAddress, mintData);
      
    } catch (error) {
      console.error('❌ Error handling mint event:', error);
      this.stats.failedTrades++;
    }
  }

  async validateToken(mintData) {
    try {
      const mintAddress = mintData.mint;
      
      // Check if it's a valid PumpFun token
      const isPumpFunToken = await this.trader.isPumpFunToken(mintAddress);
      if (!isPumpFunToken) {
        console.log(`❌ ${mintAddress} is not a valid PumpFun token`);
        return false;
      }
      
      // Check minimum liquidity (if available in metadata)
      if (mintData.metadata && mintData.metadata.liquidity) {
        if (mintData.metadata.liquidity < this.config.trading.minLiquidity) {
          console.log(`❌ Insufficient liquidity: ${mintData.metadata.liquidity}`);
          return false;
        }
      }
      
      // Additional validation can be added here
      // e.g., check for honeypot, rug pull indicators, etc.
      
      return true;
    } catch (error) {
      console.error('❌ Error validating token:', error);
      return false;
    }
  }

  async executeBuy(mintAddress, mintData) {
    try {
      console.log(`🛒 Executing buy order for ${mintAddress}`);
      
      const buyResult = await this.trader.buyToken(
        mintAddress,
        this.config.trading.maxSolAmount,
        this.config.trading.slippageTolerance
      );
      
      if (!buyResult.success) {
        console.error(`❌ Buy failed for ${mintAddress}:`, buyResult.error);
        this.stats.failedTrades++;
        return;
      }
      
      this.stats.successfulBuys++;
      console.log(`✅ Buy successful for ${mintAddress}`);
      console.log(`📝 Transaction: ${buyResult.signature}`);
      
      // Store trade information
      const trade = {
        mintAddress,
        buySignature: buyResult.signature,
        tokenAccount: buyResult.tokenAccount,
        buyTime: Date.now(),
        buyPrice: this.config.trading.maxSolAmount,
        status: 'bought'
      };
      
      this.activeTrades.set(mintAddress, trade);
      
      // Subscribe to token updates
      await this.pumpPortal.subscribeToToken(mintAddress);
      
      // Schedule sell after delay
      setTimeout(() => {
        this.executeSell(mintAddress);
      }, this.config.trading.sellDelayMs);
      
      console.log(`⏰ Sell scheduled for ${mintAddress} in ${this.config.trading.sellDelayMs}ms`);
      
    } catch (error) {
      console.error('❌ Error executing buy:', error);
      this.stats.failedTrades++;
    }
  }

  async executeSell(mintAddress) {
    try {
      const trade = this.activeTrades.get(mintAddress);
      if (!trade) {
        console.log(`⚠️ No active trade found for ${mintAddress}`);
        return;
      }
      
      console.log(`💰 Executing sell order for ${mintAddress}`);
      
      const sellResult = await this.trader.sellToken(
        mintAddress,
        trade.tokenAccount,
        this.config.trading.slippageTolerance
      );
      
      if (!sellResult.success) {
        console.error(`❌ Sell failed for ${mintAddress}:`, sellResult.error);
        this.stats.failedTrades++;
        return;
      }
      
      this.stats.successfulSells++;
      console.log(`✅ Sell successful for ${mintAddress}`);
      console.log(`📝 Transaction: ${sellResult.signature}`);
      
      // Update trade status
      trade.status = 'sold';
      trade.sellSignature = sellResult.signature;
      trade.sellTime = Date.now();
      
      // Calculate profit (simplified)
      const profit = await this.calculateProfit(mintAddress);
      this.stats.totalProfit += profit;
      
      console.log(`💰 Trade completed for ${mintAddress}, Profit: ${profit} SOL`);
      
      // Clean up
      await this.closeTrade(mintAddress, 'Trade completed');
      
    } catch (error) {
      console.error('❌ Error executing sell:', error);
      this.stats.failedTrades++;
    }
  }

  async calculateProfit(mintAddress) {
    try {
      // This is a simplified profit calculation
      // In reality, you'd need to track the actual token amounts and prices
      const trade = this.activeTrades.get(mintAddress);
      if (!trade) return 0;
      
      // For now, return a placeholder calculation
      // You would need to implement proper price tracking
      return 0; // Placeholder
    } catch (error) {
      console.error('❌ Error calculating profit:', error);
      return 0;
    }
  }

  async closeTrade(mintAddress, reason) {
    try {
      console.log(`🔒 Closing trade for ${mintAddress}: ${reason}`);
      
      // Unsubscribe from token updates
      await this.pumpPortal.unsubscribeFromToken(mintAddress);
      
      // Remove from active trades
      this.activeTrades.delete(mintAddress);
      
    } catch (error) {
      console.error('❌ Error closing trade:', error);
    }
  }

  startPingInterval() {
    setInterval(async () => {
      if (this.isRunning) {
        try {
          await this.pumpPortal.ping();
        } catch (error) {
          console.error('❌ Ping failed:', error);
        }
      }
    }, 30000); // Ping every 30 seconds
  }

  getStats() {
    return {
      ...this.stats,
      activeTrades: this.activeTrades.size,
      isRunning: this.isRunning
    };
  }

  printStats() {
    const stats = this.getStats();
    console.log('\n📊 Bot Statistics:');
    console.log(`   Total Mints: ${stats.totalMints}`);
    console.log(`   Successful Buys: ${stats.successfulBuys}`);
    console.log(`   Successful Sells: ${stats.successfulSells}`);
    console.log(`   Failed Trades: ${stats.failedTrades}`);
    console.log(`   Active Trades: ${stats.activeTrades}`);
    console.log(`   Total Profit: ${stats.totalProfit} SOL`);
    console.log(`   Status: ${stats.isRunning ? 'Running' : 'Stopped'}\n`);
  }
}
