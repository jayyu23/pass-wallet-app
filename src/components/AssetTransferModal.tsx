import React, { useState } from 'react';
import { toast } from 'react-hot-toast';

interface AssetTransferModalProps {
  isOpen: boolean;
  onClose: () => void;
  assetId: string;
  asset: any;
  walletAddress: string;
  connectedAddress: string;
  onTransferComplete?: () => void;
}

const AssetTransferModal: React.FC<AssetTransferModalProps> = ({
  isOpen,
  onClose,
  assetId,
  asset,
  walletAddress,
  connectedAddress,
  onTransferComplete
}) => {
  const [destinationAddress, setDestinationAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  if (!isOpen) return null;

  const formatAmount = (amount: string, decimals: number, symbol: string) => {
    if (decimals === 0) {
      return `${amount} ${symbol}`;
    }
    
    const divisor = Math.pow(10, decimals);
    const formattedAmount = (parseFloat(amount) / divisor).toFixed(6);
    return `${formattedAmount} ${symbol}`;
  };

  const handleMaxClick = () => {
    const maxAmount = asset.total_balance.toString();
    const divisor = Math.pow(10, asset.decimals);
    const formattedMaxAmount = (parseFloat(maxAmount) / divisor).toFixed(asset.decimals);
    setAmount(formattedMaxAmount);
  };

  const handleSend = async () => {
    if (!destinationAddress || !amount) {
      toast.error('Please fill in all fields');
      return;
    }

    if (parseFloat(amount) <= 0) {
      toast.error('Amount must be greater than 0');
      return;
    }

    const amountInWei = Math.floor(parseFloat(amount) * Math.pow(10, asset.decimals));
    if (amountInWei > asset.total_balance) {
      toast.error('Insufficient balance');
      return;
    }

    try {
      setIsLoading(true);

      const response = await fetch('/api/assets/transfer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          walletAddress,
          fromAddress: connectedAddress,
          toAddress: destinationAddress,
          assetId,
          amount: amountInWei.toString()
        }),
      });

      if (!response.ok) {
        let errorMessage = 'Transfer failed';
        try {
          const errorData = await response.json();
          errorMessage = errorData.error || 'Transfer failed';
        } catch (parseError) {
          // If response is not JSON, try to get text
          try {
            const errorText = await response.text();
            errorMessage = errorText || `Transfer failed: ${response.status}`;
          } catch (textError) {
            errorMessage = `Transfer failed: ${response.status} ${response.statusText}`;
          }
        }
        throw new Error(errorMessage);
      }

      toast.success('Transfer completed successfully!');
      onClose();
      if (onTransferComplete) {
        onTransferComplete();
      }
    } catch (error) {
      console.error('Transfer error:', error);
      toast.error(`Transfer failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    setDestinationAddress('');
    setAmount('');
    onClose();
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.5)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000
    }}>
      <div style={{
        backgroundColor: 'white',
        padding: '32px',
        borderRadius: '12px',
        minWidth: '400px',
        maxWidth: '500px',
        width: '90%'
      }}>
        <h2 style={{ marginBottom: '24px', fontSize: '20px', fontWeight: '600' }}>
          Transfer Asset
        </h2>

        <div style={{ marginBottom: '20px' }}>
          <div style={{ 
            padding: '16px', 
            backgroundColor: '#f8f9fa', 
            borderRadius: '8px',
            border: '1px solid #e9ecef'
          }}>
            <div style={{ fontWeight: '500', marginBottom: '4px' }}>
              {asset.name} ({asset.symbol})
            </div>
            <div style={{ color: '#666', fontSize: '0.9rem' }}>
              Available: {formatAmount(asset.total_balance.toString(), asset.decimals, asset.symbol)}
            </div>
            {asset.contract_address && (
              <div style={{ color: '#666', fontSize: '0.8rem', marginTop: '4px' }}>
                Contract: {asset.contract_address}
              </div>
            )}
          </div>
        </div>

        <div style={{ marginBottom: '20px' }}>
          <label style={{ 
            display: 'block', 
            marginBottom: '8px', 
            fontWeight: '500' 
          }}>
            Destination Subaccount Address
          </label>
          <input
            type="text"
            value={destinationAddress}
            onChange={(e) => setDestinationAddress(e.target.value)}
            placeholder="0x..."
            style={{
              width: '100%',
              padding: '12px',
              border: '1px solid #ddd',
              borderRadius: '6px',
              fontSize: '14px'
            }}
          />
        </div>

        <div style={{ marginBottom: '24px' }}>
          <label style={{ 
            display: 'block', 
            marginBottom: '8px', 
            fontWeight: '500' 
          }}>
            Amount
          </label>
          <div style={{ display: 'flex', gap: '8px' }}>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              step={`0.${'0'.repeat(asset.decimals - 1)}1`}
              style={{
                flex: 1,
                padding: '12px',
                border: '1px solid #ddd',
                borderRadius: '6px',
                fontSize: '14px'
              }}
            />
            <button
              onClick={handleMaxClick}
              style={{
                padding: '12px 16px',
                backgroundColor: '#f8f9fa',
                border: '1px solid #ddd',
                borderRadius: '6px',
                cursor: 'pointer',
                fontSize: '12px',
                color: '#666'
              }}
            >
              Max
            </button>
          </div>
        </div>

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            onClick={handleCancel}
            disabled={isLoading}
            style={{
              padding: '12px 24px',
              backgroundColor: '#6c757d',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              opacity: isLoading ? 0.6 : 1
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSend}
            disabled={isLoading}
            style={{
              padding: '12px 24px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              opacity: isLoading ? 0.6 : 1
            }}
          >
            {isLoading ? 'Sending...' : 'Send'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default AssetTransferModal;