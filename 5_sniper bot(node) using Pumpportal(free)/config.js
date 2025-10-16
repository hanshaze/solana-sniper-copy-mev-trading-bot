import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

export const config = {
  // Solana Configuration
  solana: {
    rpcUrl: process.env.SOLANA_RPC_URL || 'https://api.mainnet-beta.solana.com',
    privateKey: process.env.PRIVATE_KEY || '',
  },
  
  // PumpPortal Configuration
  pumpportal: {
    apiKey: process.env.PUMPPORTAL_API_KEY || '',
    wsUrl: process.env.PUMPPORTAL_WS_URL || 'wss://api.pumpportal.fun/ws',
  },
  
  // Trading Configuration
  trading: {
    slippageTolerance: parseFloat(process.env.SLIPPAGE_TOLERANCE) || 0.1, // 10%
    maxSolAmount: parseFloat(process.env.MAX_SOL_AMOUNT) || 0.01, // Maximum SOL to spend per trade
    sellDelayMs: parseInt(process.env.SELL_DELAY_MS) || 3000, // 3 seconds delay before selling
    minLiquidity: parseFloat(process.env.MIN_LIQUIDITY) || 1000, // Minimum liquidity required
  },
  
  // Logging
  logging: {
    level: process.env.LOG_LEVEL || 'info', // debug, info, warn, error
  }
};

// Validate required configuration
export function validateConfig() {
  const errors = [];
  
  if (!config.solana.privateKey) {
    errors.push('PRIVATE_KEY is required');
  }
  
  if (!config.pumpportal.apiKey) {
    errors.push('PUMPPORTAL_API_KEY is required');
  }
  
  if (config.trading.maxSolAmount <= 0) {
    errors.push('MAX_SOL_AMOUNT must be greater than 0');
  }
  
  if (config.trading.sellDelayMs < 1000) {
    errors.push('SELL_DELAY_MS must be at least 1000ms');
  }
  
  if (errors.length > 0) {
    throw new Error(`Configuration errors:\n${errors.join('\n')}`);
  }
  
  return true;
}
