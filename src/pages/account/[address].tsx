import type { NextPage } from 'next';
import Head from 'next/head';
import { useRouter } from 'next/router';
import styles from '../../styles/Home.module.css';
import Navbar from '../../components/Navbar';
import { useAccount } from 'wagmi';
import { useState, useEffect } from 'react';
import TransferModal from '../../components/TransferModal';
import MessageModal from '../../components/MessageModal';
import AssetTransferModal from '../../components/AssetTransferModal';
import WithdrawModal from '../../components/WithdrawModal';
import { submitTransactionToRPC } from '../../utils/rpcSubmit';
import { MessageRequest } from '../../types';
import DomainTransferModal from '../../components/DomainTransferModal';

import { toast } from 'react-hot-toast';
import { Core } from '@walletconnect/core';
import { IWalletKit, WalletKit } from '@reown/walletkit';

import { buildApprovedNamespaces, getSdkError } from '@walletconnect/utils';
import { ProposalTypes } from '@walletconnect/types';
import { SUPPORTED_CHAINS, SUPPORTED_METHODS, SUPPORTED_EVENTS, CHAIN_NAME_MAP } from '../../constants';
import { hexToString } from 'viem';

// FontAwesome imports
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { 
  faUser, 
  faLink, 
  faMapMarkerAlt, 
  faWallet, 
  faCrown, 
  faCopy,
  faSync,
  faSpinner,
  faPaperPlane
} from '@fortawesome/free-solid-svg-icons';

interface Transaction {
  hash: string;
  type: 'send' | 'receive';
  amount: string;
  timestamp: string;
}

interface Asset {
  symbol: string;
  name: string;
  balance: string;
  value: string;
  icon?: string;
}

interface SignedMessage {
  message: string;
  signer: string;
  domainUrl: string;
  signature: string;
  sessionId: string | null;
  createdAt: string;
}

interface InboxTransaction {
  hash: string;
  blockNumber: string;
  tokenType: 'ETH' | 'ERC20' | 'ERC721' | 'ERC1155';
  amount: string;
  fromAddress: string;
  toAddress: string;
  symbol: string;
  name: string;
  decimals: number;
  contractAddress?: string;
  tokenId?: string;
  createdAt: string;
  claimed: boolean;
}

// Add these helper functions before the component
const getTokenTypeColor = (tokenType: string) => {
  switch (tokenType) {
    case 'ETH':
      return '#627eea';
    case 'ERC20':
      return '#f7931a';
    case 'ERC721':
      return '#9c27b0';
    case 'ERC1155':
      return '#4caf50';
    default:
      return '#666';
  }
};

const formatAmount = (amount: string, decimals: number, symbol: string) => {
  if (decimals === 0) {
    return `${amount} ${symbol}`;
  }
  
  const divisor = Math.pow(10, decimals);
  const formattedAmount = (parseFloat(amount) / divisor).toFixed(6);
  return `${formattedAmount} ${symbol}`;
};

const AccountDetailsPage: NextPage = () => {
  const router = useRouter();
  const { address: accountAddress } = router.query;
  const { isConnected, address: connectedAddress } = useAccount();
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isMessageModalOpen, setIsMessageModalOpen] = useState(false);
  const [isProposalModalOpen, setIsProposalModalOpen] = useState(false);
  const [isDomainTransferModalOpen, setIsDomainTransferModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'assets' | 'messages' | 'inbox' | 'provenance'>('assets');
  const [accountDetails, setAccountDetails] = useState<{
    name: string;
    balance: string;
    owner: string;
    assets: Asset[];
    transactions: Transaction[];
    signedMessages: SignedMessage[];
  } | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [proposalParams, setProposalParams] = useState<ProposalTypes.Struct | null>(null);

  const [walletKit, setWalletKit] = useState<IWalletKit | null>(null);
  const [messageRequest, setMessageRequest] = useState<MessageRequest | null>(null);
  const [activeSessions, setActiveSessions] = useState<any[]>([]);
  const [inboxTransactions, setInboxTransactions] = useState<InboxTransaction[]>([]);
  const [isLoadingInbox, setIsLoadingInbox] = useState(false);
  const [subaccountAssets, setSubaccountAssets] = useState<any>(null);
  const [isLoadingAssets, setIsLoadingAssets] = useState(false);
  const [isAssetTransferModalOpen, setIsAssetTransferModalOpen] = useState(false);
  const [isWithdrawModalOpen, setIsWithdrawModalOpen] = useState(false);
  const [selectedAsset, setSelectedAsset] = useState<{assetId: string, asset: any} | null>(null);
  const [provenanceData, setProvenanceData] = useState<any>(null);
  const [isLoadingProvenance, setIsLoadingProvenance] = useState(false);
  const [submittingTxs, setSubmittingTxs] = useState<Set<string>>(new Set());
  const [claimingTxId, setClaimingTxId] = useState<string | null>(null);



  const handleApproveProposal = async () => {
    console.log("Approve proposal");
    if (!walletKit) {
      toast.error('WalletKit not initialized');
      return;
    }
    console.log(proposalParams);
    const approvedNamespaces = buildApprovedNamespaces({
      proposal: proposalParams as ProposalTypes.Struct,
      supportedNamespaces: {
        eip155: {
          chains: SUPPORTED_CHAINS,
          methods: SUPPORTED_METHODS,
          events: SUPPORTED_EVENTS,
          accounts: [`eip155:11155111:${accountAddress}`, `eip155:1:${accountAddress}`],
        },
      },
    });

    await walletKit.approveSession({
      id: parseInt(proposalParams!.id.toString()),
      namespaces: approvedNamespaces,
    });
    
    // Update the active sessions list
    updateActiveSessions(walletKit);
    toast.success("Session approved");
    setIsProposalModalOpen(false);
  }

  const handleRejectProposal = async () => {
    console.log("Reject proposal");
    await walletKit?.rejectSession({
      id: proposalParams!.id,
      reason: getSdkError("USER_REJECTED")
    });
    toast.success("Reject proposal successful");
    setIsProposalModalOpen(false);
    updateActiveSessions(walletKit);
  }

  const handleApproveSignRequest = async () => {
    console.log("Approve sign request");
    if (!messageRequest?.message || !accountAddress || !walletKit) {
      toast.error("Invalid message or account address");
      setIsMessageModalOpen(false);
      updateActiveSessions(walletKit);
      return;
    }
    // Call backend Sign API
    try {
      const response = await fetch('/api/sign', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: messageRequest.message,
          address: accountAddress,
          signerAddress: connectedAddress,
          domainUrl: messageRequest.dappUrl || 'unknown',
          sessionId: messageRequest.topic || null
        }),
      });
  
      if (!response.ok) {
        throw new Error('Failed to sign message');
      }
  
      const { signature } = await response.json();
  
      if (!messageRequest.topic) {
        toast.error("Error: No topic found");
        return;
      }
      
      // Send signature back to DApp
      await walletKit.respondSessionRequest({
        topic: messageRequest.topic,
        response: {
          id: messageRequest.id,
          result: signature,
          jsonrpc: "2.0",
        },
      });
      
    } catch (error) {
      console.error('Error signing message:', error);
      toast.error("Failed to sign message");
    }
    // OK
    toast.success("Message signed successfully");
    setIsMessageModalOpen(false);
    updateActiveSessions(walletKit);
  }

  const handleRejectRequest = async () => {
    console.log("Reject request");
    if (!messageRequest?.topic) {
      toast.error("Error: No topic found");
      return;
    }
    await walletKit?.respondSessionRequest({
      topic: messageRequest.topic,
      response: {
        id: messageRequest.id,
        error: {
          code: 5000,
          message: "User rejected the request",
        },
        jsonrpc: "2.0",
      },
    });
    toast.success("Reject request successful");
    setIsProposalModalOpen(false);
    updateActiveSessions(walletKit);
  }

  useEffect(() => {
    if (!isConnected) {
      router.push('/');
    }
  }, [isConnected, router]);

  useEffect(() => {
    const fetchAccountDetails = async () => {
      if (!accountAddress) return;
      
      try {
        setIsLoading(true);
        const response = await fetch(`/api/account/${accountAddress}?connectedAddress=${connectedAddress}`);
        const data = await response.json();
        
        if (response.ok) {
          setAccountDetails(data);
        } else {
          console.error('Failed to fetch account details:', data);
        }
      } catch (error) {
        console.error('Error fetching account details:', error);
      } finally {
        setIsLoading(false);
      }
    };
  
    fetchAccountDetails();
  }, [accountAddress, activeSessions]);

  useEffect(() => {
    const initializeWalletKit = async () => {
      try {
        const core = new Core({
          projectId: process.env.NEXT_PUBLIC_PROJECT_ID,
        });

        // Initialize the walletkit
        const walletKit = await WalletKit.init({
          core,
          metadata: {
            name: 'PassWallet',
            description: 'PassWallet',
            url: 'www.example.com',
            icons: []
          }
        });

        setWalletKit(walletKit);
        updateActiveSessions(walletKit);

        // Set up event listeners
        walletKit.on('session_proposal', (proposal) => {
          console.log('Session proposal received:', proposal);
          console.log(proposal.params);
          // toast.success('Session proposal received' + proposal.id);
          setProposalParams(proposal.params);
          setIsProposalModalOpen(true);

          setMessageRequest({
            type: 'proposal',
            message: "",
            dappUrl: proposal.params.proposer.metadata.url,
            id: proposal.id
          });
          updateActiveSessions(walletKit);
        });

        walletKit.on('session_request', (requestEvent) => {
          console.log('Session request received:', requestEvent);
          // toast.success('Session request received');
          console.log("Request event params: " + JSON.stringify(requestEvent.params));

          const message = requestEvent.params.request.params[0];
          const messageString = hexToString(message);
          console.log("Message string: " + messageString);

          // Get URL
          const url = requestEvent.verifyContext?.verified?.origin;
          console.log("URL: " + url);
          
          // TODO: Pass the URL correctly.
          setMessageRequest({
            type: 'request',
            message: messageString,
            dappUrl: url,
            id: requestEvent.id,
            topic: requestEvent.topic
          });
          setIsMessageModalOpen(true);
        });

        walletKit.on('session_delete', (session) => {
          // console.log('Session deleted:', session);
          toast.success('Session deleted');
          updateActiveSessions(walletKit);
        });
      } catch (error) {
        console.error('Failed to initialize WalletKit:', error);
        toast.error('Failed to initialize WalletKit');
      }
    };

    initializeWalletKit();
    // console.log('WalletKit initialized');
    // console.log(walletKit?.getActiveSessions());
  }, []);

  // Update the fetchInboxTransactions function
  useEffect(() => {
    const fetchInboxTransactions = async () => {
      if (!accountAddress) return;
      
      try {
        setIsLoadingInbox(true);
        
        // First, monitor for new transactions
        const monitorResponse = await fetch('/api/assets/monitorInbox', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            wallet: accountAddress,
            fromBlock: '0'
          }),
        });
        
        if (monitorResponse.ok) {
          const monitorData = await monitorResponse.json();
          console.log('Monitor response:', monitorData);
          
          // Now fetch the actual transactions from database
          const transactionsResponse = await fetch(`/api/account/${accountAddress}`);
          if (transactionsResponse.ok) {
            const accountData = await transactionsResponse.json();
            console.log('Account data:', accountData);
            console.log('Connected address:', connectedAddress);
            
            // The account API should return transactions in the expected format
            if (accountData.transactions && Array.isArray(accountData.transactions)) {
              console.log('Raw transactions:', accountData.transactions);
              
              const mappedTransactions: InboxTransaction[] = accountData.transactions.map((tx: any) => {
                const mapped = {
                  hash: tx.hash,
                  blockNumber: tx.blockNumber,
                  tokenType: tx.tokenType || 'ETH',
                  amount: tx.value,
                  fromAddress: tx.from,
                  toAddress: tx.to,
                  symbol: tx.tokenSymbol || 'ETH',
                  name: tx.tokenName || 'Ethereum',
                  decimals: parseInt(tx.tokenDecimal || '18'),
                  contractAddress: tx.contractAddress,
                  tokenId: tx.tokenID,
                  createdAt: new Date().toISOString(),
                  claimed: tx.claimed || false
                };
                console.log('Mapped transaction:', mapped);
                return mapped;
              });
              
              console.log('All mapped transactions:', mappedTransactions);
              console.log('Filtered transactions:', mappedTransactions.filter(tx => tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase()));
              
              setInboxTransactions(mappedTransactions);
            } else {
              console.log('No transactions found or invalid format');
              setInboxTransactions([]);
            }
          } else {
            console.error('Failed to fetch account data');
            setInboxTransactions([]);
          }
        } else {
          console.error('Failed to monitor inbox');
          setInboxTransactions([]);
        }
      } catch (error) {
        console.error('Error monitoring inbox:', error);
        setInboxTransactions([]);
      } finally {
        setIsLoadingInbox(false);
      }
    };

    fetchInboxTransactions();
  }, [accountAddress, connectedAddress]);

  const refreshInboxTransactions = async () => {
    if (!accountAddress) return;
    
    try {
      setIsLoadingInbox(true);
      
      // First, monitor for new transactions
      const monitorResponse = await fetch('/api/assets/monitorInbox', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet: accountAddress,
          fromBlock: '0'
        }),
      });
      
      if (monitorResponse.ok) {
        const monitorData = await monitorResponse.json();
        console.log('Monitor response:', monitorData);
        
        // Now fetch the actual transactions from database
        const transactionsResponse = await fetch(`/api/account/${accountAddress}`);
        if (transactionsResponse.ok) {
          const accountData = await transactionsResponse.json();
          console.log('Account data:', accountData);
          
          // The account API should return transactions in the expected format
          if (accountData.transactions && Array.isArray(accountData.transactions)) {
            const mappedTransactions: InboxTransaction[] = accountData.transactions.map((tx: any) => ({
              hash: tx.hash,
              blockNumber: tx.blockNumber,
              tokenType: tx.tokenType || 'ETH',
              amount: tx.value,
              fromAddress: tx.from,
              toAddress: tx.to,
              symbol: tx.tokenSymbol || 'ETH',
              name: tx.tokenName || 'Ethereum',
              decimals: parseInt(tx.tokenDecimal || '18'),
              contractAddress: tx.contractAddress,
              tokenId: tx.tokenID,
              createdAt: new Date().toISOString(),
              claimed: tx.claimed || false
            }));
            
            setInboxTransactions(mappedTransactions);
            toast.success('Inbox refreshed successfully');
          } else {
            console.log('No transactions found or invalid format');
            setInboxTransactions([]);
            toast.success('No new transactions found');
          }
        } else {
          console.error('Failed to fetch account data');
          setInboxTransactions([]);
          toast.error('Failed to fetch transactions');
        }
      } else {
        console.error('Failed to monitor inbox');
        setInboxTransactions([]);
        toast.error('Failed to monitor inbox');
      }
    } catch (error) {
      console.error('Error refreshing inbox transactions:', error);
      setInboxTransactions([]);
      toast.error('Error refreshing inbox transactions');
    } finally {
      setIsLoadingInbox(false);
    }
  };

  const handleClaim = async (hash: string) => {
    const response = await fetch('/api/assets/claim', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        walletAddress: accountAddress,
        transactionHash: hash,
        claimAddress: connectedAddress
      }),
    });
    if (response.ok) {
      toast.success('Claimed successfully');
    } else {
      const errorData = await response.json();
      toast.error(`Failed to claim: ${errorData.error || 'Unknown error'}`);
    }
  };

  const fetchSubaccountAssets = async () => {
    if (!accountAddress || !connectedAddress) {
      return;
    }
    
    try {
      setIsLoadingAssets(true);
      
      // Call through Next.js API route with connected address for filtering
      const response = await fetch('/api/assets/list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          wallet_address: accountAddress,
          connected_address: connectedAddress 
        }),
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to fetch assets: ${response.status} ${errorText}`);
      }
      
      const assetsData = await response.json();
      console.log('Filtered enclave assets data:', assetsData);
      setSubaccountAssets(assetsData);
    } catch (error: unknown) {
      console.error('Error fetching subaccount assets:', error);
      setSubaccountAssets(null);
      toast.error(`Failed to fetch subaccount assets: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoadingAssets(false);
    }
  };

  // Fetch provenance data for the connected subaccount
  const fetchProvenanceData = async () => {
    if (!accountAddress || !connectedAddress) return;
    
    try {
      setIsLoadingProvenance(true);
      console.log('Fetching provenance data for:', accountAddress);
      
      const response = await fetch('/api/provenance/subaccount', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet_address: accountAddress,
          connected_address: connectedAddress
        }),
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        toast.error(`Failed to fetch provenance data: ${response.status} ${errorText}`);
      }
      
      const provenanceResponse = await response.json();
      console.log('Provenance data:', provenanceResponse);
      setProvenanceData(provenanceResponse);
    } catch (error: unknown) {
      console.error('Error fetching provenance data:', error);
      setProvenanceData(null);
      toast.error(`Failed to fetch provenance data: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsLoadingProvenance(false);
    }
  };

  // Fetch subaccount assets when component loads
  useEffect(() => {
    if (router.isReady && accountAddress && connectedAddress) {
      fetchSubaccountAssets();
    }
  }, [router.isReady, accountAddress, connectedAddress]);

  // Handle transaction submission to RPC
  const handleSubmitTransaction = async (signedRawTransaction: string, recordIndex: number) => {
    const txId = `${accountAddress}-${recordIndex}`;
    
    try {
      setSubmittingTxs(prev => new Set(prev).add(txId));
      
      const result = await submitTransactionToRPC(signedRawTransaction);
      
      if (result.success && result.txHash) {
        toast.success(
          <div>
            <div style={{ fontWeight: 'bold', marginBottom: '8px' }}>
              Transaction Submitted Successfully!
            </div>
            <div style={{ fontFamily: 'monospace', fontSize: '12px', wordBreak: 'break-all' }}>
              <strong>TX Hash:</strong> {result.txHash}
            </div>
            <div style={{ fontSize: '12px', marginTop: '4px' }}>
              <a 
                href={`https://sepolia.etherscan.io/tx/${result.txHash}`} 
                target="_blank" 
                rel="noopener noreferrer"
                style={{ color: '#007bff', textDecoration: 'underline' }}
              >
                View on Sepolia Etherscan →
              </a>
            </div>
          </div>,
          { duration: 8000 }
        );
      } else {
        toast.error(`Failed to submit transaction: ${result.error}`);
      }
    } catch (error: any) {
      console.error('Transaction submission error:', error);
      toast.error(`Transaction submission failed: ${error.message}`);
    } finally {
      setSubmittingTxs(prev => {
        const newSet = new Set(prev);
        newSet.delete(txId);
        return newSet;
      });
    }
  };

  // Fetch provenance data when provenance tab is accessed
  useEffect(() => {
    if (activeTab === 'provenance' && router.isReady && accountAddress && connectedAddress && !provenanceData) {
      fetchProvenanceData();
    }
  }, [activeTab, router.isReady, accountAddress, connectedAddress, provenanceData]);
  

  const cardStyle = {
    padding: '24px',
    border: '1px solid #eaeaea',
    borderRadius: '12px',
    width: '100%',
    maxWidth: '800px',
    backgroundColor: 'white',
    marginBottom: '24px',
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
    marginRight: '12px',
  };

  const tabStyle = {
    padding: '12px 24px',
    backgroundColor: 'transparent',
    color: '#666',
    border: 'none',
    borderBottom: '2px solid transparent',
    cursor: 'pointer',
    fontSize: '16px',
    fontWeight: '500',
    marginRight: '12px',
  };

  const activeTabStyle = {
    ...tabStyle,
    color: '#0d76fc',
    borderBottomColor: '#0d76fc',
  };

  const inputStyle = {
    padding: '12px',
    fontSize: '16px',
    border: '1px solid #eaeaea',
    borderRadius: '8px',
    width: '100%',
    marginBottom: '12px',
  };

  const handleConnect = async () => {
    const uri = (document.getElementById('walletconnect-uri') as HTMLInputElement).value;
    if (!walletKit) {
      toast.error('WalletKit not initialized');
      return;
    }
    await walletKit.pair({ uri });
    toast.success('WalletConnect session established');
    // Log the active sessions
    console.log(
      "WalletConnect session established"
    )
    console.log(walletKit?.getActiveSessions());
  };

  // Convert object to array when setting sessions
  const updateActiveSessions = (walletKit: IWalletKit | null) => {
    // clear input field
    if (document.getElementById('walletconnect-uri') as HTMLInputElement) {
      (document.getElementById('walletconnect-uri') as HTMLInputElement).value = '';
    }
    const sessions = walletKit?.getActiveSessions() || {};
    setActiveSessions(Object.values(sessions));
    // Update signed messages history

  };

  const copyToClipboard = async (text: string, label: string) => {
    try {
      // Try modern clipboard API first
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        toast.success(`${label} copied to clipboard!`);
        return;
      }
      
      // Fallback for older browsers or non-secure contexts
      const textArea = document.createElement('textarea');
      textArea.value = text;
      textArea.style.position = 'fixed';
      textArea.style.left = '-999999px';
      textArea.style.top = '-999999px';
      document.body.appendChild(textArea);
      textArea.focus();
      textArea.select();
      
      const successful = document.execCommand('copy');
      document.body.removeChild(textArea);
      
      if (successful) {
        toast.success(`${label} copied to clipboard!`);
      } else {
        throw new Error('execCommand failed');
      }
    } catch (err) {
      console.error('Failed to copy: ', err);
      
      // As a last resort, show the text in a prompt for manual copying
      try {
        if (window.prompt) {
          window.prompt('Copy to clipboard: Ctrl+C, Enter', text);
        } else {
          toast.error('Copy failed. Please manually select and copy the text.');
        }
      } catch (promptErr) {
        toast.error('Failed to copy to clipboard. Please copy manually.');
      }
    }
  };

  return (
    <div className={styles.container}>
      <Head>
        <title>Account Details - PASS Wallet</title>
        <meta name="description" content="View account details" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Navbar />

      <main className={styles.main}>
        {!isConnected ? (
          <div className={styles.connectContainer}>
            <p>Please connect your wallet to view account details</p>
          </div>
        ) : isLoading ? (
          <div className={styles.loadingContainer}>
            <p>Loading account details...</p>
          </div>
        ) : accountDetails ? (
          <>
            <div style={{ width: '100%', maxWidth: '800px' }}>
              <div style={{ marginBottom: '2rem' }}>
                <h1 className={styles.title}>{accountDetails.name}</h1>
              </div>
              
              <div style={{
                ...cardStyle,
                backgroundColor: 'white',
                position: 'relative',
                overflow: 'hidden'
              }}>
                
                <div style={{ position: 'relative', zIndex: 2 }}>
                <h2 style={{ marginBottom: '20px' }}>Account Overview</h2>
                  
                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', 
                    gap: '20px',
                    marginBottom: '20px'
                  }}>
                  

                    {/* Account Address Card */}
                    <div style={{
                      background: '#f8f9fa',
                      borderRadius: '12px',
                      padding: '20px',
                      border: '1px solid #e9ecef'
                    }}>
                      <div style={{ 
                        display: 'flex', 
                        alignItems: 'center', 
                        marginBottom: '12px',
                        fontSize: '16px',
                        fontWeight: '500',
                        color: '#333'
                      }}>
                        <FontAwesomeIcon 
                          icon={faMapMarkerAlt} 
                          style={{ marginRight: '8px', fontSize: '18px' }} 
                        />
                        Account Address
                      </div>
                      <div style={{ 
                        display: 'flex',
                        alignItems: 'center',
                        background: '#e9ecef',
                        padding: '8px 12px',
                        borderRadius: '6px',
                        gap: '8px'
                      }}>
                        <div style={{ 
                          fontFamily: 'monospace',
                          fontSize: '14px',
                          wordBreak: 'break-all',
                          color: '#495057',
                          flex: 1
                        }}>
                          {accountAddress}
                        </div>
                        <button
                          onClick={() => copyToClipboard(accountAddress as string, 'Account address')}
                          style={{
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            padding: '4px',
                            borderRadius: '4px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '14px',
                            color: '#666',
                            transition: 'color 0.2s'
                          }}
                          onMouseEnter={(e) => e.currentTarget.style.color = '#333'}
                          onMouseLeave={(e) => e.currentTarget.style.color = '#666'}
                        >
                          <FontAwesomeIcon icon={faCopy} />
                        </button>
                      </div>
                    </div>
                     {/* Connected User Card */}
                     <div style={{
                      background: '#f8f9fa',
                      borderRadius: '12px',
                      padding: '20px',
                      border: '1px solid #e9ecef'
                    }}>
                      <div style={{ 
                        display: 'flex', 
                        alignItems: 'center', 
                        marginBottom: '12px',
                        fontSize: '16px',
                        fontWeight: '500',
                        color: '#333'
                      }}>
                        <FontAwesomeIcon 
                          icon={faLink} 
                          style={{ marginRight: '8px', fontSize: '18px' }} 
                        />
                        Connected As
                      </div>
                      <div style={{ 
                        display: 'flex',
                        alignItems: 'center',
                        background: '#e9ecef',
                        padding: '8px 12px',
                        borderRadius: '6px',
                        gap: '8px'
                      }}>
                        <div style={{ 
                          fontFamily: 'monospace',
                          fontSize: '14px',
                          wordBreak: 'break-all',
                          color: '#495057',
                          flex: 1
                        }}>
                          {connectedAddress}
                        </div>
                        <button
                          onClick={() => copyToClipboard(connectedAddress || '', 'Connected address')}
                          style={{
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            padding: '4px',
                            borderRadius: '4px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '14px',
                            color: '#666',
                            transition: 'color 0.2s'
                          }}
                          onMouseEnter={(e) => e.currentTarget.style.color = '#333'}
                          onMouseLeave={(e) => e.currentTarget.style.color = '#666'}
                        >
                          <FontAwesomeIcon icon={faCopy} />
                        </button>
                      </div>
                    </div>
                  </div>

                  <div style={{ 
                    display: 'grid', 
                    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', 
                    gap: '20px'
                  }}>
                    {/* Balance Card */}
                    <div style={{
                      background: '#f8f9fa',
                      borderRadius: '12px',
                      padding: '20px',
                      border: '1px solid #e9ecef',
                      textAlign: 'center'
                    }}>
                      <div style={{ 
                        fontSize: '32px',
                        marginBottom: '8px',
                      }}>
                        <FontAwesomeIcon icon={faWallet} />
                      </div>
                      <div style={{ 
                        fontSize: '14px',
                        color: '#666',
                        marginBottom: '4px'
                      }}>Balance</div>
                      <div style={{ 
                        fontSize: '20px',
                        fontWeight: '600',
                        color: '#333'
                      }}>
                        {(() => {
                          // Show loading state while assets are being fetched
                          if (isLoadingAssets) {
                            return (
                              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px' }}>
                                <FontAwesomeIcon icon={faSpinner} spin style={{ fontSize: '16px' }} />
                                Loading...
                              </div>
                            );
                          }
                          
                          // Get ETH balance from subaccount assets
                          if (subaccountAssets?.assets?.eth) {
                            const ethAsset = subaccountAssets.assets.eth;
                            const ethBalance = formatAmount(
                              ethAsset.total_balance.toString(), 
                              ethAsset.decimals, 
                              ethAsset.symbol
                            );
                            return ethBalance;
                          }
                          return "0.0000 ETH";
                        })()}
                      </div>
                    </div>

                    {/* Owner Card */}
                    <div style={{
                      background: '#f8f9fa',
                      borderRadius: '12px',
                      padding: '20px',
                      border: '1px solid #e9ecef',
                      textAlign: 'center'
                      
                    }}>
                      <div style={{ 
                        fontSize: '32px',
                        marginBottom: '8px',
                        
                      }}>
                        <FontAwesomeIcon icon={faCrown} />
                      </div>
                      <div style={{ 
                        fontSize: '14px',
                        color: '#666',
                        marginBottom: '4px'
                      }}>Owner</div>
                      <div style={{ 
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: '#e9ecef',
                        padding: '6px 8px',
                        borderRadius: '4px',
                        gap: '8px'
                      }}>
                        <div style={{ 
                          fontSize: '14px',
                          fontFamily: 'monospace',
                          wordBreak: 'break-all',
                          color: '#495057',
                          flex: 1,
                          textAlign: 'center'
                        }}>
                          {accountDetails.owner}
                        </div>
                        <button
                          onClick={() => copyToClipboard(accountDetails.owner, 'Owner address')}
                          style={{
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            padding: '2px',
                            borderRadius: '4px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '12px',
                            color: '#666',
                            transition: 'color 0.2s'
                          }}
                          onMouseEnter={(e) => e.currentTarget.style.color = '#333'}
                          onMouseLeave={(e) => e.currentTarget.style.color = '#666'}
                        >
                          <FontAwesomeIcon icon={faCopy} />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div style={cardStyle}>
                <h2 style={{ marginBottom: '20px' }}>Connection Manager</h2>
                
                {/* Connection Input Section */}
                <div style={{ 
                  marginBottom: '24px',
                  padding: '16px',
                  backgroundColor: '#f8f9fa',
                  borderRadius: '8px',
                  border: '1px solid #e9ecef'
                }}>
                  <h3 style={{ 
                    margin: '0 0 12px 0', 
                    fontSize: '16px', 
                    fontWeight: '500',
                    color: '#333'
                  }}>
                    Connect New Session
                  </h3>
                  <div style={{ display: 'flex', gap: '12px', alignItems: 'flex-end' }}>
                    <div style={{ flex: 1 }}>
                      <input 
                        style={{
                          ...inputStyle,
                          marginBottom: '0'
                        }} 
                        type="text" 
                        id="walletconnect-uri" 
                        placeholder="Enter WalletConnect URI" 
                      />
                    </div>
                    <button style={buttonStyle} onClick={handleConnect}>
                      Connect
                    </button>
                  </div>
                </div>

                {/* Active Sessions Section */}
                <div>
                  <h3 style={{ 
                    margin: '0 0 16px 0', 
                    fontSize: '16px', 
                    fontWeight: '500',
                    color: '#333'
                  }}>
                    Active Sessions ({activeSessions.length})
                  </h3>
                  {activeSessions.length === 0 ? (
                    <div style={{
                      padding: '20px',
                      textAlign: 'center',
                      color: '#666',
                      backgroundColor: '#f8f9fa',
                      borderRadius: '8px',
                      border: '1px solid #e9ecef'
                    }}>
                      No active sessions. Connect to a dApp using the URI above.
                    </div>
                  ) : (
                    <div style={{
                      border: '1px solid #e9ecef',
                      borderRadius: '8px',
                      overflow: 'hidden'
                    }}>
                      {activeSessions.map((session, index) => (
                        <div
                          key={session.topic}
                          style={{
                            padding: '16px',
                            borderBottom: index < activeSessions.length - 1 ? '1px solid #eaeaea' : 'none',
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            backgroundColor: 'white'
                          }}
                        >
                          <div>
                            <div style={{ fontWeight: '500', marginBottom: '4px' }}>
                              {session.peer.metadata.name}
                            </div>
                            <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '2px' }}>
                              {session.peer.metadata.url}
                            </div>
                            <div style={{ color: '#666', fontSize: '0.9rem' }}>
                              Chains: {Object.keys(session.namespaces).map(namespace => 
                                session.namespaces[namespace].chains?.map((chain: string) => {
                                  const chainId = chain.split(':')[1];
                                  return CHAIN_NAME_MAP[chainId] || chainId;
                                }).join(', ')
                              ).join(', ')}
                            </div>
                          </div>
                          <button
                            style={{
                              ...buttonStyle,
                              backgroundColor: '#dc3545',
                              padding: '8px 16px'
                            }}
                            onClick={async () => {
                              await walletKit?.disconnectSession({
                                topic: session.topic,
                                reason: getSdkError("USER_DISCONNECTED")
                              });
                              updateActiveSessions(walletKit);
                              toast.success('Session disconnected');
                            }}
                          >
                            Disconnect
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
              {/* Assets Section */}
              <div style={cardStyle}>
                <h2 style={{ marginBottom: '20px', fontSize: '1.5rem', fontWeight: '600' }}>Assets</h2>
                <div style={{ borderBottom: '1px solid #eaeaea', marginBottom: '20px' }}>
                  <button
                    style={activeTab === 'assets' ? activeTabStyle : tabStyle}
                    onClick={() => setActiveTab('assets')}
                  >
                    Subaccount Assets
                  </button>
                  <button
                    style={activeTab === 'provenance' ? activeTabStyle : tabStyle}
                    onClick={() => setActiveTab('provenance')}
                  >
                    Asset Provenance Log
                  </button>
                  <button
                    style={activeTab === 'inbox' ? activeTabStyle : tabStyle}
                    onClick={() => setActiveTab('inbox')}
                  >
                    Inbox Transactions
                  </button>
                </div>

                {activeTab === 'assets' && (
                  <div>
                    <div style={{ marginBottom: '16px', textAlign: 'right' }}>
                      <button
                        style={{
                          ...buttonStyle,
                          backgroundColor: '#28a745',
                          padding: '8px 16px',
                          fontSize: '0.9rem',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '8px',
                          marginLeft: 'auto'
                        }}
                        onClick={fetchSubaccountAssets}
                        disabled={isLoadingAssets}
                      >
                        <FontAwesomeIcon 
                          icon={faSync} 
                          spin={isLoadingAssets}
                          style={{ fontSize: '0.8rem' }}
                        />
                        {isLoadingAssets ? 'Loading...' : 'Refresh Assets'}
                      </button>
                    </div>

                    {isLoadingAssets ? (
                      <p style={{ color: '#666', textAlign: 'center', padding: '20px' }}>Loading assets...</p>
                    ) : !subaccountAssets || !subaccountAssets.assets ? (
                      <p style={{ color: '#666' }}>No assets found in this wallet</p>
                    ) : Object.keys(subaccountAssets.assets).length === 0 ? (
                      <p style={{ color: '#666' }}>No assets registered yet</p>
                    ) : (
                      <div style={{ overflowY: 'auto', maxHeight: '600px' }}>
                        {Object.entries(subaccountAssets.assets).map(([assetId, asset]: [string, any]) => (
                          <div
                            key={assetId}
                            style={{
                              padding: '16px',
                              borderBottom: '1px solid #eaeaea',
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'flex-start'
                            }}
                          >
                            <div style={{ flex: 1 }}>
                              <div style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
                                <span
                                  style={{
                                    backgroundColor: getTokenTypeColor(asset.token_type),
                                    color: 'white',
                                    padding: '4px 8px',
                                    borderRadius: '4px',
                                    fontSize: '0.8rem',
                                    fontWeight: '500',
                                    marginRight: '12px'
                                  }}
                                >
                                  {asset.token_type}
                                </span>
                                <div style={{ fontWeight: '500', fontSize: '1.1rem' }}>
                                  {formatAmount(asset.total_balance.toString(), asset.decimals, asset.symbol)}
                                </div>
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>{asset.name}</strong>
                                {asset.token_id && ` (Token ID: ${asset.token_id})`}
                              </div>
                              
                              {asset.contract_address && (
                                <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                  Contract: <span style={{ fontFamily: 'monospace' }}>{asset.contract_address}</span>
                                </div>
                              )}
                              
                              {/* <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>Subaccount Balances:</strong>
                              </div>
                              {Object.entries(asset.subaccount_balances || {}).map(([subaccountId, balance]: [string, any]) => (
                                <div key={subaccountId} style={{ color: '#666', fontSize: '0.8rem', marginLeft: '16px' }}>
                                  {subaccountId}: {formatAmount(balance.toString(), asset.decimals, asset.symbol)}
                                </div>
                              ))} */}
                            </div>
                            
                            <div style={{ marginLeft: '16px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
                              <button
                                style={{
                                  ...buttonStyle,
                                  backgroundColor: '#007bff',
                                  padding: '6px 12px',
                                  fontSize: '0.8rem',
                                  border: 'none',
                                  cursor: 'pointer',
                                  color: 'white'
                                }}
                                onClick={() => {
                                  setSelectedAsset({ assetId, asset });
                                  setIsAssetTransferModalOpen(true);
                                }}
                              >
                                Transfer
                              </button>
                              <button
                                style={{
                                  ...buttonStyle,
                                  backgroundColor: '#dc3545',
                                  padding: '6px 12px',
                                  fontSize: '0.8rem',
                                  border: 'none',
                                  cursor: 'pointer',
                                  color: 'white'
                                }}
                                onClick={() => {
                                  setSelectedAsset({ assetId, asset });
                                  setIsWithdrawModalOpen(true);
                                }}
                              >
                                Withdraw
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                {activeTab === 'messages' && (
                  <div>
                    {accountDetails.signedMessages.length === 0 ? (
                      <p style={{ color: '#666' }}>No signed messages yet</p>
                    ) : (
                      accountDetails.signedMessages.map((msg, index) => (
                        <div
                          key={index}
                          style={{
                            padding: '16px',
                            borderBottom: index < accountDetails.signedMessages.length - 1 ? '1px solid #eaeaea' : 'none',
                          }}
                        >
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                            <div style={{ fontWeight: '500' }}>{msg.domainUrl}</div>
                            <div style={{ color: '#666', fontSize: '0.9rem' }}>
                              {new Date(msg.createdAt).toLocaleString()}
                            </div>
                          </div>
                          <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '8px' }}>
                            Signer: {msg.signer}
                          </div>
                          <div style={{ 
                            background: '#f5f5f5', 
                            padding: '12px', 
                            borderRadius: '6px',
                            fontSize: '0.9rem',
                            wordBreak: 'break-all'
                          }}>
                            <div style={{ marginBottom: '8px' }}>Message: {msg.message}</div>
                            <div>Signature: {msg.signature.substring(0, 32)}...</div>
                          </div>
                          {msg.sessionId && (
                            <div style={{ color: '#666', fontSize: '0.9rem', marginTop: '8px' }}>
                              Session ID: {msg.sessionId}
                            </div>
                          )}
                        </div>
                      ))
                    )}
                  </div>
                )}

                {activeTab === 'inbox' && (
                  <div>
                    <div style={{ marginBottom: '16px', textAlign: 'right' }}>
                      <button
                        style={{
                          ...buttonStyle,
                          backgroundColor: '#28a745',
                          padding: '8px 16px',
                          fontSize: '0.9rem',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '8px',
                          marginLeft: 'auto'
                        }}
                        onClick={refreshInboxTransactions}
                        disabled={isLoadingInbox}
                      >
                        <FontAwesomeIcon 
                          icon={faSync} 
                          spin={isLoadingInbox}
                          style={{ fontSize: '0.8rem' }}
                        />
                        {isLoadingInbox ? 'Loading...' : 'Refresh Inbox'}
                      </button>
                    </div>

                    {(() => {
                      // Debug logging
                      const filteredTransactions = inboxTransactions.filter(tx => tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase());
                      console.log('UI Debug - All inbox transactions:', inboxTransactions);
                      console.log('UI Debug - Connected address:', connectedAddress);
                      console.log('UI Debug - Filtered transactions:', filteredTransactions);
                      console.log('UI Debug - Filter condition:', inboxTransactions.map(tx => ({
                        fromAddress: tx.fromAddress,
                        connectedAddress: connectedAddress,
                        matches: tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase()
                      })));
                      
                      return null;
                    })()}
                    
                    {isLoadingInbox ? (
                      <p style={{ color: '#666', textAlign: 'center', padding: '20px' }}>Loading transactions...</p>
                    ) : inboxTransactions.filter(tx => tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase()).length === 0 ? (
                      <p style={{ color: '#666' }}>No incoming transactions from your connected wallet yet</p>
                    ) : (
                      <div style={{ overflowY: 'auto', maxHeight: '1200px' }}>
                        {inboxTransactions
                          .filter(tx => tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase())
                          .map((tx, index) => (
                          <div
                            key={tx.hash}
                            style={{
                              padding: '16px',
                              borderBottom: index < inboxTransactions.filter(tx => tx.fromAddress && tx.fromAddress.toLowerCase() === connectedAddress?.toLowerCase()).length - 1 ? '1px solid #eaeaea' : 'none',
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'flex-start'
                            }}
                          >
                            <div style={{ flex: 1 }}>
                              <div style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
                                <span
                                  style={{
                                    backgroundColor: getTokenTypeColor(tx.tokenType),
                                    color: 'white',
                                    padding: '4px 8px',
                                    borderRadius: '4px',
                                    fontSize: '0.8rem',
                                    fontWeight: '500',
                                    marginRight: '12px'
                                  }}
                                >
                                  {tx.tokenType}
                                </span>
                                <div style={{ fontWeight: '500', fontSize: '1.1rem' }}>
                                  {formatAmount(tx.amount, tx.decimals, tx.symbol)}
                                </div>
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>{tx.name}</strong>
                                {tx.tokenId && ` (Token ID: ${tx.tokenId})`}
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                From: <span style={{ fontFamily: 'monospace' }}>{tx.fromAddress}</span>
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                Block: {tx.blockNumber}
                              </div>
                              
                              {tx.contractAddress && (
                                <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                  Contract: <span style={{ fontFamily: 'monospace' }}>{tx.contractAddress}</span>
                                </div>
                              )}
                              
                              <div style={{ color: '#666', fontSize: '0.8rem' }}>
                                Last Updated: {new Date(tx.createdAt).toLocaleString()}
                              </div>
                            </div>
                            
                            <div style={{ marginLeft: '16px', display: 'flex', gap: '8px' }}>
                              <a
                                href={`https://sepolia.etherscan.io/tx/${tx.hash}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                style={{
                                  ...buttonStyle,
                                  backgroundColor: '#6c757d',
                                  padding: '8px 12px',
                                  fontSize: '0.9rem',
                                  textDecoration: 'none',
                                  display: 'inline-block'
                                }}
                              >
                                View on Etherscan
                              </a>
                              <button
                                onClick={() => handleClaim(tx.hash)}
                                disabled={tx.claimed}
                                style={{
                                  ...buttonStyle,
                                  backgroundColor: tx.claimed ? '#6c757d' : '#007bff',
                                  padding: '8px 12px',
                                  fontSize: '0.9rem',
                                  border: 'none',
                                  cursor: tx.claimed ? 'not-allowed' : 'pointer',
                                  color: 'white',
                                  opacity: tx.claimed ? 0.6 : 1
                                }}
                              >
                                {tx.claimed ? 'Claimed' : 'Claim'}
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                {activeTab === 'provenance' && (
                  <div>
                    <div style={{ marginBottom: '16px', textAlign: 'right' }}>
                      <button
                        style={{
                          ...buttonStyle,
                          backgroundColor: '#17a2b8',
                          padding: '8px 16px',
                          fontSize: '0.9rem',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '8px',
                          marginLeft: 'auto'
                        }}
                        onClick={fetchProvenanceData}
                        disabled={isLoadingProvenance}
                      >
                        <FontAwesomeIcon 
                          icon={faSync} 
                          spin={isLoadingProvenance}
                          style={{ fontSize: '0.8rem' }}
                        />
                        {isLoadingProvenance ? 'Loading...' : 'Refresh Provenance'}
                      </button>
                    </div>

                    {isLoadingProvenance ? (
                      <p style={{ color: '#666', textAlign: 'center', padding: '20px' }}>Loading provenance data...</p>
                    ) : !provenanceData || !provenanceData.provenance_records ? (
                      <p style={{ color: '#666' }}>No provenance records found for this subaccount</p>
                    ) : provenanceData.provenance_records.length === 0 ? (
                      <p style={{ color: '#666' }}>No asset transactions recorded yet</p>
                    ) : (
                      <div style={{ overflowY: 'auto', maxHeight: '600px' }}>
                        {provenanceData.provenance_records
                          .sort((a: any, b: any) => b.timestamp - a.timestamp) // Sort by timestamp descending (most recent first)
                          .map((record: any, index: number) => {
                          const operation = record.operation;
                          const timestamp = new Date(record.timestamp * 1000).toLocaleString();
                          
                          // Helper function to get correct decimals for asset
                          const getAssetDecimals = (assetId: string) => {
                            // Default decimals mapping for common assets
                            const defaultDecimals: { [key: string]: number } = {
                              'eth': 18,
                              'usdc': 6,
                              'usdt': 6,
                              'dai': 18,
                              'wbtc': 8
                            };
                            
                            // Try to get decimals from subaccountAssets if available
                            if (subaccountAssets?.assets?.[assetId]?.decimals !== undefined) {
                              return subaccountAssets.assets[assetId].decimals;
                            }
                            
                            // Fall back to default mapping
                            return defaultDecimals[assetId.toLowerCase()] || 18;
                          };

                          // Helper function to get actual subaccount address
                          const getSubaccountAddress = (subaccountId: string) => {
                            // Check if we have the mapping from the API response
                            if (provenanceData.subaccount_mapping && provenanceData.subaccount_mapping[subaccountId]) {
                              return provenanceData.subaccount_mapping[subaccountId];
                            }
                            
                            // Fallback to known connected address
                            if (subaccountId === provenanceData.subaccount_id && provenanceData.subaccount_address) {
                              return provenanceData.subaccount_address;
                            }
                            
                            // Last resort placeholder
                            return `Unknown-${subaccountId}`;
                          };

                          // Helper function to format address for display
                          const formatAddress = (address: string) => {
                            if (address.startsWith('0x') && address.length === 42) {
                              return `${address}`;
                            }
                            return address;
                          };
                          
                          // Determine operation type and details
                          let operationType = '';
                          let operationDetails = '';
                          let operationColor = '#6c757d';
                          
                          if (operation.Claim) {
                            const decimals = getAssetDecimals(operation.Claim.asset_id);
                            operationType = 'Claim';
                            operationDetails = `${formatAmount(operation.Claim.amount.toString(), decimals, operation.Claim.asset_id)}`;
                            operationColor = '#28a745';
                          } else if (operation.Transfer) {
                            const decimals = getAssetDecimals(operation.Transfer.asset_id);
                            const isOutgoing = operation.Transfer.from_subaccount === provenanceData.subaccount_id;
                            operationType = isOutgoing ? 'PASS Transfer Out' : 'PASS Transfer In';
                            operationDetails = `${formatAmount(operation.Transfer.amount.toString(), decimals, operation.Transfer.asset_id)}`;
                            operationColor = isOutgoing ? '#dc3545' : '#007bff';
                          } else if (operation.Withdraw) {
                            const decimals = getAssetDecimals(operation.Withdraw.asset_id);
                            operationType = 'Withdraw';
                            operationDetails = `${formatAmount(operation.Withdraw.amount.toString(), decimals, operation.Withdraw.asset_id)}`;
                            operationColor = '#fd7e14';
                          }

                          return (
                            <div
                              key={index}
                              style={{
                                padding: '16px',
                                borderBottom: '1px solid #eaeaea',
                                display: 'flex',
                                justifyContent: 'space-between',
                                alignItems: 'flex-start'
                              }}
                            >
                              <div style={{ flex: 1 }}>
                                <div style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
                                  <span
                                    style={{
                                      backgroundColor: operationColor,
                                      color: 'white',
                                      padding: '4px 8px',
                                      borderRadius: '4px',
                                      fontSize: '0.8rem',
                                      fontWeight: '500',
                                      marginRight: '12px'
                                    }}
                                  >
                                    {operationType}
                                  </span>
                                  <div style={{ fontWeight: '500', fontSize: '1rem' }}>
                                    {operationDetails.toUpperCase()}
                                  </div>
                                </div>
                                
                                <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                  <strong>Time:</strong> {timestamp}
                                </div>

                                {/* Show transfer destination/source address */}
                                {operation.Transfer && (
                                  <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                    <strong>{operation.Transfer.from_subaccount === provenanceData.subaccount_id ? 'To:' : 'From:'}</strong>{' '}
                                    <span style={{ fontFamily: 'monospace' }}>
                                      {formatAddress(getSubaccountAddress(
                                        operation.Transfer.from_subaccount === provenanceData.subaccount_id 
                                          ? operation.Transfer.to_subaccount 
                                          : operation.Transfer.from_subaccount
                                      ))}
                                    </span>
                                  </div>
                                )}

                                {/* Show withdrawal destination */}
                                {operation.Withdraw && (
                                  <>
                                    <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                      <strong>To:</strong>{' '}
                                      <span style={{ fontFamily: 'monospace' }}>
                                        {formatAddress(operation.Withdraw.destination)}
                                      </span>
                                    </div>
                                    
                                    {/* Show withdrawal nonce */}
                                    {operation.Withdraw.nonce && operation.Withdraw.nonce > 0 && (
                                      <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                        <strong>Nonce:</strong> {operation.Withdraw.nonce}
                                      </div>
                                    )}
                                    
                                    {/* Show gas price and gas limit */}
                                    {operation.Withdraw.gas_price && operation.Withdraw.gas_price > 0 && (
                                      <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                        <strong>Gas Price:</strong> {operation.Withdraw.gas_price} wei ({(operation.Withdraw.gas_price / 1000000000).toFixed(2)} Gwei)
                                      </div>
                                    )}
                                    
                                    {operation.Withdraw.gas_limit && operation.Withdraw.gas_limit > 0 && (
                                      <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                        <strong>Gas Limit:</strong> {operation.Withdraw.gas_limit.toLocaleString()} gas units
                                      </div>
                                    )}
                                    
                                    {/* Show signed raw transaction */}
                                    {operation.Withdraw.signed_raw_transaction && 
                                     operation.Withdraw.signed_raw_transaction !== 'pending' && (
                                      <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                        <strong>Signed Transaction:</strong>
                                        <div style={{ 
                                          fontFamily: 'monospace', 
                                          fontSize: '0.8rem', 
                                          backgroundColor: '#f8f9fa', 
                                          padding: '8px', 
                                          borderRadius: '4px',
                                          marginTop: '4px',
                                          wordBreak: 'break-all',
                                          border: '1px solid #e9ecef',
                                          maxHeight: '80px',
                                          overflow: 'auto'
                                        }}>
                                          {operation.Withdraw.signed_raw_transaction}
                                        </div>
                                        <div style={{ 
                                          display: 'flex', 
                                          justifyContent: 'space-between', 
                                          alignItems: 'center',
                                          marginTop: '8px' 
                                        }}>
                                          <div style={{ fontSize: '0.8rem', color: '#6c757d' }}>
                                            Copy this transaction to submit to any Ethereum RPC endpoint
                                          </div>
                                          <button
                                            onClick={() => handleSubmitTransaction(operation.Withdraw.signed_raw_transaction, index)}
                                            disabled={submittingTxs.has(`${accountAddress}-${index}`)}
                                            style={{
                                              padding: '6px 12px',
                                              backgroundColor: submittingTxs.has(`${accountAddress}-${index}`) ? '#6c757d' : '#28a745',
                                              color: 'white',
                                              border: 'none',
                                              borderRadius: '4px',
                                              fontSize: '0.8rem',
                                              cursor: submittingTxs.has(`${accountAddress}-${index}`) ? 'not-allowed' : 'pointer',
                                              display: 'flex',
                                              alignItems: 'center',
                                              gap: '4px'
                                            }}
                                          >
                                            {submittingTxs.has(`${accountAddress}-${index}`) ? (
                                              <>
                                                <FontAwesomeIcon icon={faSpinner} spin />
                                                Submitting...
                                              </>
                                            ) : (
                                              <>
                                                <FontAwesomeIcon icon={faPaperPlane} />
                                                Submit to Sepolia
                                              </>
                                            )}
                                          </button>
                                        </div>
                                      </div>
                                    )}
                                  </>
                                )}

                                {/* Show claim source */}
                                {operation.Claim && (
                                  <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                    <strong>From Deposit:</strong>{' '}
                                    <span style={{ fontFamily: 'monospace' }}>
                                      {operation.Claim.deposit_id}
                                    </span>
                                  </div>
                                )}
                                
                                {record.block_number && (
                                  <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                    <strong>Block:</strong> {record.block_number}
                                  </div>
                                )}

                                {/* Show asset ID if available */}
                                {(operation.Claim?.asset_id || operation.Transfer?.asset_id || operation.Withdraw?.asset_id) && (
                                  <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                    <strong>Asset:</strong> <span style={{ fontFamily: 'monospace' }}>
                                      {(operation.Claim?.asset_id || operation.Transfer?.asset_id || operation.Withdraw?.asset_id).toUpperCase()}
                                    </span>
                                  </div>
                                )}
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                )}

                {/* {activeTab === 'inbox' && (
                  <div>
                    <div style={{ marginBottom: '16px', textAlign: 'right' }}>
                      <button
                        style={{
                          ...buttonStyle,
                          backgroundColor: '#28a745',
                          padding: '8px 16px',
                          fontSize: '0.9rem',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '8px',
                          marginLeft: 'auto'
                        }}
                        onClick={refreshInboxTransactions}
                        disabled={isLoadingInbox}
                      >
                        <FontAwesomeIcon 
                          icon={faSync} 
                          spin={isLoadingInbox}
                          style={{ fontSize: '0.8rem' }}
                        />
                        {isLoadingInbox ? 'Loading...' : 'Refresh Inbox'}
                      </button>
                    </div>

                    {inboxTransactions.length === 0 ? (
                      <p style={{ color: '#666' }}>No transactions in the inbox yet</p>
                    ) : (
                      <div style={{
                        border: '1px solid #e9ecef',
                        borderRadius: '8px',
                        overflow: 'hidden'
                      }}>
                        {inboxTransactions.map((tx, index) => (
                          <div
                            key={tx.hash}
                            style={{
                              padding: '16px',
                              borderBottom: index < inboxTransactions.length - 1 ? '1px solid #eaeaea' : 'none',
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'center',
                              backgroundColor: 'white'
                            }}
                          >
                            <div style={{ flex: 1 }}>
                              <div style={{ display: 'flex', alignItems: 'center', marginBottom: '8px' }}>
                                <span style={{ 
                                  fontWeight: '500', 
                                  marginRight: '12px',
                                  fontSize: '1rem'
                                }}>
                                  {formatAmount(tx.amount.toString(), tx.decimals, tx.symbol)}
                                </span>
                                <span
                                  style={{
                                    backgroundColor: tx.claimed ? '#28a745' : '#ffc107',
                                    color: tx.claimed ? 'white' : '#212529',
                                    padding: '4px 8px',
                                    borderRadius: '12px',
                                    fontSize: '0.8rem',
                                    fontWeight: '500'
                                  }}
                                >
                                  {tx.claimed ? 'Claimed' : 'Pending'}
                                </span>
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>From:</strong> {tx.fromAddress}
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>Asset:</strong> {tx.name} ({tx.symbol})
                              </div>
                              
                              <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '4px' }}>
                                <strong>Time:</strong> {new Date(tx.createdAt).toLocaleString()}
                              </div>

                              <div style={{ color: '#666', fontSize: '0.8rem', fontFamily: 'monospace' }}>
                                Tx: {tx.hash}
                              </div>
                            </div>

                            {!tx.claimed && (
                              <button
                                style={{
                                  ...buttonStyle,
                                  backgroundColor: '#007bff',
                                  marginLeft: '16px',
                                  padding: '8px 16px'
                                }}
                                onClick={() => handleClaim(tx.hash)}
                                disabled={claimingTxId === tx.hash}
                              >
                                {claimingTxId === tx.hash ? 'Claiming...' : 'Claim'}
                              </button>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div> */}
                {/* } */}
              </div>

              {/* Messages Section */}
              <div style={cardStyle}>
                <div style={{ 
                  display: 'flex', 
                  justifyContent: 'space-between', 
                  alignItems: 'center',
                  marginBottom: '20px'
                }}>
                  <h2 style={{ fontSize: '1.5rem', fontWeight: '600', margin: '0' }}>Messages</h2>
                  <button 
                    style={{...buttonStyle, backgroundColor: '#4CAF50'}}
                    onClick={() => {
                      setIsDomainTransferModalOpen(true);
                    }}
                  >
                    Transfer Domain
                  </button>
                </div>

                <div>
                  <h3 style={{ 
                    marginBottom: '16px', 
                    fontSize: '1.2rem', 
                    fontWeight: '500',
                    color: '#333'
                  }}>
                    Message Signing History
                  </h3>
                  {accountDetails.signedMessages.length === 0 ? (
                    <p style={{ color: '#666' }}>No signed messages yet</p>
                  ) : (
                    <div style={{
                      border: '1px solid #e9ecef',
                      borderRadius: '8px',
                      overflow: 'hidden'
                    }}>
                      {accountDetails.signedMessages.map((msg, index) => (
                        <div
                          key={index}
                          style={{
                            padding: '16px',
                            borderBottom: index < accountDetails.signedMessages.length - 1 ? '1px solid #eaeaea' : 'none',
                            backgroundColor: 'white'
                          }}
                        >
                          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                            <div style={{ fontWeight: '500' }}>{msg.domainUrl}</div>
                            <div style={{ color: '#666', fontSize: '0.9rem' }}>
                              {new Date(msg.createdAt).toLocaleString()}
                            </div>
                          </div>
                          <div style={{ color: '#666', fontSize: '0.9rem', marginBottom: '8px' }}>
                            Signer: {msg.signer}
                          </div>
                          <div style={{ 
                            background: '#f5f5f5', 
                            padding: '12px', 
                            borderRadius: '6px',
                            fontSize: '0.9rem',
                            wordBreak: 'break-all'
                          }}>
                            <div style={{ marginBottom: '8px' }}>Message: {msg.message}</div>
                            <div>Signature: {msg.signature.substring(0, 32)}...</div>
                          </div>
                          {msg.sessionId && (
                            <div style={{ color: '#666', fontSize: '0.9rem', marginTop: '8px' }}>
                              Session ID: {msg.sessionId}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>


            </div>
          </>
        ) : (
          <div className={styles.errorContainer}>
            <p>Failed to load account details</p>
          </div>
        )}
      </main>

      <DomainTransferModal
        isOpen={isDomainTransferModalOpen}
        onClose={() => setIsDomainTransferModalOpen(false)}
        account={accountAddress as string}
        fromAddress={connectedAddress as string}
      />

      <TransferModal
        isOpen={isTransferModalOpen}
        onClose={() => setIsTransferModalOpen(false)}
      />

      <MessageModal
        isOpen={isMessageModalOpen}
        onClose={() => setIsMessageModalOpen(false)}
        messageRequest={messageRequest as MessageRequest}
        onSign={handleApproveSignRequest}
        onReject={handleRejectRequest}
      />
      <MessageModal
        isOpen={isProposalModalOpen}
        onClose={() => setIsProposalModalOpen(false)}
        onSign={handleApproveProposal}
        onReject={handleRejectProposal}
        messageRequest={messageRequest || undefined}
      />

      <AssetTransferModal
        isOpen={isAssetTransferModalOpen}
        onClose={() => setIsAssetTransferModalOpen(false)}
        assetId={selectedAsset?.assetId || ''}
        asset={selectedAsset?.asset || {}}
        walletAddress={accountAddress as string}
        connectedAddress={connectedAddress as string}
        onTransferComplete={() => {
          fetchSubaccountAssets();
          setSelectedAsset(null);
        }}
      />

      <WithdrawModal
        isOpen={isWithdrawModalOpen}
        onClose={() => setIsWithdrawModalOpen(false)}
        assetId={selectedAsset?.assetId || ''}
        asset={selectedAsset?.asset || {}}
        walletAddress={accountAddress as string}
        connectedAddress={connectedAddress as string}
        onWithdrawComplete={() => {
          fetchSubaccountAssets();
          fetchProvenanceData();
          setSelectedAsset(null);
        }}
      />
    </div>
  );
};

export default AccountDetailsPage;