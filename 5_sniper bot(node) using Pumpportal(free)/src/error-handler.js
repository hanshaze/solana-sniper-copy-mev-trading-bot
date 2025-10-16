import { logger } from './logger.js';

export class ErrorHandler {
  static handle(error, context = 'Unknown') {
    logger.error(`Error in ${context}:`, error);
    
    // Log error details
    if (error.stack) {
      logger.debug('Stack trace:', error.stack);
    }
    
    // Handle specific error types
    if (error.code) {
      this.handleSolanaError(error, context);
    } else if (error.message) {
      this.handleGenericError(error, context);
    }
    
    // Return user-friendly error message
    return this.getUserFriendlyMessage(error);
  }

  static handleSolanaError(error, context) {
    switch (error.code) {
      case 4001:
        logger.warn('User rejected transaction');
        break;
      case 4100:
        logger.error('Unauthorized - check wallet connection');
        break;
      case 4200:
        logger.error('Unsupported method');
        break;
      case 4900:
        logger.error('Wallet disconnected');
        break;
      case 5900:
        logger.error('Transaction rejected');
        break;
      default:
        logger.error(`Solana error ${error.code}: ${error.message}`);
    }
  }

  static handleGenericError(error, context) {
    if (error.message.includes('insufficient funds')) {
      logger.error('Insufficient funds for transaction');
    } else if (error.message.includes('network')) {
      logger.error('Network connection error');
    } else if (error.message.includes('timeout')) {
      logger.warn('Operation timed out');
    } else if (error.message.includes('rate limit')) {
      logger.warn('Rate limit exceeded');
    } else {
      logger.error(`Generic error: ${error.message}`);
    }
  }

  static getUserFriendlyMessage(error) {
    if (error.message.includes('insufficient funds')) {
      return 'Insufficient funds. Please add more SOL to your wallet.';
    } else if (error.message.includes('network')) {
      return 'Network error. Please check your internet connection.';
    } else if (error.message.includes('timeout')) {
      return 'Operation timed out. Please try again.';
    } else if (error.message.includes('rate limit')) {
      return 'Too many requests. Please wait before trying again.';
    } else if (error.code === 4001) {
      return 'Transaction was rejected by user.';
    } else if (error.code === 4100) {
      return 'Unauthorized. Please check your wallet connection.';
    } else {
      return 'An unexpected error occurred. Please try again.';
    }
  }

  static async withRetry(operation, maxRetries = 3, delay = 1000) {
    let lastError;
    
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;
        logger.warn(`Attempt ${attempt}/${maxRetries} failed:`, error.message);
        
        if (attempt < maxRetries) {
          logger.info(`Retrying in ${delay}ms...`);
          await this.sleep(delay);
          delay *= 2; // Exponential backoff
        }
      }
    }
    
    throw lastError;
  }

  static sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  static isRetryableError(error) {
    const retryableErrors = [
      'timeout',
      'network',
      'rate limit',
      'ECONNRESET',
      'ENOTFOUND',
      'ECONNREFUSED'
    ];
    
    return retryableErrors.some(keyword => 
      error.message.toLowerCase().includes(keyword)
    );
  }
}
