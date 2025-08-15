import { config } from '../config.js';
import logger from '../utils/logger.js';
import notificationService from './notifications.js';

class RiskManager {
  constructor() {
    this.dailyStats = {
      totalTrades: 0,
      profitableTrades: 0,
      losingTrades: 0,
      totalProfit: 0,
      totalLoss: 0,
      netPnL: 0,
      startTime: new Date(),
    };

    this.activePositions = new Map();
    this.tradeHistory = [];
    this.lastTradeTime = 0;
    
    // Reset daily stats at midnight
    this.scheduleDailyReset();
  }

  // Check if a new trade is allowed
  canExecuteTrade(amount, tokenMint) {
    const now = Date.now();
    const errors = [];

    // Check daily loss limit
    if (this.dailyStats.netPnL <= -config.risk.maxDailyLoss) {
      errors.push(`Daily loss limit reached: ${this.dailyStats.netPnL.toFixed(4)} SOL`);
    }

    // Check single trade loss limit
    if (amount > config.risk.maxSingleLoss) {
      errors.push(`Trade amount ${amount} SOL exceeds single trade limit ${config.risk.maxSingleLoss} SOL`);
    }

    // Check position limit
    if (this.activePositions.size >= config.trading.maxPositions) {
      errors.push(`Maximum positions limit reached: ${this.activePositions.size}/${config.trading.maxPositions}`);
    }

    // Check cooldown period
    if (now - this.lastTradeTime < config.risk.tradeCooldown) {
      const remainingCooldown = config.risk.tradeCooldown - (now - this.lastTradeTime);
      errors.push(`Trade cooldown active: ${Math.ceil(remainingCooldown / 1000)}s remaining`);
    }

    // Check if token is already in active positions
    if (this.activePositions.has(tokenMint)) {
      errors.push(`Token ${tokenMint} already has an active position`);
    }

    if (errors.length > 0) {
      logger.warn('Trade blocked by risk manager', { errors, amount, tokenMint });
      return { allowed: false, errors };
    }

    return { allowed: true, errors: [] };
  }

  // Record a new trade
  recordTrade(tradeType, tokenMint, amount, price, txHash) {
    const now = Date.now();
    this.lastTradeTime = now;

    const trade = {
      id: `${tokenMint}-${now}`,
      type: tradeType,
      tokenMint,
      amount,
      price,
      txHash,
      timestamp: now,
      status: 'pending',
    };

    if (tradeType === 'buy') {
      this.activePositions.set(tokenMint, {
        entryPrice: price,
        entryAmount: amount,
        entryTime: now,
        entryValue: amount * price,
        currentPrice: price,
        tradeId: trade.id,
      });

      logger.info(`Position opened: ${tokenMint}`, {
        entryPrice: price,
        entryAmount: amount,
        entryValue: amount * price,
      });

      notificationService.notifyPositionUpdate('opened', tokenMint, {
        entryPrice: price,
        entryAmount: amount,
        entryValue: amount * price,
      });
    } else if (tradeType === 'sell') {
      const position = this.activePositions.get(tokenMint);
      if (position) {
        const exitValue = amount * price;
        const pnl = exitValue - position.entryValue;
        const pnlRatio = exitValue / position.entryValue;

        trade.pnl = pnl;
        trade.pnlRatio = pnlRatio;
        trade.entryPrice = position.entryPrice;
        trade.entryValue = position.entryValue;

        // Update daily stats
        this.updateDailyStats(pnl, pnlRatio > 1);

        // Remove from active positions
        this.activePositions.delete(tokenMint);

        logger.info(`Position closed: ${tokenMint}`, {
          pnl,
          pnlRatio: pnlRatio.toFixed(2),
          entryPrice: position.entryPrice,
          exitPrice: price,
        });

        // Send appropriate notification
        if (pnlRatio >= config.trading.profitTarget) {
          notificationService.notifyProfitTarget(tokenMint, pnlRatio, amount);
        } else if (pnlRatio <= config.trading.stopLoss) {
          notificationService.notifyStopLoss(tokenMint, pnlRatio, amount);
        } else {
          notificationService.notifyPositionUpdate('closed', tokenMint, {
            pnl,
            pnlRatio: pnlRatio.toFixed(2),
            reason: 'manual',
          });
        }
      }
    }

    this.tradeHistory.push(trade);
    logger.debug('Trade recorded', trade);

    return trade;
  }

  // Update position price
  updatePositionPrice(tokenMint, newPrice) {
    const position = this.activePositions.get(tokenMint);
    if (position) {
      position.currentPrice = newPrice;
      position.currentValue = position.entryAmount * newPrice;
      position.pnl = position.currentValue - position.entryValue;
      position.pnlRatio = position.currentValue / position.entryValue;
    }
  }

  // Check if position should be closed based on risk parameters
  shouldClosePosition(tokenMint) {
    const position = this.activePositions.get(tokenMint);
    if (!position) return { shouldClose: false, reason: null };

    const now = Date.now();
    const holdTime = now - position.entryTime;

    // Check profit target
    if (position.pnlRatio >= config.trading.profitTarget) {
      return { shouldClose: true, reason: 'profit_target', pnlRatio: position.pnlRatio };
    }

    // Check stop loss
    if (position.pnlRatio <= config.trading.stopLoss) {
      return { shouldClose: true, reason: 'stop_loss', pnlRatio: position.pnlRatio };
    }

    // Check max hold time
    if (holdTime >= config.trading.maxHoldTime) {
      return { shouldClose: true, reason: 'max_hold_time', holdTime };
    }

    return { shouldClose: false, reason: null };
  }

  // Get position information
  getPosition(tokenMint) {
    return this.activePositions.get(tokenMint);
  }

  // Get all active positions
  getActivePositions() {
    return Array.from(this.activePositions.entries()).map(([mint, position]) => ({
      mint,
      ...position,
      holdTime: Date.now() - position.entryTime,
    }));
  }

  // Get position summary
  getPositionSummary() {
    const positions = this.getActivePositions();
    const totalValue = positions.reduce((sum, pos) => sum + pos.currentValue, 0);
    const totalPnL = positions.reduce((sum, pos) => sum + pos.pnl, 0);

    return {
      activePositions: positions.length,
      totalValue,
      totalPnL,
      averagePnL: positions.length > 0 ? totalPnL / positions.length : 0,
    };
  }

  // Get daily statistics
  getDailyStats() {
    const now = new Date();
    const startOfDay = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const uptime = now - this.dailyStats.startTime;

    return {
      ...this.dailyStats,
      uptime,
      winRate: this.dailyStats.totalTrades > 0 
        ? (this.dailyStats.profitableTrades / this.dailyStats.totalTrades * 100).toFixed(2)
        : 0,
      averageProfit: this.dailyStats.profitableTrades > 0 
        ? this.dailyStats.totalProfit / this.dailyStats.profitableTrades
        : 0,
      averageLoss: this.dailyStats.losingTrades > 0 
        ? this.dailyStats.totalLoss / this.dailyStats.losingTrades
        : 0,
    };
  }

  // Update daily statistics
  updateDailyStats(pnl, isProfitable) {
    this.dailyStats.totalTrades++;
    
    if (isProfitable) {
      this.dailyStats.profitableTrades++;
      this.dailyStats.totalProfit += pnl;
    } else {
      this.dailyStats.losingTrades++;
      this.dailyStats.totalLoss += Math.abs(pnl);
    }

    this.dailyStats.netPnL += pnl;

    // Log daily stats update
    logger.debug('Daily stats updated', {
      totalTrades: this.dailyStats.totalTrades,
      netPnL: this.dailyStats.netPnL.toFixed(4),
    });

    // Check if daily loss limit is approaching
    if (this.dailyStats.netPnL <= -config.risk.maxDailyLoss * 0.8) {
      notificationService.sendNotification(
        `⚠️ Daily loss limit approaching: ${this.dailyStats.netPnL.toFixed(4)} SOL`,
        'warning',
        { dailyStats: this.dailyStats }
      );
    }
  }

  // Reset daily statistics
  resetDailyStats() {
    const previousStats = { ...this.dailyStats };
    
    this.dailyStats = {
      totalTrades: 0,
      profitableTrades: 0,
      losingTrades: 0,
      totalProfit: 0,
      totalLoss: 0,
      netPnL: 0,
      startTime: new Date(),
    };

    logger.info('Daily stats reset', { previousStats });
    notificationService.sendNotification(
      `📊 Daily trading session ended. Net PnL: ${previousStats.netPnL.toFixed(4)} SOL`,
      'info',
      { previousStats }
    );
  }

  // Schedule daily reset at midnight
  scheduleDailyReset() {
    const now = new Date();
    const tomorrow = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
    const timeUntilMidnight = tomorrow - now;

    setTimeout(() => {
      this.resetDailyStats();
      // Schedule next reset
      this.scheduleDailyReset();
    }, timeUntilMidnight);
  }

  // Emergency close all positions
  async emergencyCloseAll(reason = 'emergency') {
    logger.warn('Emergency closing all positions', { reason });
    
    const positions = Array.from(this.activePositions.keys());
    const results = [];

    for (const mint of positions) {
      try {
        const position = this.activePositions.get(mint);
        if (position) {
          // Mark position for emergency closure
          position.emergencyClose = true;
          position.emergencyReason = reason;
          
          results.push({
            mint,
            status: 'marked_for_closure',
            reason,
          });
        }
      } catch (error) {
        logger.error('Error marking position for emergency closure', { mint, error });
        results.push({
          mint,
          status: 'error',
          error: error.message,
        });
      }
    }

    notificationService.sendNotification(
      `🚨 Emergency closure initiated for ${positions.length} positions`,
      'warning',
      { reason, results }
    );

    return results;
  }

  // Get risk metrics
  getRiskMetrics() {
    const dailyStats = this.getDailyStats();
    const positionSummary = this.getPositionSummary();
    
    return {
      dailyStats,
      positionSummary,
      riskLevel: this.calculateRiskLevel(),
      recommendations: this.getRiskRecommendations(),
    };
  }

  // Calculate overall risk level
  calculateRiskLevel() {
    const dailyStats = this.getDailyStats();
    const positionSummary = this.getPositionSummary();
    
    let riskScore = 0;
    
    // Daily loss proximity
    if (dailyStats.netPnL <= -config.risk.maxDailyLoss * 0.9) {
      riskScore += 30;
    } else if (dailyStats.netPnL <= -config.risk.maxDailyLoss * 0.7) {
      riskScore += 20;
    } else if (dailyStats.netPnL <= -config.risk.maxDailyLoss * 0.5) {
      riskScore += 10;
    }

    // Position concentration
    if (positionSummary.activePositions >= config.trading.maxPositions * 0.8) {
      riskScore += 20;
    }

    // Win rate
    if (dailyStats.winRate < 30) {
      riskScore += 25;
    } else if (dailyStats.winRate < 50) {
      riskScore += 15;
    }

    if (riskScore >= 60) return 'HIGH';
    if (riskScore >= 30) return 'MEDIUM';
    return 'LOW';
  }

  // Get risk recommendations
  getRiskRecommendations() {
    const recommendations = [];
    const dailyStats = this.getDailyStats();
    const positionSummary = this.getPositionSummary();

    if (dailyStats.netPnL <= -config.risk.maxDailyLoss * 0.8) {
      recommendations.push('Consider reducing position sizes or stopping trading for the day');
    }

    if (positionSummary.activePositions >= config.trading.maxPositions * 0.8) {
      recommendations.push('Approaching maximum position limit - consider closing some positions');
    }

    if (dailyStats.winRate < 40) {
      recommendations.push('Low win rate - review trading strategy and risk parameters');
    }

    if (positionSummary.totalPnL < 0) {
      recommendations.push('Overall portfolio in loss - consider implementing stricter stop losses');
    }

    return recommendations;
  }
}

// Create singleton instance
const riskManager = new RiskManager();

export default riskManager;
