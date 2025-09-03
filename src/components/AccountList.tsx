import { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import { useAccount } from 'wagmi';
interface PassAccount {
  address: string;
  name: string;
  owner: string;
  createdAt: string;
}

// TODO: change this to the actual PASS Wallet address
const PASS_WALLET_ADDRESS = process.env.NEXT_PUBLIC_PASS_WALLET_ADDRESS || "";

const AccountsList = () => {
  const router = useRouter();
  const { address } = useAccount();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [newWalletName, setNewWalletName] = useState('');
  const [accounts, setAccounts] = useState<PassAccount[]>([]); // Start with empty array instead of default data

  const accountCardStyle = {
    padding: '20px',
    border: '1px solid #eaeaea',
    borderRadius: '12px',
    marginBottom: '16px',
    width: '100%',
    maxWidth: '600px',
    backgroundColor: 'white',
    transition: 'all 0.2s ease',
    cursor: 'pointer',
    '&:hover': {
      transform: 'translateY(-2px)',
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
    }
  };

  const buttonStyle = {
    padding: '12px 24px',
    backgroundColor: '#0d76fc',
    color: 'white',
    border: 'none',
    borderRadius: '8px',
    cursor: 'pointer',
    fontSize: '16px',
    fontWeight: '600',
    marginTop: '20px',
  };

  useEffect(() => {
    const fetchAccounts = async () => {
      try {
        const response = await fetch('/api/account/');
        const data = await response.json();
        console.log('API response:', data);
        
        // Ensure data is an array
        if (Array.isArray(data)) {
          setAccounts(data);
        } else {
          console.error('Expected array but got:', typeof data, data);
          setAccounts([]);
        }
      } catch (error) {
        console.error('Error fetching accounts:', error);
        setAccounts([]);
      }
    };
    fetchAccounts();
  }, []);

  const handleCreateNewAccount = async () => {
    const defaultName = `PassWallet ${accounts.length + 1}`;
    setNewWalletName(defaultName);
    setIsModalOpen(true);
  };

  const handleSubmitNewAccount = async () => {
    try {
      const response = await fetch('/api/account', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: newWalletName,
          owner: address,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to create wallet');
      }

      const newWallet = await response.json();
      setAccounts([...accounts, newWallet]);
      setIsModalOpen(false);
      setNewWalletName('');
    } catch (error) {
      console.error('Error creating wallet:', error);
    }
  };

  const handleAccountClick = (address: string) => {
    router.push(`/account/${address}`);
  };

  return (
    <div style={{ width: '100%', maxWidth: '600px', marginTop: '2rem' }}>
      <div style={{ marginBottom: '2rem' }}>
        <button
          onClick={handleCreateNewAccount}
          style={buttonStyle}
        >
          + Create New Account
        </button>
      </div>

      {accounts.map((account, index) => (
        <div 
          key={index} 
          style={accountCardStyle}
          onClick={() => handleAccountClick(account.address)}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <h3 style={{ margin: '0' }}>{account.name}</h3>
            <span style={{ color: '#666' }}>Created: {account.createdAt}</span>
          </div>
          
          <div style={{ marginTop: '12px', color: '#666' }}>
            <p style={{ margin: '4px 0' }}>Address: {account.address}</p>
            <p style={{ margin: '4px 0' }}>Owner: {account.owner}</p>
          </div>
        </div>
      ))}

      {accounts.length === 0 && (
        <div style={{ textAlign: 'center', color: '#666', marginTop: '2rem' }}>
          <p>You don't have any PASS accounts yet.</p>
          <p>Create your first account to get started!</p>
        </div>
      )}

      {isModalOpen && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.5)',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          zIndex: 1000,
        }}>
          <div style={{
            backgroundColor: 'white',
            padding: '24px',
            borderRadius: '12px',
            width: '90%',
            maxWidth: '400px',
          }}>
            <h3>Create New Account</h3>
            <input
              type="text"
              value={newWalletName}
              onChange={(e) => setNewWalletName(e.target.value)}
              placeholder="Enter wallet name"
              style={{
                width: '100%',
                padding: '8px',
                marginBottom: '16px',
                borderRadius: '4px',
                border: '1px solid #eaeaea',
              }}
            />
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
              <button
                onClick={() => {
                  setIsModalOpen(false);
                  setNewWalletName('');
                }}
                style={{
                  ...buttonStyle,
                  backgroundColor: '#dc3545',
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleSubmitNewAccount}
                style={buttonStyle}
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AccountsList;