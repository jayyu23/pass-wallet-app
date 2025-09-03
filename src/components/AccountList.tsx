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
    border: '1px solid #e2e8f0',
    borderRadius: '12px',
    marginBottom: '16px',
    width: '100%',
    backgroundColor: 'white',
    transition: 'all 0.2s ease',
    cursor: 'pointer',
    boxShadow: '0 2px 4px rgba(0, 0, 0, 0.05)',
  };

  const buttonStyle = {
    padding: '12px 24px',
    backgroundColor: '#667eea',
    color: 'white',
    border: 'none',
    borderRadius: '8px',
    cursor: 'pointer',
    fontSize: '16px',
    fontWeight: '600',
    transition: 'all 0.2s ease',
    boxShadow: '0 2px 4px rgba(102, 126, 234, 0.2)',
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
    <div style={{ width: '100%', maxWidth: '600px' }}>
      <div style={{ marginBottom: '2rem', textAlign: 'center' }}>
        <button
          onClick={handleCreateNewAccount}
          style={buttonStyle}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#5a67d8';
            e.currentTarget.style.transform = 'translateY(-1px)';
            e.currentTarget.style.boxShadow = '0 4px 8px rgba(102, 126, 234, 0.3)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = '#667eea';
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = '0 2px 4px rgba(102, 126, 234, 0.2)';
          }}
        >
          + Create New Account
        </button>
      </div>

      {accounts.map((account, index) => (
        <div 
          key={index} 
          style={accountCardStyle}
          onClick={() => handleAccountClick(account.address)}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'translateY(-2px)';
            e.currentTarget.style.boxShadow = '0 8px 20px rgba(0, 0, 0, 0.1)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = '0 2px 4px rgba(0, 0, 0, 0.05)';
          }}
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
          padding: '20px',
        }}>
          <div style={{
            backgroundColor: 'white',
            padding: '32px',
            borderRadius: '12px',
            width: '100%',
            maxWidth: '400px',
            boxShadow: '0 20px 40px rgba(0, 0, 0, 0.15)',
          }}>
            <h3 style={{ 
              margin: '0 0 24px 0', 
              fontSize: '1.5rem', 
              fontWeight: '600',
              color: '#2d3748',
              fontFamily: 'Inter, sans-serif'
            }}>
              Create New Account
            </h3>
            <div style={{ marginBottom: '24px' }}>
              <label style={{ 
                display: 'block', 
                marginBottom: '8px', 
                fontSize: '14px', 
                fontWeight: '500',
                color: '#4a5568'
              }}>
                Account Name
              </label>
              <input
                type="text"
                value={newWalletName}
                onChange={(e) => setNewWalletName(e.target.value)}
                placeholder="Enter account name"
                style={{
                  width: '100%',
                  padding: '12px 16px',
                  borderRadius: '8px',
                  border: '1px solid #e2e8f0',
                  fontSize: '16px',
                  fontFamily: 'Inter, sans-serif',
                  outline: 'none',
                  transition: 'border-color 0.2s ease',
                }}
                onFocus={(e) => {
                  e.target.style.borderColor = '#667eea';
                }}
                onBlur={(e) => {
                  e.target.style.borderColor = '#e2e8f0';
                }}
              />
            </div>
            <div style={{ 
              display: 'flex', 
              gap: '12px', 
              justifyContent: 'flex-end',
              marginTop: '32px'
            }}>
              <button
                onClick={() => {
                  setIsModalOpen(false);
                  setNewWalletName('');
                }}
                style={{
                  padding: '12px 24px',
                  backgroundColor: '#e2e8f0',
                  color: '#4a5568',
                  border: 'none',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  fontSize: '16px',
                  fontWeight: '600',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = '#cbd5e0';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = '#e2e8f0';
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleSubmitNewAccount}
                style={buttonStyle}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = '#5a67d8';
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.boxShadow = '0 4px 8px rgba(102, 126, 234, 0.3)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = '#667eea';
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = '0 2px 4px rgba(102, 126, 234, 0.2)';
                }}
              >
                Create Account
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AccountsList;