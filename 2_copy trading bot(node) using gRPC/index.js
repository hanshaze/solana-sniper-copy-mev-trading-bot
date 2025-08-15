
import { pump_geyser } from "./main.js";
import { config, validateConfig } from "./config.js";
import logger from "./utils/logger.js";
import notificationService from "./services/notifications.js";
import riskManager from "./services/riskManager.js";

// Start dashboard server if enabled
let dashboardServer = null;
if (process.env.ENABLE_DASHBOARD === 'true') {
  try {
    const dashboardApp = await import("./dashboard/server.js");
    dashboardServer = dashboardApp.default;
    logger.info("Dashboard server started");
  } catch (error) {
    logger.warn("Failed to start dashboard server", error);
  }
}

// Validate configuration before starting
try {
  validateConfig();
  logger.info("Configuration validated successfully");
} catch (error) {
  logger.error("Configuration validation failed", error);
  process.exit(1);
}

// Check wallet balance
const checkWalletBalance = async () => {
  try {
    const { getBalance } = await import("./swap.js");
    const balance = await getBalance();
    
    if (balance < 1) {
      logger.error("Wallet balance is below 1 SOL. Current balance:", balance, "SOL");
      await notificationService.sendNotification(
        `❌ Insufficient wallet balance: ${balance} SOL`,
        'error',
        { balance, required: 1 }
      );
      process.exit(1);
    }
    
    logger.info(`Wallet balance: ${balance} SOL`);
    await notificationService.sendNotification(
      `✅ Bot startup successful. Wallet balance: ${balance} SOL`,
      'success',
      { balance }
    );
    
    return balance;
  } catch (err) {
    logger.error("Error checking wallet balance", err);
    await notificationService.sendNotification(
      `❌ Failed to check wallet balance: ${err.message}`,
      'error',
      { error: err.message }
    );
    process.exit(1);
  }
};

// Main startup function
const main = async () => {
  try {
    logger.info("🚀 Starting Solana Trading Bot...");
    
    // Check wallet balance
    await checkWalletBalance();
    
    // Start the main bot
    await pump_geyser();
    
    // Send startup notification
    await notificationService.notifyBotStatus("Started", {
      timestamp: new Date().toISOString(),
      config: {
        trading: config.trading,
        risk: config.risk,
        pools: config.pools,
      },
    });
    
    logger.info("✅ Bot startup completed successfully");
    
  } catch (error) {
    logger.error("❌ Bot startup failed", error);
    await notificationService.notifyError(error, "Bot Startup");
    process.exit(1);
  }
};

// Handle process signals
process.on('SIGINT', async () => {
  logger.info("🛑 SIGINT received, shutting down...");
  await handleShutdown("SIGINT");
});

process.on('SIGTERM', async () => {
  logger.info("🛑 SIGTERM received, shutting down...");
  await handleShutdown("SIGTERM");
});

process.on('uncaughtException', async (error) => {
  logger.error("Uncaught exception", error);
  await notificationService.notifyError(error, "Uncaught Exception");
  await handleShutdown("Uncaught Exception");
});

process.on('unhandledRejection', async (reason, promise) => {
  logger.error("Unhandled rejection", { reason, promise });
  await notificationService.notifyError(new Error(reason), "Unhandled Rejection");
});

// Graceful shutdown handler
const handleShutdown = async (reason) => {
  try {
    logger.info("Initiating graceful shutdown...");
    
    // Get final statistics
    const finalStats = riskManager.getDailyStats();
    const riskMetrics = riskManager.getRiskMetrics();
    
    // Send shutdown notification
    await notificationService.notifyBotStatus("Shutdown", {
      reason,
      finalStats,
      riskMetrics,
      timestamp: new Date().toISOString(),
    });
    
    // Close dashboard server if running
    if (dashboardServer) {
      logger.info("Closing dashboard server...");
      // Note: In a real implementation, you'd want to properly close the Express server
    }
    
    logger.info("Graceful shutdown completed");
    process.exit(0);
    
  } catch (error) {
    logger.error("Error during shutdown", error);
    process.exit(1);
  }
};

// Start the bot
main().catch(async (error) => {
  logger.error("Fatal error in main function", error);
  await notificationService.notifyError(error, "Main Function");
  process.exit(1);
});

