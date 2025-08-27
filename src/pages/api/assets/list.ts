import type { NextApiRequest, NextApiResponse } from 'next';
import { prisma } from '../../../lib/prisma';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { wallet_address, connected_address } = req.body;
    
    if (!wallet_address) {
      return res.status(400).json({ error: 'wallet_address is required' });
    }

    if (!connected_address) {
      return res.status(400).json({ error: 'connected_address is required' });
    }

    // Call the enclave API to get all assets
    const enclaveUrl = process.env.ENCLAVE_URL || 'http://localhost:5000';
    const enclaveResponse = await fetch(`${enclaveUrl}/pass/wallets/assets/list`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ wallet_address }),
    });

    if (!enclaveResponse.ok) {
      const errorText = await enclaveResponse.text();
      console.error('Enclave API error:', errorText);
      return res.status(enclaveResponse.status).json({ 
        error: `Enclave API error: ${errorText}` 
      });
    }

    const assetsData = await enclaveResponse.json();

    // Find which subaccount the connected address belongs to
    const wallet = await prisma.passWallet.findUnique({
      where: { address: wallet_address },
    });

    if (!wallet) {
      return res.status(404).json({ error: 'Wallet not found' });
    }

    const subaccount = await prisma.subaccount.findFirst({
      where: {
        walletId: wallet.id,
        address: connected_address,
      },
    });

    if (!subaccount) {
      // If no subaccount found, user has no access to any balances
      const filteredAssetsData = {
        ...assetsData,
        assets: Object.fromEntries(
          Object.entries(assetsData.assets || {}).map(([assetId, asset]: [string, any]) => [
            assetId,
            {
              ...asset,
              total_balance: 0,
              subaccount_balances: {}
            }
          ])
        )
      };
      return res.status(200).json(filteredAssetsData);
    }

    // Filter assets to only show the connected user's subaccount balance
    const filteredAssetsData = {
      ...assetsData,
      assets: Object.fromEntries(
        Object.entries(assetsData.assets || {}).map(([assetId, asset]: [string, any]) => {
          const userBalance = asset.subaccount_balances?.[subaccount.id] || 0;
          return [
            assetId,
            {
              ...asset,
              total_balance: userBalance,
              subaccount_balances: {
                [subaccount.id]: userBalance
              }
            }
          ];
        })
      )
    };

    return res.status(200).json(filteredAssetsData);

  } catch (error) {
    console.error('Error in assets list API:', error);
    return res.status(500).json({ 
      error: 'Internal server error',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
}