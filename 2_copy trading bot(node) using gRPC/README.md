# 🚀 Solana Trading Bot v2.0

A high-performance, enterprise-grade Solana trading bot with advanced MEV protection, comprehensive risk management, and real-time monitoring capabilities.

## ✨ New Features in v2.0

- **🔒 Advanced Risk Management**: Daily loss limits, position limits, and automated risk scoring
- **📊 Real-time Dashboard**: Web-based monitoring interface with live charts and metrics
- **🔔 Multi-channel Notifications**: Telegram, Discord, and email notifications
- **📝 Structured Logging**: Winston-based logging with daily rotation and multiple levels
- **⚡ Performance Optimization**: Connection pooling, rate limiting, and retry mechanisms
- **🛡️ Enhanced Security**: IP whitelisting, rate limiting, and secure configuration management
- **📈 Advanced Analytics**: PnL tracking, win rate analysis, and risk metrics
- **🔄 Graceful Shutdown**: Safe position closure and cleanup on exit

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   gRPC Stream  │───▶│ Transaction     │───▶│ Trading Engine  │
│   (Triton One) │    │   Parser        │    │   (Main Bot)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Pool Detection  │    │ Risk Manager    │
                       │ (PumpFun, etc.) │    │ & Position Mgmt │
                       └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Jupiter API     │    │ Notification    │
                       │ Swap Execution  │    │ Service         │
                       └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Web Dashboard   │    │ Logger &        │
                       │ (Real-time UI)  │    │ Analytics       │
                       └─────────────────┘    └─────────────────┘
```

## 🚀 Quick Start

### 1. Prerequisites

- **Node.js 18+** (Latest LTS recommended)
- **Solana wallet** with SOL balance
- **Triton One gRPC** access
- **RPC endpoint** (Helius, QuickNode, etc.)

### 2. Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd solana-trading-bot

# Install dependencies
npm install

# Copy environment template
cp env.template .env
```

### 3. Configuration

Edit the `.env` file with your configuration:

```bash
# Essential Configuration
PRIVATE_KEY=your_wallet_private_key
RPC_URL=https://your-rpc-endpoint.com
GRPC_ENDPOINT=https://your-grpc-endpoint.com
GRPCTOKEN=your_grpc_token

# Trading Parameters
SNIPERAMOUNT=0.1          # SOL amount per snipe
PROFIT_TARGET=2.0         # 2x profit target
STOP_LOSS=0.5             # 50% stop loss
MAX_HOLD_TIME=300000      # 5 minutes max hold
MIN_LIQUIDITY=10          # Minimum liquidity in SOL
MAX_POSITIONS=5           # Maximum concurrent positions

# Risk Management
MAX_DAILY_LOSS=1.0        # Maximum daily loss in SOL
MAX_SINGLE_LOSS=0.5       # Maximum single trade loss
TRADE_COOLDOWN=5000       # Cooldown between trades (ms)

# Dashboard (Optional)
ENABLE_DASHBOARD=true      # Enable web dashboard
DASHBOARD_PORT=3000        # Dashboard port

# Notifications (Optional)
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
DISCORD_WEBHOOK_URL=your_webhook_url
```

### 4. Run the Bot

```bash
# Start the trading bot
npm start

# Or for development with auto-restart
npm run dev

# Start with dashboard enabled
ENABLE_DASHBOARD=true npm start
```

## 🌐 Web Dashboard

Access the real-time dashboard at `http://localhost:3000` (when enabled):

- **📊 Live Metrics**: Real-time PnL, positions, and risk levels
- **📈 Interactive Charts**: PnL trends and position distribution
- **🔍 Position Management**: View and manage active positions
- **📋 Trading History**: Complete trade history with analytics
- **⚙️ Configuration**: View and monitor bot settings
- **🚨 Emergency Controls**: Emergency position closure

## ⚙️ Configuration Options

### Trading Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `SNIPERAMOUNT` | 0.1 | SOL amount to use for each snipe |
| `PROFIT_TARGET` | 2.0 | Profit target multiplier (2x) |
| `STOP_LOSS` | 0.5 | Stop loss multiplier (50% loss) |
| `MAX_HOLD_TIME` | 300000 | Maximum time to hold position (5 min) |
| `MIN_LIQUIDITY` | 10 | Minimum liquidity required in SOL |
| `MAX_POSITIONS` | 5 | Maximum concurrent positions |

### Risk Management

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MAX_DAILY_LOSS` | 1.0 | Maximum daily loss in SOL |
| `MAX_SINGLE_LOSS` | 0.5 | Maximum single trade loss in SOL |
| `TRADE_COOLDOWN` | 5000 | Cooldown between trades (ms) |

### Pool Filters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `ENABLE_PUMPFUN` | true | Enable PumpFun pool monitoring |
| `ENABLE_PUMPSWAP` | true | Enable PumpSwap pool monitoring |
| `ENABLE_RAYDIUM_LAUNCHLAB` | true | Enable Raydium LaunchLab monitoring |
| `ENABLE_RAYDIUM_CPMM` | true | Enable Raydium CPMM monitoring |

## 📊 Risk Management Features

### Automated Risk Controls

- **Daily Loss Limits**: Automatic trading halt when daily loss threshold reached
- **Position Limits**: Maximum concurrent position management
- **Trade Cooldowns**: Prevents rapid-fire trading
- **Risk Scoring**: Real-time risk level assessment (LOW/MEDIUM/HIGH)

### Position Management

- **Profit Targets**: Automatic exit at configured profit levels
- **Stop Losses**: Automatic exit at configured loss levels
- **Time-based Exits**: Maximum hold time enforcement
- **Emergency Closure**: Immediate closure of all positions

### Risk Metrics

- **Win Rate Analysis**: Track profitable vs. losing trades
- **PnL Tracking**: Real-time profit/loss monitoring
- **Risk Recommendations**: AI-powered trading suggestions
- **Performance Analytics**: Detailed trading performance metrics

## 🔔 Notification System

### Multi-channel Alerts

- **Telegram**: Real-time trading alerts and updates
- **Discord**: Webhook-based notifications with rich embeds
- **Email**: Detailed reports for important events
- **Dashboard**: Real-time web interface updates

### Notification Types

- **Trade Execution**: Buy/sell confirmations
- **Profit Targets**: When profit targets are reached
- **Stop Losses**: When stop losses are triggered
- **Risk Alerts**: High-risk situation notifications
- **Bot Status**: Startup, shutdown, and error notifications

## 📝 Logging & Monitoring

### Structured Logging

- **Multiple Levels**: Debug, Info, Warn, Error, Trade, Profit, Loss
- **Daily Rotation**: Automatic log file rotation and compression
- **Performance Tracking**: Operation timing and performance metrics
- **Error Context**: Detailed error information with stack traces

### Monitoring Features

- **Health Checks**: API endpoint health monitoring
- **Performance Metrics**: Response times and throughput
- **Error Tracking**: Comprehensive error logging and alerting
- **Audit Trail**: Complete trading activity audit log

## 🛡️ Security Features

### Access Control

- **Rate Limiting**: API request rate limiting
- **IP Whitelisting**: Configurable IP address restrictions
- **Authentication**: Secure API key management
- **Input Validation**: Comprehensive input sanitization

### Data Protection

- **Secure Configuration**: Environment variable management
- **Sensitive Data Masking**: Private key and API key protection
- **Audit Logging**: Complete access and action logging
- **Error Handling**: Secure error message handling

## 🔧 Advanced Configuration

### Performance Optimization

```bash
# Connection settings
CONNECTION_TIMEOUT=30000
MAX_RETRIES=3
RETRY_DELAY=1000

# Rate limiting
MAX_REQUESTS_PER_MINUTE=100
ENABLE_RATE_LIMITING=true
```

### Logging Configuration

```bash
# Log levels
LOG_LEVEL=info
DEBUG=false
LOG_TO_FILE=true
LOG_FILE_PATH=./logs/trading-bot.log
```

### MEV Protection

```bash
# MEV protection settings
ENABLE_MEV_PROTECTION=true
SWAP_METHOD=race  # race, nozomi, 0slot, solana
PRIORITY_FEE=1000
```

## 📈 Trading Strategies

### Conservative Strategy
```bash
SNIPERAMOUNT=0.05
PROFIT_TARGET=1.5
STOP_LOSS=0.7
MAX_HOLD_TIME=600000
MIN_LIQUIDITY=20
MAX_DAILY_LOSS=0.5
```

### Aggressive Strategy
```bash
SNIPERAMOUNT=0.2
PROFIT_TARGET=3.0
STOP_LOSS=0.3
MAX_HOLD_TIME=180000
MIN_LIQUIDITY=5
MAX_DAILY_LOSS=2.0
```

### Balanced Strategy
```bash
SNIPERAMOUNT=0.1
PROFIT_TARGET=2.0
STOP_LOSS=0.5
MAX_HOLD_TIME=300000
MIN_LIQUIDITY=10
MAX_DAILY_LOSS=1.0
```

## 🚨 Emergency Procedures

### Emergency Stop

```bash
# Send SIGINT to gracefully shutdown
Ctrl+C

# Or use the dashboard emergency button
# Click "Emergency Close All" in the web interface
```

### Manual Position Closure

```bash
# Close specific position via dashboard
# Or modify the code to add CLI commands
```

## 🔍 Troubleshooting

### Common Issues

1. **gRPC Connection Failed**
   - Verify `GRPC_ENDPOINT` and `GRPCTOKEN`
   - Check network connectivity
   - Ensure Triton One subscription is active

2. **Transaction Failures**
   - Verify wallet has sufficient SOL
   - Check RPC endpoint status
   - Adjust slippage tolerance

3. **Dashboard Not Loading**
   - Ensure `ENABLE_DASHBOARD=true`
   - Check port availability
   - Verify firewall settings

4. **Notification Failures**
   - Verify API keys and tokens
   - Check network connectivity
   - Review notification service logs

### Debug Mode

Enable debug logging:
```bash
DEBUG=true
LOG_LEVEL=debug
```

### Log Files

Check log files for detailed information:
```bash
# Main logs
tail -f logs/trading-bot-*.log

# Error logs
tail -f logs/trading-bot-error-*.log
```

## 📚 API Reference

### Dashboard API Endpoints

- `GET /api/health` - Health check
- `GET /api/status` - Bot status and configuration
- `GET /api/risk` - Risk metrics and analytics
- `GET /api/positions` - Active positions
- `GET /api/history` - Trading history
- `GET /api/stats` - Daily statistics
- `POST /api/emergency/close-all` - Emergency position closure

### WebSocket Support

Real-time updates via WebSocket (planned for v2.1):
```javascript
// Future implementation
const ws = new WebSocket('ws://localhost:3000/api/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Handle real-time updates
};
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install development dependencies
npm install

# Run linting
npm run lint

# Format code
npm run format

# Run tests
npm test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Triton One](https://triton.one/) for gRPC streaming
- [Jupiter](https://jup.ag/) for swap aggregation
- [Solana Labs](https://solana.com/) for the blockchain
- [Raydium](https://raydium.io/) for the DEX platform
- [Winston](https://github.com/winstonjs/winston) for logging
- [Express](https://expressjs.com/) for the web framework

## 📞 Support

For support and questions:
- Create an issue on GitHub
- Join our Discord community
- Check the documentation
- Review the troubleshooting guide

## 🔮 Roadmap

### v2.1 (Q2 2024)
- WebSocket real-time updates
- Advanced charting and analytics
- Mobile-responsive dashboard
- API rate limiting improvements

### v2.2 (Q3 2024)
- Machine learning price prediction
- Advanced order types
- Portfolio rebalancing
- Multi-wallet support

### v3.0 (Q4 2024)
- Cross-chain support
- Advanced MEV strategies
- Institutional features
- Cloud deployment options

---

**⚠️ Disclaimer: This software is for educational purposes only. Use at your own risk. The authors are not responsible for any financial losses. Cryptocurrency trading involves significant risk and may not be suitable for all investors.**