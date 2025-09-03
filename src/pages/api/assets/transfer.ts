import type { NextApiRequest, NextApiResponse } from 'next';
import { prisma } from '../../../lib/prisma';

const ENCLAVE_URL = process.env.ENCLAVE_URL || 'http://localhost:5000';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { walletAddress, fromAddress, toAddress, assetId, amount } = req.body;

    if (!walletAddress || !fromAddress || !toAddress || !assetId || !amount) {
      return res.status(400).json({ 
        error: 'Missing required fields: walletAddress, fromAddress, toAddress, assetId, amount' 
      });
    }

    // Validate wallet exists
    const wallet = await prisma.passWallet.findUnique({
      where: { address: walletAddress },
    });

    if (!wallet) {
      return res.status(404).json({ error: 'Wallet not found' });
    }

    // Find the from subaccount (sender)
    const fromSubaccount = await prisma.subaccount.findFirst({
      where: {
        walletId: wallet.id,
        address: fromAddress,
      },
    });

    if (!fromSubaccount) {
      return res.status(404).json({ error: 'From subaccount not found' });
    }

    // Find or create the to subaccount (receiver)
    let toSubaccount = await prisma.subaccount.findFirst({
      where: {
        walletId: wallet.id,
        address: toAddress,
      },
    });

    if (!toSubaccount) {
      // Create new subaccount for the recipient
      toSubaccount = await prisma.subaccount.create({
        data: {
          walletId: wallet.id,
          address: toAddress,
          label: `Subaccount ${toAddress.slice(0, 10)}...`,
        },
      });
    }

    // Call the enclave API to perform the transfer
    const enclaveResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/transfers`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        wallet_address: walletAddress,
        asset_id: assetId,
        amount: parseInt(amount),
        from_subaccount: fromSubaccount.id.toString(),
        to_subaccount: toSubaccount.id.toString(),
      }),
    });

    if (!enclaveResponse.ok) {
      const errorText = await enclaveResponse.text();
      console.error('Enclave transfer error:', errorText);
      
      try {
        const errorData = JSON.parse(errorText);
        return res.status(400).json({ 
          error: errorData.error || 'Transfer failed in enclave' 
        });
      } catch (parseError) {
        return res.status(400).json({ 
          error: `Transfer failed: ${errorText}` 
        });
      }
    }

    const transferResult = await enclaveResponse.json();
    console.log('Transfer successful:', transferResult);

    return res.status(200).json({ 
      success: true,
      message: 'Transfer completed successfully',
      transferResult 
    });

  } catch (error) {
    console.error('Error in transfer API:', error);
    return res.status(500).json({ 
      error: 'Internal server error',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
}