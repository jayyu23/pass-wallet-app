import type { NextApiRequest, NextApiResponse } from 'next';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { wallet_address, connected_address } = req.body;

    if (!wallet_address) {
      return res.status(400).json({ error: 'Wallet address is required' });
    }

    if (!connected_address) {
      return res.status(400).json({ error: 'Connected address is required' });
    }

    // Look up the subaccount ID for the connected address
    const wallet = await prisma.passWallet.findUnique({
      where: { address: wallet_address },
      include: { subaccounts: true }
    });

    if (!wallet) {
      return res.status(404).json({ error: 'Wallet not found' });
    }

    const subaccount = wallet.subaccounts.find(
      sa => sa.address.toLowerCase() === connected_address.toLowerCase()
    );

    if (!subaccount) {
      return res.status(404).json({ error: 'Subaccount not found for connected address' });
    }

    // Get provenance data from enclave, filtered by subaccount
    const ENCLAVE_URL = process.env.ENCLAVE_URL || 'http://localhost:5000';
    
    const enclaveResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/provenance/subaccount`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: wallet_address,
        subaccount_id: subaccount.id.toString()
      }),
    });

    if (!enclaveResponse.ok) {
      const errorText = await enclaveResponse.text();
      console.error('Enclave error:', errorText);
      return res.status(500).json({ error: `Enclave error: ${errorText}` });
    }

    const provenanceData = await enclaveResponse.json();
    
    // Create a mapping of subaccount IDs to addresses for the frontend
    const subaccountMapping: { [key: string]: string } = {};
    wallet.subaccounts.forEach(sa => {
      subaccountMapping[sa.id.toString()] = sa.address;
    });
    
    // Add additional metadata for the frontend
    const responseData = {
      ...provenanceData,
      subaccount_address: connected_address,
      subaccount_id: subaccount.id.toString(),
      wallet_address: wallet_address,
      subaccount_mapping: subaccountMapping
    };

    res.status(200).json(responseData);
  } catch (error) {
    console.error('Error fetching provenance data:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
}