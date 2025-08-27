import { NextApiRequest, NextApiResponse } from 'next';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { 
      walletAddress, 
      connectedAddress, 
      assetId, 
      amount, 
      destination,
      gas_price,
      gas_limit,
      chain_id = 11155111
    } = req.body;

    if (!walletAddress || !connectedAddress || !assetId || !amount || !destination) {
      return res.status(400).json({ 
        error: 'Missing required fields: walletAddress, connectedAddress, assetId, amount, destination' 
      });
    }

    // First find the wallet
    const wallet = await prisma.passWallet.findUnique({
      where: {
        address: walletAddress
      }
    });

    if (!wallet) {
      return res.status(404).json({ error: 'Wallet not found' });
    }

    // Find the subaccount for this wallet and connected address
    // If not found in database, try a common subaccount ID that might exist in enclave
    let subaccountId = "13"; // Default to common subaccount ID
    
    try {
      const subaccount = await prisma.subaccount.findFirst({
        where: {
          walletId: wallet.id,
          address: connectedAddress
        }
      });
      
      if (subaccount) {
        subaccountId = subaccount.id.toString();
      }
    } catch (error) {
      console.log('Database subaccount lookup failed, using default:', error);
    }

    // Prepare the withdrawal request for the enclave
    // Fetch on-chain nonce and current gas price
    const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL || 'https://eth-sepolia.g.alchemy.com/v2/LkVcda5a_AEu1VGxzs_WNcMG53-v0reS';
    
    let onChainNonce: number | null = null;
    let currentGasPrice: number | null = null;
    
    try {
      // Get on-chain nonce
      const nonceResponse = await fetch(rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'eth_getTransactionCount',
          params: [walletAddress, 'pending'],
          id: 1
        })
      });
      
      if (nonceResponse.ok) {
        const nonceData = await nonceResponse.json();
        if (nonceData.result) {
          onChainNonce = parseInt(nonceData.result, 16);
        }
      }
      
      // Get current gas price if not provided
      if (!gas_price) {
        const gasPriceResponse = await fetch(rpcUrl, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'eth_gasPrice',
            params: [],
            id: 2
          })
        });
        
        if (gasPriceResponse.ok) {
          const gasPriceData = await gasPriceResponse.json();
          if (gasPriceData.result) {
            const networkGasPrice = parseInt(gasPriceData.result, 16);
            // Add 20% buffer for faster confirmation
            currentGasPrice = Math.max(Math.floor(networkGasPrice * 1.2), 1_000_000_000); // Minimum 1 gwei
          }
        }
      }
    } catch (error) {
      console.warn('Failed to fetch on-chain data, using fallbacks:', error);
    }

    const withdrawalRequest = {
      wallet_address: walletAddress,
      subaccount_id: subaccountId,
      asset_id: assetId,
      amount: parseInt(amount),
      destination: destination,
      chain_id: chain_id,
      // Always include gas parameters with defaults if not provided
      gas_price: gas_price ? parseInt(gas_price) : (currentGasPrice || null),
      gas_limit: gas_limit ? parseInt(gas_limit) : null,
      override_nonce: onChainNonce
    };

    // Get enclave URL from environment
    const enclaveUrl = process.env.NEXT_PUBLIC_ENCLAVE_URL || 'http://localhost:5000';

    // Call the enclave withdrawal endpoint
    const enclaveResponse = await fetch(`${enclaveUrl}/pass/wallets/withdrawals/external`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(withdrawalRequest),
    });

    if (!enclaveResponse.ok) {
      let errorMessage = 'Enclave withdrawal failed';
      try {
        const errorData = await enclaveResponse.text();
        console.error('Enclave error response:', errorData);
        const parsedError = JSON.parse(errorData);
        errorMessage = parsedError.error || errorMessage;
      } catch {
        errorMessage = `Enclave returned status ${enclaveResponse.status}`;
      }
      
      console.error('Enclave withdrawal failed:', errorMessage);
      return res.status(enclaveResponse.status).json({ 
        error: errorMessage 
      });
    }

    const enclaveData = await enclaveResponse.json();
    console.log('Enclave response:', enclaveData);

    // Check if enclave returned signed transaction directly (new format)
    if (enclaveData.signed_raw_transaction) {
      return res.status(200).json({
        success: true,
        data: {
          signed_raw_transaction: enclaveData.signed_raw_transaction,
          wallet_address: enclaveData.wallet_address,
          subaccount_id: enclaveData.subaccount_id,
          subaccount_address: connectedAddress,
          asset_id: enclaveData.asset_id,
          amount: enclaveData.amount,
          destination: enclaveData.destination,
          chain_id: enclaveData.chain_id,
          nonce: enclaveData.nonce,
          gas_price: enclaveData.gas_price,
          gas_limit: enclaveData.gas_limit
        }
      });
    }
    // Handle legacy format with success field
    else if (enclaveData.success) {
      return res.status(200).json({
        success: true,
        data: {
          signed_raw_transaction: enclaveData.data.signed_raw_transaction,
          wallet_address: enclaveData.data.wallet_address,
          subaccount_id: enclaveData.data.subaccount_id,
          subaccount_address: connectedAddress,
          asset_id: enclaveData.data.asset_id,
          amount: enclaveData.data.amount,
          destination: enclaveData.data.destination,
          chain_id: enclaveData.data.chain_id,
          nonce: enclaveData.data.nonce,
          gas_price: enclaveData.data.gas_price,
          gas_limit: enclaveData.data.gas_limit
        }
      });
    } else {
      console.error('Withdrawal failed - enclave response:', enclaveData);
      return res.status(400).json({ 
        error: enclaveData.error || 'Withdrawal failed' 
      });
    }

  } catch (error: any) {
    console.error('Withdrawal API error:', error);
    return res.status(500).json({ 
      error: error.message || 'Internal server error' 
    });
  }
}