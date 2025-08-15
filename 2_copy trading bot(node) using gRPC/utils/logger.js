import winston from 'winston';
import DailyRotateFile from 'winston-daily-rotate-file';
import chalk from 'chalk';
import { config } from '../config.js';

// Custom format for console output
const consoleFormat = winston.format.combine(
  winston.format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
  winston.format.printf(({ timestamp, level, message, ...meta }) => {
    const levelColors = {
      error: chalk.red,
      warn: chalk.yellow,
      info: chalk.blue,
      debug: chalk.gray,
      success: chalk.green,
      trade: chalk.cyan,
      profit: chalk.green,
      loss: chalk.red,
    };

    const levelIcon = {
      error: '❌',
      warn: '⚠️',
      info: 'ℹ️',
      debug: '🔍',
      success: '✅',
      trade: '💰',
      profit: '📈',
      loss: '📉',
    };

    const color = levelColors[level] || chalk.white;
    const icon = levelIcon[level] || '📝';
    
    return `${chalk.gray(timestamp)} ${icon} ${color(level.toUpperCase())}: ${message}`;
  })
);

// File format for logging to files
const fileFormat = winston.format.combine(
  winston.format.timestamp(),
  winston.format.errors({ stack: true }),
  winston.format.json()
);

// Create logger instance
const logger = winston.createLogger({
  level: config.logging.level,
  format: fileFormat,
  defaultMeta: { service: 'solana-trading-bot' },
  transports: [
    // Daily rotate file transport
    new DailyRotateFile({
      filename: config.logging.logFilePath.replace('.log', '-%DATE%.log'),
      datePattern: 'YYYY-MM-DD',
      zippedArchive: true,
      maxSize: '20m',
      maxFiles: '14d',
      level: 'info',
    }),
    
    // Error log file
    new DailyRotateFile({
      filename: config.logging.logFilePath.replace('.log', '-error-%DATE%.log'),
      datePattern: 'YYYY-MM-DD',
      zippedArchive: true,
      maxSize: '20m',
      maxFiles: '30d',
      level: 'error',
    }),
  ],
});

// Add console transport if not in production
if (config.logging.debug || process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: consoleFormat,
    level: config.logging.debug ? 'debug' : 'info',
  }));
}

// Custom logging methods
export const logTrade = (message, meta = {}) => {
  logger.log('trade', message, meta);
};

export const logProfit = (message, meta = {}) => {
  logger.log('profit', message, meta);
};

export const logLoss = (message, meta = {}) => {
  logger.log('loss', message, meta);
};

export const logSuccess = (message, meta = {}) => {
  logger.log('success', message, meta);
};

// Performance logging
export const logPerformance = (operation, duration, meta = {}) => {
  logger.info(`Performance: ${operation} took ${duration}ms`, {
    operation,
    duration,
    ...meta,
  });
};

// Trade execution logging
export const logTradeExecution = (tradeType, tokenMint, amount, price, meta = {}) => {
  logger.info(`Trade Execution: ${tradeType} ${tokenMint}`, {
    tradeType,
    tokenMint,
    amount,
    price,
    timestamp: new Date().toISOString(),
    ...meta,
  });
};

// Error logging with context
export const logError = (error, context = '', meta = {}) => {
  logger.error(`Error in ${context}: ${error.message}`, {
    error: error.stack,
    context,
    timestamp: new Date().toISOString(),
    ...meta,
  });
};

// Wallet balance logging
export const logBalance = (balance, token = 'SOL') => {
  logger.info(`Wallet Balance: ${balance} ${token}`);
};

// Position logging
export const logPosition = (action, mint, details, meta = {}) => {
  logger.info(`Position ${action}: ${mint}`, {
    action,
    mint,
    details,
    timestamp: new Date().toISOString(),
    ...meta,
  });
};

// Configuration logging
export const logConfig = (configSection, values) => {
  logger.debug(`Configuration ${configSection}:`, values);
};

// API request logging
export const logApiRequest = (method, endpoint, params = {}) => {
  logger.debug(`API Request: ${method} ${endpoint}`, {
    method,
    endpoint,
    params,
    timestamp: new Date().toISOString(),
  });
};

// API response logging
export const logApiResponse = (method, endpoint, status, duration, meta = {}) => {
  logger.debug(`API Response: ${method} ${endpoint}`, {
    method,
    endpoint,
    status,
    duration,
    timestamp: new Date().toISOString(),
    ...meta,
  });
};

// Export the main logger
export default logger;
