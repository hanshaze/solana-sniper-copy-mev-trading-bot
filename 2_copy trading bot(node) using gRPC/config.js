import dotenv from 'dotenv';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import fs from 'fs';

// Load environment variables
dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Validate required environment variables
const requiredEnvVars = [
  'PRIVATE_KEY',
  'RPC_URL',
  'GRPC_ENDPOINT',
  'GRPCTOKEN'
];

for (const envVar of requiredEnvVars) {
  if (!process.env[envVar]) {
    throw new Error(`Missing required environment variable: ${envVar}`);
  }
}

// Configuration object
export const config = {
  // Essential Configuration
  wallet: {
    privateKey: process.env.PRIVATE_KEY,
    rpcUrl: process.env.RPC_URL,
    grpcEndpoint: process.env.GRPC_ENDPOINT,
    grpcToken: process.env.GRPCTOKEN,
  },

  // Trading Parameters
  trading: {
    sniperAmount: parseFloat(process.env.SNIPERAMOUNT || '0.1'),
    profitTarget: parseFloat(process.env.PROFIT_TARGET || '2.0'),
    stopLoss: parseFloat(process.env.STOP_LOSS || '0.5'),
    maxHoldTime: parseInt(process.env.MAX_HOLD_TIME || '300000'),
    minLiquidity: parseFloat(process.env.MIN_LIQUIDITY || '10'),
    maxPositions: parseInt(process.env.MAX_POSITIONS || '5'),
    minTxAge: parseInt(process.env.MIN_TX_AGE || '1'),
  },

  // Pool Filters
  pools: {
    pumpFun: process.env.ENABLE_PUMPFUN === 'true',
    pumpSwap: process.env.ENABLE_PUMPSWAP === 'true',
    raydiumLaunchLab: process.env.ENABLE_RAYDIUM_LAUNCHLAB === 'true',
    raydiumCpmm: process.env.ENABLE_RAYDIUM_CPMM === 'true',
  },

  // Swap Configuration
  swap: {
    method: process.env.SWAP_METHOD || 'solana',
    slippageTolerance: parseFloat(process.env.SLIPPAGE_TOLERANCE || '1.0'),
    priorityFee: parseInt(process.env.PRIORITY_FEE || '1000'),
  },

  // Risk Management
  risk: {
    maxDailyLoss: parseFloat(process.env.MAX_DAILY_LOSS || '1.0'),
    maxSingleLoss: parseFloat(process.env.MAX_SINGLE_LOSS || '0.5'),
    tradeCooldown: parseInt(process.env.TRADE_COOLDOWN || '5000'),
  },

  // Notifications
  notifications: {
    telegram: {
      botToken: process.env.TELEGRAM_BOT_TOKEN,
      chatId: process.env.TELEGRAM_CHAT_ID,
    },
    email: {
      host: process.env.SMTP_HOST,
      port: parseInt(process.env.SMTP_PORT || '587'),
      user: process.env.SMTP_USER,
      pass: process.env.SMTP_PASS,
    },
    discord: {
      webhookUrl: process.env.DISCORD_WEBHOOK_URL,
    },
  },

  // Logging & Monitoring
  logging: {
    level: process.env.LOG_LEVEL || 'info',
    debug: process.env.DEBUG === 'true',
    logToFile: process.env.LOG_TO_FILE === 'true',
    logFilePath: process.env.LOG_FILE_PATH || './logs/trading-bot.log',
  },

  // API Keys
  api: {
    helius: process.env.HELIUS_API_KEY,
    jupiter: process.env.JUPITER_API_KEY,
  },

  // Advanced Settings
  advanced: {
    mevProtection: process.env.ENABLE_MEV_PROTECTION === 'true',
    backtestMode: process.env.BACKTEST_MODE === 'true',
    paperTrading: process.env.PAPER_TRADING === 'true',
    databaseUrl: process.env.DATABASE_URL || 'sqlite://./trades.db',
  },

  // Performance
  performance: {
    maxRetries: parseInt(process.env.MAX_RETRIES || '3'),
    retryDelay: parseInt(process.env.RETRY_DELAY || '1000'),
    connectionTimeout: parseInt(process.env.CONNECTION_TIMEOUT || '30000'),
  },

  // Security
  security: {
    rateLimiting: process.env.ENABLE_RATE_LIMITING === 'true',
    maxRequestsPerMinute: parseInt(process.env.MAX_REQUESTS_PER_MINUTE || '100'),
    ipWhitelist: process.env.ENABLE_IP_WHITELIST === 'false',
    allowedIps: process.env.ALLOWED_IPS ? process.env.ALLOWED_IPS.split(',') : ['127.0.0.1', '::1'],
  },
};

// Create logs directory if it doesn't exist
const logsDir = dirname(config.logging.logFilePath);
if (!fs.existsSync(logsDir)) {
  fs.mkdirSync(logsDir, { recursive: true });
}

// Validation functions
export const validateConfig = () => {
  const errors = [];

  if (config.trading.sniperAmount <= 0) {
    errors.push('SNIPERAMOUNT must be greater than 0');
  }

  if (config.trading.profitTarget <= 1.0) {
    errors.push('PROFIT_TARGET must be greater than 1.0');
  }

  if (config.trading.stopLoss >= 1.0) {
    errors.push('STOP_LOSS must be less than 1.0');
  }

  if (config.trading.maxHoldTime <= 0) {
    errors.push('MAX_HOLD_TIME must be greater than 0');
  }

  if (config.trading.minLiquidity <= 0) {
    errors.push('MIN_LIQUIDITY must be greater than 0');
  }

  if (errors.length > 0) {
    throw new Error(`Configuration validation failed:\n${errors.join('\n')}`);
  }

  return true;
};

// Get configuration for specific module
export const getModuleConfig = (moduleName) => {
  switch (moduleName) {
    case 'trading':
      return config.trading;
    case 'swap':
      return config.swap;
    case 'risk':
      return config.risk;
    case 'notifications':
      return config.notifications;
    case 'logging':
      return config.logging;
    default:
      return config;
  }
};

// Export default configuration
export default config;
