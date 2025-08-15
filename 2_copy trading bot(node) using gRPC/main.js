import { newlunched_subscribeCommand, stopNewLaunch } from "./grpc.js";
import { token_buy, token_sell, getSplTokenBalance, getPublicKeyFromPrivateKey } from "./fuc.js";
import { getBalance } from "./swap.js";
import { config, validateConfig } from "./config.js";
import logger from "./utils/logger.js";
import notificationService from "./services/notifications.js";
import riskManager from "./services/riskManager.js";
import chalk from "chalk";

// Trading configuration from config service
const tradingConfig = config.trading;

// Track active positions
const activePositions = new Map();

export const pump_geyser = async () => {
  try {
    // Validate configuration
    validateConfig();
    
    const walletKey = getPublicKeyFromPrivateKey();
    
    // Log startup banner
    console.log(chalk.magentaBright(`
   ██████╗██████╗ ██╗   ██╗██████╗ ████████╗ ██████╗ ██╗  ██╗██╗███╗   ██╗ ██████╗ 
  ██╔════╝██╔══██╗██║   ██║██╔══██╗╚══██╔══╝██╔═══██╗██║ ██╔╝██║████╗  ██║██╔════╝ 
  ██║     ██████╔╝ ██║ ██╔╝██████╔╝   ██║   ██║   ██║█████╔╝ ██║██╔██╗ ██║██║  ██╗ 
  ██║     ██╔══██╗   ██╔═╝ ██╔═══╝    ██║   ██║   ██║██╔═██╗ ██║██║╚██╗██║██║   ██║
  ╚██████╗██║  ██║   ██║   ██║        ██║   ╚██████╔╝██║  ██╗██║██║ ╚████║╚██████╔╝
   ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝        ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝ 
    `));
    
    logger.info("🚀 Starting Solana Raydium Sniper Bot...");
    logger.info(`🔑 Wallet Public Key: ${walletKey}`);
    logger.info(`💰 Sniper Amount: ${tradingConfig.sniperAmount} SOL`);
    logger.info(`🎯 Profit Target: ${tradingConfig.profitTarget}x`);
    logger.info(`🛑 Stop Loss: ${tradingConfig.stopLoss}x`);
    logger.info(`⏱️ Max Hold Time: ${tradingConfig.maxHoldTime/1000}s`);
    logger.info(`📊 Max Positions: ${tradingConfig.maxPositions}`);
    
    // Log configuration
    logger.debug("Trading configuration loaded", tradingConfig);
    logger.debug("Pool filters", config.pools);
    logger.debug("Risk management settings", config.risk);
    
    // Send startup notification
    await notificationService.notifyBotStatus("Started", {
      wallet: walletKey,
      config: tradingConfig,
    });
    
    // Start monitoring for new token launches
    await newlunched_subscribeCommand();
    
    // Set up position monitoring
    setInterval(monitorPositions, 5000); // Check positions every 5 seconds
    
    // Set up risk monitoring
    setInterval(monitorRisk, 10000); // Check risk metrics every 10 seconds
    
    // Set up graceful shutdown
    process.on('SIGINT', async () => {
      logger.warn("🛑 Shutting down sniper bot...");
      stopNewLaunch();
      
      // Close all positions before exit
      await closeAllPositions();
      
      // Send shutdown notification
      await notificationService.notifyBotStatus("Shutdown", {
        reason: "SIGINT received",
        finalStats: riskManager.getDailyStats(),
      });
      
      process.exit(0);
    });
    
    // Handle other shutdown signals
    process.on('SIGTERM', async () => {
      logger.warn("🛑 SIGTERM received, shutting down...");
      await handleGracefulShutdown("SIGTERM");
    });
    
    process.on('uncaughtException', async (error) => {
      logger.error("Uncaught exception", error);
      await notificationService.notifyError(error, "Uncaught Exception");
      await handleGracefulShutdown("Uncaught Exception");
    });
    
    process.on('unhandledRejection', async (reason, promise) => {
      logger.error("Unhandled rejection", { reason, promise });
      await notificationService.notifyError(new Error(reason), "Unhandled Rejection");
    });
    
  } catch (error) {
    logger.error("Error in pump_geyser", error);
    await notificationService.notifyError(error, "Bot Startup");
    throw error;
  }
};

// Monitor active positions for profit taking or stop loss
async function monitorPositions() {
  try {
    const positions = riskManager.getActivePositions();
    
    for (const position of positions) {
      try {
        const currentBalance = await getSplTokenBalance(position.mint);
        if (!currentBalance || currentBalance <= 0) {
          logger.warn(`⚠️ No balance for ${position.mint}, removing from active positions`);
          riskManager.activePositions.delete(position.mint);
          continue;
        }

        // Update position price (you'll need to implement price fetching)
        // const currentPrice = await getCurrentPrice(position.mint);
        // riskManager.updatePositionPrice(position.mint, currentPrice);

        // Check if position should be closed
        const shouldClose = riskManager.shouldClosePosition(position.mint);
        if (shouldClose.shouldClose) {
          logger.info(`Position closure triggered for ${position.mint}: ${shouldClose.reason}`);
          await closePosition(position.mint, position, shouldClose.reason);
        }
      } catch (error) {
        logger.error(`Error monitoring position ${position.mint}`, error);
      }
    }
  } catch (error) {
    logger.error("Error in position monitoring", error);
  }
}

// Monitor risk metrics
async function monitorRisk() {
  try {
    const riskMetrics = riskManager.getRiskMetrics();
    
    // Log risk metrics periodically
    if (riskMetrics.riskLevel === 'HIGH') {
      logger.warn("High risk level detected", riskMetrics);
      await notificationService.sendNotification(
        "🚨 High risk level detected - review positions and consider reducing exposure",
        "warning",
        riskMetrics
      );
    }
    
    // Log daily stats every hour
    const now = new Date();
    if (now.getMinutes() === 0) {
      logger.info("Hourly risk summary", riskMetrics);
    }
    
  } catch (error) {
    logger.error("Error in risk monitoring", error);
  }
}

// Close a specific position
async function closePosition(mint, position, reason = 'manual') {
  try {
    logger.info(`Closing position for ${mint}`, { reason, position });
    
    // Get current token balance
    const currentBalance = await getSplTokenBalance(mint);
    if (!currentBalance || currentBalance <= 0) {
      logger.warn(`No balance to sell for ${mint}`);
      riskManager.activePositions.delete(mint);
      return;
    }

    // Execute sell transaction
    const sellResult = await token_sell(mint, currentBalance);
    
    if (sellResult && sellResult.txHash) {
      logger.info(`Position closed successfully for ${mint}`, {
        txHash: sellResult.txHash,
        reason,
        balance: currentBalance,
      });
      
      // Record the trade
      riskManager.recordTrade('sell', mint, currentBalance, position.currentPrice || 0, sellResult.txHash);
      
      // Send notification
      await notificationService.notifyPositionUpdate('closed', mint, {
        reason,
        txHash: sellResult.txHash,
        balance: currentBalance,
      });
    } else {
      logger.error(`Failed to close position for ${mint}`);
    }
  } catch (error) {
    logger.error(`Error closing position for ${mint}`, error);
    await notificationService.notifyError(error, `Position Closure - ${mint}`);
  }
}

// Close all active positions
async function closeAllPositions() {
  try {
    const positions = riskManager.getActivePositions();
    logger.info(`Closing ${positions.length} active positions...`);
    
    const results = [];
    
    for (const position of positions) {
      try {
        await closePosition(position.mint, position, 'shutdown');
        results.push({ mint: position.mint, status: 'closed' });
      } catch (error) {
        logger.error(`Error closing position ${position.mint}`, error);
        results.push({ mint: position.mint, status: 'error', error: error.message });
      }
    }
    
    logger.info("Position closure summary", { results });
    return results;
  } catch (error) {
    logger.error("Error in closeAllPositions", error);
    throw error;
  }
}

// Handle graceful shutdown
async function handleGracefulShutdown(reason) {
  try {
    logger.warn(`Graceful shutdown initiated: ${reason}`);
    
    // Stop gRPC monitoring
    stopNewLaunch();
    
    // Close all positions
    await closeAllPositions();
    
    // Send final notification
    await notificationService.notifyBotStatus("Shutdown", {
      reason,
      finalStats: riskManager.getDailyStats(),
    });
    
    logger.info("Graceful shutdown completed");
    process.exit(0);
  } catch (error) {
    logger.error("Error during graceful shutdown", error);
    process.exit(1);
  }
}

// Handle new token launch detected by gRPC
async function handleNewTokenLaunch(tokenMint, poolStatus, context) {
  try {
    logger.info(`New token launch detected: ${tokenMint}`, {
      poolStatus,
      context,
      timestamp: new Date().toISOString(),
    });

    // Check if we can execute a trade
    const tradeCheck = riskManager.canExecuteTrade(config.trading.sniperAmount, tokenMint);
    if (!tradeCheck.allowed) {
      logger.warn(`Trade blocked for ${tokenMint}`, { reasons: tradeCheck.errors });
      return;
    }

    // Execute the sniper trade
    logger.info(`Executing sniper trade for ${tokenMint}`);
    
    // Get current SOL balance
    const solBalance = await getBalance();
    if (solBalance < config.trading.sniperAmount) {
      logger.warn(`Insufficient SOL balance for sniper trade: ${solBalance} SOL`);
      return;
    }

    // Execute buy transaction
    const buyResult = await token_buy(tokenMint, config.trading.sniperAmount);
    
    if (buyResult && buyResult.txHash) {
      logger.info(`Sniper trade executed successfully for ${tokenMint}`, {
        txHash: buyResult.txHash,
        amount: config.trading.sniperAmount,
        poolStatus,
      });
      
      // Record the trade
      riskManager.recordTrade('buy', tokenMint, config.trading.sniperAmount, 0, buyResult.txHash);
      
      // Send notification
      await notificationService.notifyTradeExecution('buy', tokenMint, config.trading.sniperAmount, 0, buyResult.txHash);
      
      // Add to active positions for monitoring
      const position = {
        entryPrice: 0, // Will be updated when we get price data
        entryAmount: config.trading.sniperAmount,
        entryTime: Date.now(),
        entryValue: config.trading.sniperAmount,
        currentPrice: 0,
        poolStatus,
        context,
      };
      
      riskManager.activePositions.set(tokenMint, position);
      
      logger.info(`Position opened for ${tokenMint}`, position);
      
    } else {
      logger.error(`Failed to execute sniper trade for ${tokenMint}`);
      await notificationService.notifyError(
        new Error('Buy transaction failed'), 
        `Sniper Trade - ${tokenMint}`
      );
    }
    
  } catch (error) {
    logger.error(`Error handling new token launch for ${tokenMint}`, error);
    await notificationService.notifyError(error, `New Token Launch - ${tokenMint}`);
  }
}

// Export functions for external use
export { closePosition, closeAllPositions, monitorPositions, handleNewTokenLaunch }; 