import type { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { wallet_address } = req.body;

    if (!wallet_address) {
      return res.status(400).json({ error: 'Wallet address is required' });
    }

    // Get full provenance data from enclave
    const ENCLAVE_URL = process.env.ENCLAVE_URL || 'http://localhost:5000';
    
    const enclaveResponse = await fetch(`${ENCLAVE_URL}/pass/wallets/provenance`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address: wallet_address
      }),
    });

    if (!enclaveResponse.ok) {
      const errorText = await enclaveResponse.text();
      console.error('Enclave error:', errorText);
      return res.status(500).json({ error: `Enclave error: ${errorText}` });
    }

    const provenanceData = await enclaveResponse.json();
    
    res.status(200).json(provenanceData);
  } catch (error) {
    console.error('Error fetching provenance data:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
}