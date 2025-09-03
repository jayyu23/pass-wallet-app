import React, { useState } from 'react';
import { toast } from 'react-hot-toast';

interface WithdrawModalProps {
  isOpen: boolean;
  onClose: () => void;
  assetId: string;
  asset: any;
  walletAddress: string;
  connectedAddress: string;
  onWithdrawComplete?: () => void;
}

const WithdrawModal: React.FC<WithdrawModalProps> = ({
  isOpen,
  onClose,
  assetId,
  asset,
  walletAddress,
  connectedAddress,
  onWithdrawComplete
}) => {
  const [destinationAddress, setDestinationAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [gasPrice, setGasPrice] = useState('');
  const [gasLimit, setGasLimit] = useState('');
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
    // Get the connected subaccount balance by looking up the subaccount ID for this address
    // The asset.subaccount_balances contains subaccount IDs as keys, not addresses
    const connectedSubaccountBalance = Object.entries(asset.subaccount_balances || {})
      .find(([subaccountId, balance]) => {
        // We'll need to check if this subaccount belongs to the connected address
        // For now, use the first available balance as a fallback
        return true;
      })?.[1] || 0;
    const maxAmount = connectedSubaccountBalance.toString();
    const divisor = Math.pow(10, asset.decimals);
    const formattedMaxAmount = (parseFloat(maxAmount) / divisor).toFixed(asset.decimals);
    setAmount(formattedMaxAmount);
  };

  const handleWithdraw = async () => {
    if (!destinationAddress || !amount) {
      toast.error('Please fill in destination address and amount');
      return;
    }

    if (parseFloat(amount) <= 0) {
      toast.error('Amount must be greater than 0');
      return;
    }

    // Validate destination address format
    if (!destinationAddress.startsWith('0x') || destinationAddress.length !== 42) {
      toast.error('Invalid destination address format');
      return;
    }

    const amountInWei = Math.floor(parseFloat(amount) * Math.pow(10, asset.decimals));
    // Get the connected subaccount balance (same logic as handleMaxClick)
    const connectedSubaccountBalance = Object.entries(asset.subaccount_balances || {})
      .find(([subaccountId, balance]) => {
        // For now, use the first available balance as a fallback
        // In production, this should properly match the connected address to subaccount ID
        return true;
      })?.[1] || 0;

    if (amountInWei > Number(connectedSubaccountBalance)) {
      toast.error('Insufficient balance');
      return;
    }

    try {
      setIsLoading(true);

      const withdrawalRequest: {
        walletAddress: string;
        connectedAddress: string;
        assetId: string;
        amount: string;
        destination: string;
        chain_id: number;
        gas_price?: number;
        gas_limit?: number;
      } = {
        walletAddress,
        connectedAddress,
        assetId,
        amount: amountInWei.toString(),
        destination: destinationAddress,
        chain_id: 11155111 // Sepolia testnet
      };

      // Add gas parameters if provided
      if (gasPrice) {
        withdrawalRequest.gas_price = parseInt(gasPrice);
      }
      if (gasLimit) {
        withdrawalRequest.gas_limit = parseInt(gasLimit);
      }

      const response = await fetch('/api/assets/withdraw', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(withdrawalRequest),
      });

      if (!response.ok) {
        let errorMessage = 'Withdrawal failed';
        try {
          const errorData = await response.json();
          errorMessage = errorData.error || errorMessage;
        } catch {
          errorMessage = `Withdrawal failed with status ${response.status}`;
        }
        throw new Error(errorMessage);
      }

      const data = await response.json();
      
      if (data.success) {
        const signedTx = data.data.signed_raw_transaction;
        const txDetails = {
          signedTransaction: signedTx,
          nonce: data.data.nonce,
          gasPrice: data.data.gas_price,
          gasLimit: data.data.gas_limit,
          amount: data.data.amount,
          destination: data.data.destination,
          chainId: data.data.chain_id
        };

        // Show success message with signed transaction
        toast.success(
          <div style={{ whiteSpace: 'pre-wrap', maxWidth: '400px', fontSize: '12px' }}>
            <div style={{ fontWeight: 'bold', marginBottom: '8px' }}>
              Withdrawal Signed Successfully!
            </div>
            <div style={{ marginBottom: '8px' }}>
              <strong>Nonce:</strong> {txDetails.nonce}<br/>
              <strong>Gas Price:</strong> {txDetails.gasPrice} wei<br/>
              <strong>Gas Limit:</strong> {txDetails.gasLimit}<br/>
              <strong>Amount:</strong> {formatAmount(txDetails.amount, asset.decimals, asset.symbol)}<br/>
              <strong>To:</strong> {txDetails.destination.slice(0, 10)}...
            </div>
            <div style={{ marginBottom: '8px' }}>
              <strong>View Provenance Log for Signed Transaction</strong>
            </div>
            {/* <div style={{ 
              fontFamily: 'monospace', 
              fontSize: '10px', 
              backgroundColor: '#f5f5f5', 
              padding: '4px', 
              borderRadius: '4px',
              wordBreak: 'break-all',
              maxHeight: '100px',
              overflow: 'auto'
            }}>
              {signedTx}
            </div> */}
          </div>,
          { 
            duration: 10000,
            style: { maxWidth: '500px' }
          }
        );
        
        // Also log to console for easy copying
        console.log('=== WITHDRAWAL SIGNED TRANSACTION ===');
        console.log('Signed Raw Transaction:', signedTx);
        console.log('Transaction Details:', txDetails);
        console.log('You can submit this transaction to any Ethereum RPC endpoint');

        // Reset form
        setDestinationAddress('');
        setAmount('');
        setGasPrice('');
        setGasLimit('');
        onClose();
        
        if (onWithdrawComplete) {
          onWithdrawComplete();
        }
      } else {
        throw new Error(data.error || 'Withdrawal failed');
      }
    } catch (error: any) {
      console.error('Withdrawal error:', error);
      toast.error(error.message || 'Withdrawal failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      setDestinationAddress('');
      setAmount('');
      setGasPrice('');
      setGasLimit('');
      onClose();
    }
  };

  const overlayStyle: React.CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  };

  const modalStyle: React.CSSProperties = {
    backgroundColor: 'white',
    borderRadius: '12px',
    padding: '24px',
    width: '90%',
    maxWidth: '480px',
    maxHeight: '90vh',
    overflowY: 'auto',
    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
  };

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: '12px',
    border: '1px solid #ddd',
    borderRadius: '6px',
    fontSize: '16px',
    marginBottom: '16px',
    boxSizing: 'border-box',
  };

  const buttonStyle: React.CSSProperties = {
    padding: '12px 24px',
    borderRadius: '6px',
    border: 'none',
    fontSize: '16px',
    fontWeight: '500',
    cursor: 'pointer',
    marginRight: '12px',
  };

  // Get the connected subaccount balance for display
  const connectedSubaccountBalance = Object.entries(asset.subaccount_balances || {})
    .find(([subaccountId, balance]) => {
      // For now, use the first available balance as a fallback
      // In production, this should properly match the connected address to subaccount ID
      return true;
    })?.[1] || 0;

  return (
    <div style={overlayStyle} onClick={handleClose}>
      <div style={modalStyle} onClick={(e) => e.stopPropagation()}>
        <h2 style={{ marginTop: 0, marginBottom: '24px', color: '#333' }}>
          Withdraw {asset.symbol}
        </h2>
        
        <div style={{ marginBottom: '16px' }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
            Available Balance: {formatAmount(connectedSubaccountBalance.toString(), asset.decimals, asset.symbol)}
          </div>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
            Asset: {asset.name} ({asset.symbol})
          </div>
          {asset.contract_address && (
            <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
              Contract: {asset.contract_address}
            </div>
          )}
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', fontWeight: '500' }}>
            Destination Address *
          </label>
          <input
            style={inputStyle}
            type="text"
            placeholder="0x..."
            value={destinationAddress}
            onChange={(e) => setDestinationAddress(e.target.value)}
            disabled={isLoading}
          />
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', fontWeight: '500' }}>
            Amount *
          </label>
          <div style={{ display: 'flex', gap: '8px' }}>
            <input
              style={{ ...inputStyle, marginBottom: '0', flex: 1 }}
              type="number"
              placeholder={`0.0 ${asset.symbol}`}
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              disabled={isLoading}
              step="any"
            />
            <button
              style={{
                ...buttonStyle,
                backgroundColor: '#6c757d',
                color: 'white',
                marginRight: '0',
                flex: '0 0 auto'
              }}
              onClick={handleMaxClick}
              disabled={isLoading}
            >
              Max
            </button>
          </div>
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', fontWeight: '500' }}>
            Gas Price (optional)
          </label>
          <input
            style={inputStyle}
            type="number"
            placeholder="Gas price in wei"
            value={gasPrice}
            onChange={(e) => setGasPrice(e.target.value)}
            disabled={isLoading}
          />
        </div>

        <div style={{ marginBottom: '24px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', fontWeight: '500' }}>
            Gas Limit (optional)
          </label>
          <input
            style={inputStyle}
            type="number"
            placeholder="Gas limit"
            value={gasLimit}
            onChange={(e) => setGasLimit(e.target.value)}
            disabled={isLoading}
          />
        </div>

        <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
          <button
            style={{
              ...buttonStyle,
              backgroundColor: '#6c757d',
              color: 'white',
            }}
            onClick={handleClose}
            disabled={isLoading}
          >
            Cancel
          </button>
          <button
            style={{
              ...buttonStyle,
              backgroundColor: isLoading ? '#ccc' : '#dc3545',
              color: 'white',
              marginRight: '0',
            }}
            onClick={handleWithdraw}
            disabled={isLoading || !destinationAddress || !amount}
          >
            {isLoading ? 'Processing...' : 'Withdraw'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default WithdrawModal;