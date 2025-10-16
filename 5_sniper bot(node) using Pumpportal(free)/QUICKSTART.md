# Quick Start Guide

## 🚀 Get Running in 5 Minutes

### 1. Install Dependencies
```bash
npm install
```

### 2. Run Setup
```bash
npm run setup
```

### 3. Configure Your Bot
Edit the `.env` file with your details:

```env
# Required - Get from your Solana wallet
PRIVATE_KEY=your_base58_private_key_here

# Required - Get from PumpPortal (free)
PUMPPORTAL_API_KEY=your_pumpportal_api_key_here

# Optional - Adjust these as needed
MAX_SOL_AMOUNT=0.01        # How much SOL to spend per trade
SELL_DELAY_MS=3000         # Wait 3 seconds before selling
SLIPPAGE_TOLERANCE=0.1     # 10% slippage tolerance
```

### 4. Start Trading
```bash
npm start
```

## 🔑 Getting Your Keys

### Solana Private Key
1. Open your Solana wallet (Phantom, Solflare, etc.)
2. Go to Settings → Export Private Key
3. Copy the Base58 encoded key

### PumpPortal API Key
1. Visit [PumpPortal](https://pumpportal.fun)
2. Sign up for a free account
3. Get your API key from the dashboard

## ⚠️ Safety First

- **Start Small**: Use 0.001-0.01 SOL for testing
- **Test First**: Run on devnet before mainnet
- **Monitor**: Watch the bot's performance
- **Stop Loss**: Be ready to stop if needed

## 🛠️ Troubleshooting

**"Insufficient funds"**
- Add more SOL to your wallet
- Reduce MAX_SOL_AMOUNT

**"WebSocket connection failed"**
- Check your internet connection
- Verify PumpPortal API key

**"Invalid private key"**
- Ensure it's Base58 encoded
- Check for typos

## 📊 What to Expect

The bot will:
1. Connect to PumpPortal
2. Listen for new token mints
3. Automatically buy new tokens
4. Wait 3 seconds (configurable)
5. Automatically sell the tokens
6. Show statistics every 30 seconds

## 🆘 Need Help?

- Check the full README.md
- Review the logs for errors
- Start with small amounts
- Test thoroughly before scaling up

**Remember: Only trade what you can afford to lose!**
