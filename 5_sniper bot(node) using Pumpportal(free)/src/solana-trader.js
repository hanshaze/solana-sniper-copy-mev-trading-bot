import {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  
} from '@solana/web3.js';
import {
  createSwapInstruction,
  getAssociatedTokenAddressSync,
  getAccount,
  createCloseAccountInstruction as createCloseTokenAccountInstruction
} from '@solana/spl-token';
import bs58 from 'bs58';

export class SolanaTrader {
  constructor(rpcUrl, privateKey) {
    this.connection = new Connection(rpcUrl, 'confirmed');
    this.wallet = this.loadWallet(privateKey);
    this.pumpFunProgramId = new PublicKey('6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P'); // PumpFun program ID
  }

  loadWallet(privateKey) {
    try {
      const secretKey = bs58.decode(privateKey);
      return Keypair.fromSecretKey(secretKey);
    } catch (error) {
      throw new Error(`Invalid private key: ${error.message}`);
    }
  }

  async getWalletBalance() {
    try {
      const balance = await this.connection.getBalance(this.wallet.publicKey);
      return balance / LAMPORTS_PER_SOL;
    } catch (error) {
      console.error('❌ Error getting wallet balance:', error);
      throw error;
    }
  }

  async getTokenBalance(mintAddress) {
    try {
      const mint = new PublicKey(mintAddress);
      const tokenAccount = await getAssociatedTokenAddress(
        mint,
        this.wallet.publicKey
      );

      try {
        const accountInfo = await getAccount(this.connection, tokenAccount);
        return Number(accountInfo.amount);
      } catch (error) {
        // Token account doesn't exist
        return 0;
      }
    } catch (error) {
      console.error('❌ Error getting token balance:', error);
      return 0;
    }
  }

  async buyToken(mintAddress, solAmount, slippageTolerance = 0.1) {
    try {
      console.log(`🛒 Buying token ${mintAddress} with ${solAmount} SOL`);
      
      const mint = new PublicKey(mintAddress);
      const walletPublicKey = this.wallet.publicKey;
      
      // Get associated token account
      const tokenAccount = await getAssociatedTokenAddress(mint, walletPublicKey);
      
      // Check if token account exists, create if not
      let createTokenAccountIx;
      try {
        await getAccount(this.connection, tokenAccount);
      } catch (error) {
        // Token account doesn't exist, create it
        createTokenAccountIx = createAssociatedTokenAccountInstruction(
          walletPublicKey, // payer
          tokenAccount, // associated token account
          walletPublicKey, // owner
          mint // mint
        );
      }

      // Create the swap instruction for PumpFun
      const swapInstruction = await this.createPumpFunSwapInstruction(
        mint,
        tokenAccount,
        solAmount * LAMPORTS_PER_SOL,
        slippageTolerance
      );

      // Build transaction
      const transaction = new Transaction();
      
      if (createTokenAccountIx) {
        transaction.add(createTokenAccountIx);
      }
      
      transaction.add(swapInstruction);

      // Send transaction
      const signature = await sendAndConfirmTransaction(
        this.connection,
        transaction,
        [this.wallet],
        {
          commitment: 'confirmed',
          maxRetries: 3
        }
      );

      console.log(`✅ Buy transaction successful: ${signature}`);
      return {
        success: true,
        signature,
        tokenAccount
      };

    } catch (error) {
      console.error('❌ Error buying token:', error);
      return {
        success: false,
        error: error.message
      };
    }
  }

  async sellToken(mintAddress, tokenAccount, slippageTolerance = 0.1) {
    try {
      console.log(`💰 Selling token ${mintAddress}`);
      
      const mint = new PublicKey(mintAddress);
      const walletPublicKey = this.wallet.publicKey;
      
      // Get token balance
      const tokenBalance = await this.getTokenBalance(mintAddress);
      if (tokenBalance === 0) {
        throw new Error('No tokens to sell');
      }

      // Create the swap instruction for PumpFun (reverse swap)
      const swapInstruction = await this.createPumpFunSwapInstruction(
        mint,
        tokenAccount,
        -tokenBalance, // Negative amount for selling
        slippageTolerance
      );

      // Build transaction
      const transaction = new Transaction();
      transaction.add(swapInstruction);

      // Send transaction
      const signature = await sendAndConfirmTransaction(
        this.connection,
        transaction,
        [this.wallet],
        {
          commitment: 'confirmed',
          maxRetries: 3
        }
      );

      console.log(`✅ Sell transaction successful: ${signature}`);
      return {
        success: true,
        signature
      };

    } catch (error) {
      console.error('❌ Error selling token:', error);
      return {
        success: false,
        error: error.message
      };
    }
  }

  async createPumpFunSwapInstruction(mint, tokenAccount, amount, slippageTolerance) {
    // This is a simplified version - in reality, you'd need to interact with PumpFun's specific program
    // For now, we'll create a basic transfer instruction as a placeholder
    
    const walletPublicKey = this.wallet.publicKey;
    
    if (amount > 0) {
      // Buying - transfer SOL to get tokens
      return SystemProgram.transfer({
        fromPubkey: walletPublicKey,
        toPubkey: this.pumpFunProgramId,
        lamports: amount
      });
    } else {
      // Selling - this would need to be implemented based on PumpFun's actual program
      // For now, we'll return a placeholder
      throw new Error('Sell functionality needs to be implemented based on PumpFun program');
    }
  }

  async getTokenMetadata(mintAddress) {
    try {
      const mint = new PublicKey(mintAddress);
      const accountInfo = await this.connection.getAccountInfo(mint);
      
      if (!accountInfo) {
        throw new Error('Token not found');
      }

      return {
        mint: mintAddress,
        decimals: accountInfo.data.readUInt8(44), // Standard position for decimals
        supply: accountInfo.data.readBigUInt64LE(36), // Standard position for supply
        isInitialized: accountInfo.data.readUInt8(36) === 1
      };
    } catch (error) {
      console.error('❌ Error getting token metadata:', error);
      throw error;
    }
  }

  async waitForConfirmation(signature, timeout = 30000) {
    try {
      const confirmation = await this.connection.confirmTransaction(signature, 'confirmed');
      return confirmation.value.err === null;
    } catch (error) {
      console.error('❌ Error waiting for confirmation:', error);
      return false;
    }
  }

  // Utility method to check if a token is a valid PumpFun token
  async isPumpFunToken(mintAddress) {
    try {
      const mint = new PublicKey(mintAddress);
      const accountInfo = await this.connection.getAccountInfo(mint);
      
      if (!accountInfo) {
        return false;
      }

      // Check if the token is associated with PumpFun program
      // This is a simplified check - you might need more sophisticated validation
      return accountInfo.owner.equals(this.pumpFunProgramId);
    } catch (error) {
      console.error('❌ Error checking PumpFun token:', error);
      return false;
    }
  }
}
