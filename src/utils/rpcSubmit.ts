import { createPublicClient, http } from 'viem';
import { sepolia } from 'viem/chains';

// Create Viem client for Sepolia
const createSepoliaClient = () => {
  const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL || 'https://eth-sepolia.g.alchemy.com/v2/LkVcda5a_AEu1VGxzs_WNcMG53-v0reS';
  
  return createPublicClient({
    chain: sepolia,
    transport: http(rpcUrl)
  });
};

/**
 * Submit a signed raw transaction to Sepolia RPC
 * @param signedRawTransaction - The hex-encoded signed transaction
 * @returns Promise with transaction hash or error
 */
export async function submitTransactionToRPC(signedRawTransaction: string): Promise<{
  success: boolean;
  txHash?: string;
  error?: string;
}> {
  try {
    const client = createSepoliaClient();
    
    // Ensure the transaction starts with 0x
    const formattedTx = signedRawTransaction.startsWith('0x') 
      ? signedRawTransaction 
      : `0x${signedRawTransaction}`;
    
    // Submit the raw transaction
    const txHash = await client.sendRawTransaction({
      serializedTransaction: formattedTx as `0x${string}`
    });
    
    return {
      success: true,
      txHash
    };
  } catch (error: any) {
    console.error('Transaction submission failed:', error);
    
    // Extract meaningful error message
    let errorMessage = 'Failed to submit transaction';
    if (error?.cause?.reason) {
      errorMessage = error.cause.reason;
    } else if (error?.cause?.message) {
      errorMessage = error.cause.message;
    } else if (error.message) {
      // Clean up common error messages
      if (error.message.includes('insufficient funds')) {
        errorMessage = 'Insufficient funds for gas fees';
      } else if (error.message.includes('nonce too low')) {
        errorMessage = 'Transaction nonce is too low (already used)';
      } else if (error.message.includes('nonce too high')) {
        errorMessage = 'Transaction nonce is too high';
      } else if (error.message.includes('gas price too low')) {
        errorMessage = 'Gas price is too low';
      } else if (error.message.includes('replacement transaction underpriced')) {
        errorMessage = 'Replacement transaction underpriced';
      } else {
        errorMessage = error.message;
      }
    } else if (error.details) {
      errorMessage = error.details;
    } else if (typeof error === 'string') {
      errorMessage = error;
    }
    
    return {
      success: false,
      error: errorMessage
    };
  }
}

/**
 * Get transaction status from Sepolia
 * @param txHash - Transaction hash to check
 * @returns Promise with transaction receipt or error
 */
export async function getTransactionStatus(txHash: string): Promise<{
  success: boolean;
  receipt?: any;
  error?: string;
}> {
  try {
    const client = createSepoliaClient();
    
    const receipt = await client.getTransactionReceipt({
      hash: txHash as `0x${string}`
    });
    
    return {
      success: true,
      receipt
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Failed to get transaction status'
    };
  }
}