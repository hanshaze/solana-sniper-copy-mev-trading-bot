# Solana PumpFun Sniper Bot

A high-performance Solana sniper bot that automatically detects new token mints on PumpFun and executes buy/sell trades with configurable timing.

## Features

- 🚀 **Real-time mint detection** using PumpPortal API
- ⚡ **Fast execution** with optimized Solana transactions
- ⏱️ **Configurable sell delay** (default: 3 seconds)
- 🛡️ **Error handling** with automatic retries
- 📊 **Live statistics** and trade monitoring
- 🔧 **Easy configuration** via environment variables

## Prerequisites

- Node.js 18+ 
- A Solana wallet with SOL for trading
- PumpPortal API key (free tier available)

## Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd solana-pumpfun-sniper-bot
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Set up environment variables**
   ```bash
   cp config.example.js .env
   ```
   
   Edit `.env` with your configuration:
   ```env
   # Solana Configuration
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
   ```

## Configuration

### Required Settings

- **PRIVATE_KEY**: Your Solana wallet private key (Base58 encoded)
- **PUMPPORTAL_API_KEY**: Your PumpPortal API key

### Trading Settings

- **MAX_SOL_AMOUNT**: Maximum SOL to spend per trade (default: 0.01)
- **SELL_DELAY_MS**: Delay before selling in milliseconds (default: 3000)
- **SLIPPAGE_TOLERANCE**: Slippage tolerance as decimal (default: 0.1 = 10%)
- **MIN_LIQUIDITY**: Minimum liquidity required for trading (default: 1000)

## Usage

### Start the bot
```bash
npm start
```

### Development mode (with auto-restart)
```bash
npm run dev
```

## How It Works

1. **Connection**: Bot connects to PumpPortal WebSocket API
2. **Subscription**: Subscribes to new mint events on PumpFun
3. **Detection**: When a new token is minted, the bot validates it
4. **Buy**: Executes a buy order for the configured SOL amount
5. **Wait**: Waits for the configured delay (default: 3 seconds)
6. **Sell**: Executes a sell order to close the position
7. **Monitor**: Tracks statistics and profit/loss

## Safety Features

- ✅ **Balance validation** before trading
- ✅ **Token validation** to avoid honeypots
- ✅ **Slippage protection** to prevent bad trades
- ✅ **Error handling** with automatic retries
- ✅ **Graceful shutdown** on interruption

## Monitoring

The bot provides real-time statistics:
- Total mints detected
- Successful buy/sell operations
- Failed trades
- Active trades
- Total profit/loss

## Important Notes

⚠️ **Risk Warning**: This bot is for educational purposes. Trading cryptocurrencies involves significant risk. Only trade with amounts you can afford to lose.

⚠️ **API Limits**: PumpPortal has rate limits. Monitor your usage to avoid hitting limits.

⚠️ **Network Fees**: Solana network fees will be deducted from your trades.

## Troubleshooting

### Common Issues

1. **"Insufficient funds"**
   - Add more SOL to your wallet
   - Reduce MAX_SOL_AMOUNT

2. **"WebSocket connection failed"**
   - Check your internet connection
   - Verify PumpPortal API key

3. **"Invalid private key"**
   - Ensure private key is Base58 encoded
   - Check for typos in the key

4. **"Transaction failed"**
   - Check Solana network status
   - Increase slippage tolerance
   - Verify RPC endpoint

### Logs

The bot provides detailed logging. Set `LOG_LEVEL=debug` for more verbose output.

## Development

### Project Structure
```
├── src/
│   ├── pumpportal.js      # PumpPortal API integration
│   ├── solana-trader.js   # Solana trading functions
│   ├── sniper-bot.js      # Main bot logic
│   ├── logger.js          # Logging utilities
│   └── error-handler.js   # Error handling
├── config.js              # Configuration management
├── index.js               # Main entry point
└── package.json           # Dependencies
```

### Adding Features

1. **Custom filters**: Modify `validateToken()` in `sniper-bot.js`
2. **Different strategies**: Extend the sell logic in `executeSell()`
3. **Additional APIs**: Add new integrations in the `src/` directory

## License

MIT License - see LICENSE file for details.

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review the logs for error details
3. Open an issue on GitHub

## Disclaimer

This software is provided "as is" without warranty. The authors are not responsible for any financial losses. Use at your own risk.
