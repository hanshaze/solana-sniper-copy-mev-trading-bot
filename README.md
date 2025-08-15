# 🚀 Solana Copy Sniper & MEV Trading Bot (Node.js & Rust) — PumpFun, Bonk, Raydium

> **Ultra-fast, customizable Solana copy trading, sniper, and MEV bot with advanced sell logic for maximum profit.**

---

### **If you find this project helpful, please star 🌟 the repo!**  
#### Your feedback and stars motivate further development. Thank you!
> **Note:** Other folders and features are currently in development and will be deployed ASAP. Stay tuned for updates!

[![Solana](https://img.shields.io/badge/Solana-3.5.0-blue.svg)](https://solana.com/)
[![Node.js](https://img.shields.io/badge/Node.js-18+-green.svg)](https://nodejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://rust-lang.org/)
[![Telegram](https://img.shields.io/badge/Telegram-Bot-blue.svg)](https://telegram.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## 🌟 Overview

**Ready to level up your Solana trading?**  
This repo provides a battle-tested, ultra-fast Solana trading bot you can fully customize for your own strategies.  
Automate your trades, add your own logic, and maximize profits with professional-grade speed and reliability.

**Need more?**  
Hire me for custom development:  
- Integrate advanced features, optimize sell logic, or build a tailored trading solution.
- Get ultra-fast swap code, 0block confirm, off-chain signing, and expert support.

---

### 🛠️ Two Powerful Implementations

#### 📁 Node.js (`copy sniper bot(node) using gRPC/`)
- Real-time gRPC transaction monitoring
- Telegram bot integration for remote control & alerts
- Multi-platform: PumpSwap, PumpFun, Raydium LaunchLab, bonk.fun
- Copy trading & sniper functionality with customizable strategies

#### 🦀 Rust (`sniper (Rust) using jito Shred stream/`)
- Ultra-high performance with JITO shred stream
- Advanced MEV strategies for maximal extractable value
- Real-time transaction parsing & analytics
- PostgreSQL integration for trade logging

**Both versions offer:**
- Copy Trading: Replicate successful trades from other wallets
- Sniper Trading: Lightning-fast trades on new launches
- MEV Bot: Capture maximal extractable value on Solana
- Customizable Sell Logic: Implement your own profit-taking strategies
- Multi-Platform Support: PumpSwap, PumpFun, Raydium LaunchLab, bonk, and more

---

## 📲 Telegram: [@hanshaze007](https://t.me/hanshaze007)

---

## 🎯 Key Features

### ⚡ Ultra-Fast Execution
- Off-chain signing for max speed
- Multiple swap methods: solana, jito, nozomi, 0slot, race
- Priority fee optimization
- Retry logic with exponential backoff

### 🎛️ Flexible Trading Strategies
- Copy Trading: Follow top traders automatically
- Sniper Trading: Target new launches with precision
- Custom Sell Logic: Your own profit-taking rules
- Multi-token support

### 📱 Remote Control & Monitoring
- Telegram Bot: Control from anywhere (Node.js)
- Real-time alerts: Buy/sell notifications with PnL
- Balance monitoring & status dashboard

### 🔧 Advanced Features
- Transaction parsing & analytics
- Multi-platform support
- Robust error handling & alerting
- Configurable strategy parameters

---

## 🚀 Quick Start

### Prerequisites
- Node.js 18+ (for Node.js version)
- Rust 1.70+ (for Rust version)
- Solana wallet with SOL
- Telegram bot token (optional, Node.js)

---

### Node.js Setup

1. **Go to Node.js directory**
    ```bash
    cd "copy sniper bot(node) using gRPC"
    ```
2. **Install dependencies**
    ```bash
    npm install
    ```
3. **Configure environment**
    ```bash
    cp .env.example .env
    ```
    Edit `.env`:
    ```env
    RPC_URL=https://api.mainnet-beta.solana.com
    PRIVATE_KEY=your_private_key_here
    WALLET=your_wallet_address
    SWAP_METHOD=solana
    SLIPPAGE_BPS=50
    MAX_RETRIES=3
    RETRY_DELAY=1000
    TELEGRAM_BOT_TOKEN=your_bot_token
    TELEGRAM_CHAT_ID=your_chat_id
    LIMIT_BALANCE=0.1
    ```
4. **Start the bot**
    ```bash
    npm start
    ```

#### Node.js Features
- gRPC transaction monitoring
- Telegram integration
- Multi-platform support
- Copy & sniper trading
- Custom alerts with PnL

---

### Rust Setup

1. **Go to Rust directory**
    ```bash
    cd "sniper (Rust) using jito Shred stream"
    ```
2. **Build**
    ```bash
    cargo build --release
    ```
3. **Set up PostgreSQL** (for trade logging)
    ```bash
    # Install PostgreSQL and create database
    # Update DB_URL in main.rs if needed
    ```
4. **Configure environment**
    ```env
    RPC_URL=https://api.mainnet-beta.solana.com
    PRIVATE_KEY=your_private_key_here
    WALLET=your_wallet_address
    ```
5. **Start the bot**
    ```bash
    cargo run --release
    ```

#### Rust Features
- JITO shred stream processing
- Advanced MEV strategies
- Real-time transaction parsing
- PostgreSQL analytics
- Multi-threaded for HFT
- Robust error handling

---

## 📊 Feature Comparison

| Feature                | Node.js      | Rust         |
|------------------------|--------------|--------------|
| Performance            | Fast         | Ultra-fast   |
| Memory Usage           | Higher       | Lower        |
| Setup Complexity       | Easy         | Moderate     |
| Telegram Integration   | ✅           | ❌           |
| MEV Strategies         | Basic        | Advanced     |
| Database Integration   | ❌           | ✅ PostgreSQL |
| Transaction Parsing    | Basic        | Advanced     |
| Multi-threading        | Limited      | Full         |

---

## 🛠️ Configuration

### Swap Methods (Node.js)

| Method   | Speed      | Cost   | Use Case             |
|----------|------------|--------|----------------------|
| solana   | Fast       | Low    | General trading      |
| jito     | Very Fast  | Medium | High-priority trades |
| nozomi   | Ultra Fast | High   | Sniper trading       |
| 0slot    | Fast       | Low    | Copy trading         |
| race     | Very Fast  | Medium | Competitive trading  |

### Alert Settings (Node.js)
Customize your Telegram notifications:

```javascript
const alertSettings = {
  buyAlerts: true,           // Buy notifications
  sellAlerts: true,          // Sell notifications
  insufficientFundsAlerts: true,  // Low balance warnings
  balanceAlerts: true,       // Balance updates
  errorAlerts: true          // Error notifications
};
```

## 📈 Trading Strategies

### Copy Trading Strategy
1. **Identify Successful Wallets**: Find wallets with consistent profits
2. **Set Copy Parameters**: Define amount scaling and timing
3. **Monitor Transactions**: Track target wallet activity
4. **Execute Copies**: Automatically replicate trades

### Sniper Trading Strategy
1. **Token Launch Detection**: Monitor new token launches
2. **Entry Timing**: Execute trades at optimal moments
3. **Position Sizing**: Calculate appropriate position sizes
4. **Exit Strategy**: Implement custom sell logic

### Custom Sell Logic Examples

#### Conservative Strategy
```javascript
{
  stopLoss: 0.05,        // 5% stop loss
  takeProfit: 0.15,      // 15% take profit
  maxHoldTime: 1800      // 30 minutes max hold
}
```

#### Aggressive Strategy
```javascript
{
  stopLoss: 0.10,        // 10% stop loss
  takeProfit: 0.50,      // 50% take profit
  trailingStop: 0.20,    // 20% trailing stop
  maxHoldTime: 3600      // 1 hour max hold
}
```

## 🔧 Advanced Configuration

### Multi-Platform Support
Both bots support multiple Solana trading platforms:

- **PumpSwap**: High-speed DEX trading
- **PumpFun**: Meme token trading
- **Raydium LaunchLab /bonk.fun**: Launchpad trading
- **Raydium**: General DEX trading

### Transaction Parsing
Advanced transaction analysis for detailed trade insights:

```javascript
const parsedData = {
  solChanges: 0.1,           // SOL amount traded
  tokenChanges: 1000000,     // Token amount traded
  isBuy: true,               // Buy or sell transaction
  user: "wallet_address",    // Trader wallet
  mint: "token_mint",        // Token mint address
  pool: "pool_address",      // Trading pool
  liquidity: 1000,           // Pool liquidity
  coinCreator: "creator"     // Token creator
};
```

## 📱 Telegram Bot Setup (Node.js Only)

### Quick Setup
1. Create a Telegram bot with [@BotFather](https://t.me/botfather)
2. Add your bot token to `.env`
3. Start the bot and send `/start`

### Available Commands
- `/start` - Main control panel
- `/status` - Bot status and balance
- `/balance` - Check wallet balance
- `/alerts` - Manage notifications
- `/stats` - Trading statistics
- `/help` - Show all commands

### Interactive Features
- **One-click Start/Stop**: Control bot with buttons
- **Real-time Balance**: Live wallet balance updates
- **Alert Management**: Toggle notification types
- **Status Monitoring**: Live trading statistics

## 🚨 Safety Features

### Balance Protection
- **Minimum Balance Check**: Prevents trading with insufficient funds
- **Automatic Stop**: Stops bot when balance is too low
- **Balance Alerts**: Notifications for balance changes

### Error Handling
- **Retry Logic**: Automatic retry on failed transactions
- **Error Alerts**: Immediate notification of issues
- **Graceful Degradation**: Continues operation despite errors

### Security
- **Private Key Protection**: Secure key management
- **Environment Variables**: Sensitive data protection
- **Transaction Validation**: Verify all transactions before execution

## 📊 Performance Monitoring

### Real-time Metrics
- **Trade Success Rate**: Percentage of successful trades
- **Average PnL**: Mean profit/loss per trade
- **Total Volume**: Total trading volume
- **Active Positions**: Current open positions

### Alert System (Node.js)
- **Buy Alerts**: New position notifications
- **Sell Alerts**: Exit notifications with PnL
- **Error Alerts**: Issue notifications
- **Balance Alerts**: Fund level warnings

## 🔄 Updates & Maintenance

### Regular Updates
- **Performance Optimizations**: Faster execution
- **New Features**: Additional trading strategies
- **Bug Fixes**: Improved stability
- **Security Updates**: Enhanced protection

### Support
- **Documentation**: Comprehensive guides
- **Community**: Active user community
- **Custom Development**: Tailored solutions
- **Ultra-fast Swap Code**: Off-chain signing implementation

## 💡 Pro Tips

### Choosing Between Implementations
- **Use Node.js version** if you want:
  - Easy setup and configuration
  - Telegram integration for remote control
  - Quick deployment and testing
  - Basic copy trading and sniper functionality

- **Use Rust version** if you want:
  - Maximum performance and speed
  - Advanced MEV strategies
  - Sophisticated transaction analysis
  - Professional-grade trading infrastructure

### Maximizing Profits
1. **Start Small**: Begin with small amounts to test strategies
2. **Monitor Performance**: Track your bot's performance regularly
3. **Adjust Parameters**: Fine-tune settings based on results
4. **Diversify Strategies**: Use multiple trading approaches

### Risk Management
1. **Set Stop Losses**: Always use stop losses to limit losses
2. **Position Sizing**: Don't risk more than you can afford to lose
3. **Monitor Markets**: Stay informed about market conditions
4. **Regular Reviews**: Periodically review and adjust strategies

## 🤝 Support & Contact

### Get Professional Support
For ultra-fast swap code using off-chain signing and custom sell logic optimization:

- **Custom Development**: Tailored solutions for your needs
- **Sell Logic Optimization**: Maximize your profit potential
- **Ultra-fast Implementation**: Off-chain signing for speed
- **24/7 Support**: Round-the-clock assistance

### Contact Information
- **Telegram**: [@hanshaze007](https://t.me/hanshaze007)


## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

**Trading cryptocurrencies involves substantial risk of loss and is not suitable for every investor. The valuation of cryptocurrencies may fluctuate, and, as a result, you may lose more than your original investment. You should not invest money that you cannot afford to lose.**

This bot is NOT for educational and research purposes. Use at your own risk. The developers are not responsible for any financial losses incurred through the use of this software without contact to me.

---

**🚀 Start your journey to profitable trading today!**

*Built with ❤️ for the Solana community and Traders*
