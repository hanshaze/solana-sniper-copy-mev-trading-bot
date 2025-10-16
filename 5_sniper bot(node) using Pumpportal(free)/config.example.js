export const config = {
  // Solana Configuration
  solana: {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    privateKey: 'your_private_key_here', // Base58 encoded private key
  },
  
  // PumpPortal Configuration
  pumpportal: {
    apiKey: 'your_pumpportal_api_key_here',
    wsUrl: 'wss://api.pumpportal.fun/ws',
  },
  
  // Trading Configuration
  trading: {
    slippageTolerance: 0.1, // 10%
    maxSolAmount: 0.01, // Maximum SOL to spend per trade
    sellDelayMs: 3000, // 3 seconds delay before selling
    minLiquidity: 1000, // Minimum liquidity required
  },
  
  // Logging
  logging: {
    level: 'info', // debug, info, warn, error
  }
};
