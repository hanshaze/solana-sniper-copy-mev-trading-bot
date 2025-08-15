import TelegramBot from 'node-telegram-bot-api';
import nodemailer from 'nodemailer';
import axios from 'axios';
import { config } from '../config.js';
import logger from '../utils/logger.js';

class NotificationService {
  constructor() {
    this.telegramBot = null;
    this.emailTransporter = null;
    this.initializeServices();
  }

  initializeServices() {
    // Initialize Telegram bot
    if (config.notifications.telegram.botToken && config.notifications.telegram.chatId) {
      try {
        this.telegramBot = new TelegramBot(config.notifications.telegram.botToken, { polling: false });
        logger.info('Telegram bot initialized successfully');
      } catch (error) {
        logger.error('Failed to initialize Telegram bot', error);
      }
    }

    // Initialize email transporter
    if (config.notifications.email.host && config.notifications.email.user && config.notifications.email.pass) {
      try {
        this.emailTransporter = nodemailer.createTransporter({
          host: config.notifications.email.host,
          port: config.notifications.email.port,
          secure: false,
          auth: {
            user: config.notifications.email.user,
            pass: config.notifications.email.pass,
          },
        });
        logger.info('Email service initialized successfully');
      } catch (error) {
        logger.error('Failed to initialize email service', error);
      }
    }
  }

  // Send notification to all configured services
  async sendNotification(message, type = 'info', data = {}) {
    const promises = [];

    // Send to Telegram
    if (this.telegramBot) {
      promises.push(this.sendTelegramMessage(message, type, data));
    }

    // Send to Discord
    if (config.notifications.discord.webhookUrl) {
      promises.push(this.sendDiscordMessage(message, type, data));
    }

    // Send to Email for important notifications
    if (this.emailTransporter && ['error', 'warning', 'trade', 'profit', 'loss'].includes(type)) {
      promises.push(this.sendEmail(message, type, data));
    }

    try {
      await Promise.allSettled(promises);
    } catch (error) {
      logger.error('Error sending notifications', error);
    }
  }

  // Telegram notifications
  async sendTelegramMessage(message, type = 'info', data = {}) {
    if (!this.telegramBot) return;

    try {
      const emoji = this.getEmojiForType(type);
      const formattedMessage = this.formatTelegramMessage(message, type, data);
      
      await this.telegramBot.sendMessage(config.notifications.telegram.chatId, formattedMessage, {
        parse_mode: 'HTML',
        disable_web_page_preview: true,
      });

      logger.debug('Telegram message sent successfully');
    } catch (error) {
      logger.error('Failed to send Telegram message', error);
    }
  }

  // Discord notifications
  async sendDiscordMessage(message, type = 'info', data = {}) {
    if (!config.notifications.discord.webhookUrl) return;

    try {
      const embed = this.createDiscordEmbed(message, type, data);
      
      await axios.post(config.notifications.discord.webhookUrl, {
        embeds: [embed],
      });

      logger.debug('Discord message sent successfully');
    } catch (error) {
      logger.error('Failed to send Discord message', error);
    }
  }

  // Email notifications
  async sendEmail(message, type = 'info', data = {}) {
    if (!this.emailTransporter) return;

    try {
      const subject = `Solana Trading Bot - ${type.toUpperCase()}`;
      const htmlContent = this.formatEmailMessage(message, type, data);

      await this.emailTransporter.sendMail({
        from: config.notifications.email.user,
        to: config.notifications.email.user, // Send to self
        subject,
        html: htmlContent,
      });

      logger.debug('Email sent successfully');
    } catch (error) {
      logger.error('Failed to send email', error);
    }
  }

  // Trade-specific notifications
  async notifyTradeExecution(tradeType, tokenMint, amount, price, txHash) {
    const message = `${tradeType.toUpperCase()} executed for ${tokenMint}`;
    const data = {
      tradeType,
      tokenMint,
      amount,
      price,
      txHash,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'trade', data);
  }

  async notifyProfitTarget(tokenMint, profitRatio, amount) {
    const message = `🎯 Profit target reached for ${tokenMint}: ${profitRatio.toFixed(2)}x`;
    const data = {
      tokenMint,
      profitRatio,
      amount,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'profit', data);
  }

  async notifyStopLoss(tokenMint, lossRatio, amount) {
    const message = `🛑 Stop loss triggered for ${tokenMint}: ${lossRatio.toFixed(2)}x`;
    const data = {
      tokenMint,
      lossRatio,
      amount,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'loss', data);
  }

  async notifyPositionUpdate(action, mint, details) {
    const message = `Position ${action}: ${mint}`;
    const data = {
      action,
      mint,
      details,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'info', data);
  }

  async notifyError(error, context = '') {
    const message = `Error in ${context}: ${error.message}`;
    const data = {
      error: error.stack,
      context,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'error', data);
  }

  async notifyBotStatus(status, details = {}) {
    const message = `Bot Status: ${status}`;
    const data = {
      status,
      details,
      timestamp: new Date().toISOString(),
    };

    await this.sendNotification(message, 'info', data);
  }

  // Helper methods
  getEmojiForType(type) {
    const emojis = {
      info: 'ℹ️',
      success: '✅',
      warning: '⚠️',
      error: '❌',
      trade: '💰',
      profit: '📈',
      loss: '📉',
    };
    return emojis[type] || '📝';
  }

  formatTelegramMessage(message, type, data) {
    let formattedMessage = `${this.getEmojiForType(type)} <b>${type.toUpperCase()}</b>\n\n`;
    formattedMessage += `${message}\n\n`;

    if (data.timestamp) {
      formattedMessage += `⏰ <i>${new Date(data.timestamp).toLocaleString()}</i>\n`;
    }

    if (data.txHash) {
      formattedMessage += `🔗 <a href="https://solscan.io/tx/${data.txHash}">View Transaction</a>\n`;
    }

    if (data.tokenMint) {
      formattedMessage += `🪙 <a href="https://solscan.io/token/${data.tokenMint}">View Token</a>\n`;
    }

    return formattedMessage;
  }

  createDiscordEmbed(message, type, data) {
    const colors = {
      info: 0x0099ff,
      success: 0x00ff00,
      warning: 0xffff00,
      error: 0xff0000,
      trade: 0x00ffff,
      profit: 0x00ff00,
      loss: 0xff0000,
    };

    const embed = {
      title: `${this.getEmojiForType(type)} ${type.toUpperCase()}`,
      description: message,
      color: colors[type] || 0x0099ff,
      timestamp: new Date().toISOString(),
      fields: [],
    };

    if (data.txHash) {
      embed.fields.push({
        name: 'Transaction',
        value: `[View on Solscan](https://solscan.io/tx/${data.txHash})`,
        inline: true,
      });
    }

    if (data.tokenMint) {
      embed.fields.push({
        name: 'Token',
        value: `[View on Solscan](https://solscan.io/token/${data.tokenMint})`,
        inline: true,
      });
    }

    if (data.profitRatio) {
      embed.fields.push({
        name: 'Profit Ratio',
        value: `${data.profitRatio.toFixed(2)}x`,
        inline: true,
      });
    }

    return embed;
  }

  formatEmailMessage(message, type, data) {
    let html = `
      <html>
        <head>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .header { background-color: #f0f0f0; padding: 15px; border-radius: 5px; }
            .content { margin: 20px 0; }
            .footer { color: #666; font-size: 12px; margin-top: 20px; }
            .highlight { background-color: #fff3cd; padding: 10px; border-radius: 3px; }
          </style>
        </head>
        <body>
          <div class="header">
            <h2>Solana Trading Bot Notification</h2>
            <p><strong>Type:</strong> ${type.toUpperCase()}</p>
            <p><strong>Time:</strong> ${new Date().toLocaleString()}</p>
          </div>
          
          <div class="content">
            <p>${message}</p>
    `;

    if (data.txHash) {
      html += `<p><strong>Transaction:</strong> <a href="https://solscan.io/tx/${data.txHash}">View on Solscan</a></p>`;
    }

    if (data.tokenMint) {
      html += `<p><strong>Token:</strong> <a href="https://solscan.io/token/${data.tokenMint}">View on Solscan</a></p>`;
    }

    html += `
          </div>
          
          <div class="footer">
            <p>This is an automated notification from your Solana Trading Bot.</p>
          </div>
        </body>
      </html>
    `;

    return html;
  }
}

// Create singleton instance
const notificationService = new NotificationService();

export default notificationService;
