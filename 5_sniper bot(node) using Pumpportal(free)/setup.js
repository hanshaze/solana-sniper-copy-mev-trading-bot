#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log('🚀 Setting up Solana PumpFun Sniper Bot...\n');

// Check if .env exists
const envPath = path.join(__dirname, '.env');
if (!fs.existsSync(envPath)) {
  console.log('📝 Creating .env file from template...');
  
  const envContent = `# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
PRIVATE_KEY=your_private_key_here

# PumpPortal Configuration
PUMPPORTAL_API_KEY=your_pumpportal_api_key_here
PUMPPORTAL_WS_URL=wss://api.pumpportal.fun/ws

# Trading Configuration
SLIPPAGE_TOLERANCE=0.1
MAX_SOL_AMOUNT=0.01
SELL_DELAY_MS=3000
MIN_LIQUIDITY=1000

# Logging
LOG_LEVEL=info
`;

  fs.writeFileSync(envPath, envContent);
  console.log('✅ .env file created');
} else {
  console.log('✅ .env file already exists');
}

console.log('\n📋 Next steps:');
console.log('1. Edit .env file with your configuration:');
console.log('   - Add your Solana private key');
console.log('   - Add your PumpPortal API key');
console.log('   - Adjust trading parameters if needed');
console.log('\n2. Install dependencies:');
console.log('   npm install');
console.log('\n3. Start the bot:');
console.log('   npm start');
console.log('\n⚠️  Remember: Only trade with amounts you can afford to lose!');
console.log('📚 Read README.md for detailed instructions and safety tips.\n');
