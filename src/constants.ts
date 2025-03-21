import { mainnet, sepolia } from "viem/chains";

export const CHAIN_NAME_MAP: { [key: string]: string } = {
  '1': mainnet.name,        // "Ethereum"
  '11155111': sepolia.name, // "Sepolia"
  // Add more chains as needed
};

export const SUPPORTED_CHAINS = [
    "eip155:11155111", // Sepolia testnet
    "eip155:1", // Ethereum mainnet
  ];
  
export const SUPPORTED_METHODS = [
    "personal_sign",
    "eth_sendTransaction"
  ];
  
export const SUPPORTED_EVENTS = [
    "chainChanged",
    "accountsChanged",
    "sessionRequest",
    "sessionProposal",
  ];