import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import { RateLimiterMemory } from 'rate-limiter-flexible';
import { config } from '../config.js';
import logger from '../utils/logger.js';
import riskManager from '../services/riskManager.js';
import notificationService from '../services/notifications.js';

const app = express();
const PORT = process.env.DASHBOARD_PORT || 3000;

// Rate limiting
const rateLimiter = new RateLimiterMemory({
  keyGenerator: (req) => req.ip,
  points: config.security.maxRequestsPerMinute,
  duration: 60,
});

// Middleware
app.use(helmet());
app.use(cors());
app.use(compression());
app.use(express.json());
app.use(express.static('dashboard/public'));

// Rate limiting middleware
app.use(async (req, res, next) => {
  try {
    await rateLimiter.consume(req.ip);
    next();
  } catch (rejRes) {
    res.status(429).json({
      error: 'Too many requests',
      retryAfter: Math.round(rejRes.msBeforeNext / 1000),
    });
  }
});

// Health check endpoint
app.get('/api/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
  });
});

// Get bot status and configuration
app.get('/api/status', (req, res) => {
  try {
    const status = {
      bot: {
        status: 'running',
        uptime: process.uptime(),
        version: '2.0.0',
        timestamp: new Date().toISOString(),
      },
      config: {
        trading: config.trading,
        risk: config.risk,
        pools: config.pools,
      },
    };
    
    res.json(status);
  } catch (error) {
    logger.error('Error getting bot status', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Get risk metrics
app.get('/api/risk', (req, res) => {
  try {
    const riskMetrics = riskManager.getRiskMetrics();
    res.json(riskMetrics);
  } catch (error) {
    logger.error('Error getting risk metrics', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Get active positions
app.get('/api/positions', (req, res) => {
  try {
    const positions = riskManager.getActivePositions();
    const summary = riskManager.getPositionSummary();
    
    res.json({
      positions,
      summary,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error getting positions', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Get trading history
app.get('/api/history', (req, res) => {
  try {
    const history = riskManager.tradeHistory || [];
    const dailyStats = riskManager.getDailyStats();
    
    res.json({
      history,
      dailyStats,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error getting trading history', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Get daily statistics
app.get('/api/stats', (req, res) => {
  try {
    const dailyStats = riskManager.getDailyStats();
    res.json(dailyStats);
  } catch (error) {
    logger.error('Error getting daily stats', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Emergency actions
app.post('/api/emergency/close-all', async (req, res) => {
  try {
    const { reason } = req.body;
    const results = await riskManager.emergencyCloseAll(reason || 'dashboard_request');
    
    res.json({
      success: true,
      message: 'Emergency closure initiated',
      results,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error initiating emergency closure', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Send test notification
app.post('/api/notifications/test', async (req, res) => {
  try {
    const { message, type } = req.body;
    await notificationService.sendNotification(
      message || 'Test notification from dashboard',
      type || 'info',
      { source: 'dashboard' }
    );
    
    res.json({
      success: true,
      message: 'Test notification sent',
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error sending test notification', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Update configuration (read-only for now, could be extended)
app.get('/api/config', (req, res) => {
  try {
    // Return safe configuration (no sensitive data)
    const safeConfig = {
      trading: config.trading,
      risk: config.risk,
      pools: config.pools,
      swap: config.swap,
      logging: config.logging,
    };
    
    res.json(safeConfig);
  } catch (error) {
    logger.error('Error getting configuration', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// WebSocket endpoint for real-time updates (placeholder)
app.get('/api/ws', (req, res) => {
  res.json({
    message: 'WebSocket endpoint - implement with Socket.IO for real-time updates',
    timestamp: new Date().toISOString(),
  });
});

// Error handling middleware
app.use((error, req, res, next) => {
  logger.error('Dashboard error', error);
  res.status(500).json({
    error: 'Internal server error',
    message: error.message,
    timestamp: new Date().toISOString(),
  });
});

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Endpoint not found',
    timestamp: new Date().toISOString(),
  });
});

// Start server
const server = app.listen(PORT, () => {
  logger.info(`Dashboard server started on port ${PORT}`);
  logger.info(`Dashboard available at: http://localhost:${PORT}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down dashboard server');
  server.close(() => {
    logger.info('Dashboard server closed');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down dashboard server');
  server.close(() => {
    logger.info('Dashboard server closed');
    process.exit(0);
  });
});

export default app;
