import type { NextApiRequest, NextApiResponse } from 'next';
import { prisma } from '../../../lib/prisma';
import { TokenType } from '@prisma/client';

const ENCLAVE_URL = process.env.ENCLAVE_URL || 'http://127.0.0.1:5000';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<{ success: boolean } | { error: string }>
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { walletAddress, transactionHash, claimAddress } = req.body;
    console.log("claim api called");
    console.log(walletAddress, transactionHash, claimAddress);

    // Get transaction from database for reference, including asset information
    const transaction = await prisma.inboxTransaction.findUnique({
      where: { transactionHash },
      include: {
        asset: true // Include the asset information
      }
    });

    if (!transaction) {
      return res.status(404).json({ error: 'Transaction not found' });
    }

    if (transaction.claimed) {
      return res.status(400).json({ error: 'Transaction already claimed' });
    }

    // Check if user is allowed to claim
    if (transaction.fromAddress.toLowerCase() !== claimAddress.toLowerCase()) {
      return res.status(400).json({ error: 'Transaction not from wallet' });
    }

    // Get or create subaccount ID
    let subaccount = await prisma.subaccount.findFirst({
      where: {
        walletId: transaction.walletId,
        address: claimAddress,
      },
    });

    if (!subaccount) {
      subaccount = await prisma.subaccount.create({
        data: {
          walletId: transaction.walletId,
          address: claimAddress,
          label: `Auto-${claimAddress.slice(0, 6)}`,
        },
      });
    }

    // First, ensure the asset is registered in the enclave
    if (transaction.asset) {
      console.log('Registering asset in enclave:', transaction.asset.symbol);
      
      // Map database token type to enclave token type
      const tokenTypeMap: Record<string, string> = {
        'ETH': 'ETH',
        'ERC20': 'ERC20', 
        'ERC721': 'ERC721',
        'ERC1155': 'ERC1155'
      };

      const assetRegistrationResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/assets`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet_address: walletAddress,
          asset_id: transaction.asset.symbol.toLowerCase(),
          token_type: tokenTypeMap[transaction.asset.tokenType] || transaction.asset.tokenType,
          contract_address: transaction.asset.contractAddress,
          token_id: transaction.asset.tokenId,
          symbol: transaction.asset.symbol,
          name: transaction.asset.name,
          decimals: transaction.asset.decimals,
        }),
      });

      // Asset registration failure is not critical - it might already exist
      if (!assetRegistrationResponse.ok) {
        const errorText = await assetRegistrationResponse.text();
        console.log('Asset registration response (may already exist):', errorText);
      } else {
        console.log('Asset successfully registered in enclave');
      }
    }

    // Ensure the deposit exists in the enclave before claiming
    if (transaction.asset) {
      console.log('Creating deposit in enclave for transaction:', transactionHash);
      
      const depositResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/deposits`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet_address: walletAddress,
          asset_id: transaction.asset.symbol.toLowerCase(),
          amount: Number(transaction.amount),
          deposit_id: transactionHash,
          transaction_hash: transactionHash,
          block_number: transaction.blockNumber,
          from_address: transaction.fromAddress,
          to_address: transaction.toAddress,
        }),
      });

      // Deposit creation failure is not critical - it might already exist
      if (!depositResponse.ok) {
        const errorText = await depositResponse.text();
        console.log('Deposit creation response (may already exist):', errorText);
      } else {
        console.log('Deposit successfully created in enclave');
      }
    }

    // Call enclave to perform the claim
    const enclaveResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/claims`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: walletAddress,
        deposit_id: transactionHash, // Use transaction hash as deposit ID
        subaccount_id: subaccount.id.toString(), // Convert to string as expected by enclave
      }),
    });

    if (!enclaveResponse.ok) {
      const errorText = await enclaveResponse.text();
      console.error('Enclave error response:', errorText);
      try {
        const errorData = JSON.parse(errorText);
        throw new Error(errorData.error || 'Failed to claim in enclave');
      } catch (parseError) {
        throw new Error(`Enclave error: ${errorText}`);
      }
    }

    // Mark as claimed in database for UI purposes
    await prisma.inboxTransaction.update({
      where: { transactionHash },
      data: {
        claimed: true,
        claimedAt: new Date(),
      },
    });

    return res.status(200).json({ success: true });
  } catch (error) {
    console.error('Error claiming transaction:', error);
    return res.status(500).json({ error: 'Internal server error' });
  }
}
