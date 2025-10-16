import { SniperBot } from './src/sniper-bot.js';
import { config, validateConfig } from './config.js';
import { logger } from './src/logger.js';
import { ErrorHandler } from './src/error-handler.js';

async function main() {
  try {
    // Validate configuration
    validateConfig();
    logger.info('✅ Configuration validated');
    
    // Create and start the sniper bot
    const bot = new SniperBot(config);
    
    // Set up graceful shutdown
    process.on('SIGINT', async () => {
      logger.info('🛑 Received SIGINT, shutting down gracefully...');
      await bot.stop();
      process.exit(0);
    });
    
    process.on('SIGTERM', async () => {
      logger.info('🛑 Received SIGTERM, shutting down gracefully...');
      await bot.stop();
      process.exit(0);
    });
    
    // Handle uncaught exceptions
    process.on('uncaughtException', async (error) => {
      logger.error('❌ Uncaught Exception:', error);
      await bot.stop();
      process.exit(1);
    });
    
    process.on('unhandledRejection', async (reason, promise) => {
      logger.error('❌ Unhandled Rejection at:', promise, 'reason:', reason);
      await bot.stop();
      process.exit(1);
    });
    
    // Start the bot
    await bot.start();
    
    // Print stats every 30 seconds
    setInterval(() => {
      bot.printStats();
    }, 30000);
    
    logger.info('🚀 Sniper bot is running! Press Ctrl+C to stop.');
    
  } catch (error) {
    const friendlyMessage = ErrorHandler.handle(error, 'Main');
    logger.error('❌ Failed to start bot:', friendlyMessage);
    process.exit(1);
  }
}

// Start the application
main().catch(error => {
  logger.error('❌ Fatal error:', error);
  process.exit(1);
});
