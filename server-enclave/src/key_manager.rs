use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::{anyhow, Result};
use hex;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    PublicKey, SecretKey,
};
use rand::RngCore;
use rlp::RlpStream;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct EncryptedKey {
    ciphertext: String,
    nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct EthereumAccount {
    pub address: String,
    pub private_key: String,
}

#[derive(Clone)]
pub struct EnclaveKMS {
    secret: [u8; 32],
    keystore: Arc<Mutex<HashMap<String, EncryptedKey>>>,
}

impl EnclaveKMS {
    pub fn new(secret: &str) -> Result<Self> {
        let mut secret_bytes = [0u8; 32];
        let secret_hash = Keccak256::digest(secret.as_bytes());
        secret_bytes.copy_from_slice(&secret_hash);

        Ok(EnclaveKMS {
            secret: secret_bytes,
            keystore: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn generate_ethereum_account(&self) -> Result<EthereumAccount> {
        let mut rng = rand::thread_rng();
        let mut private_key_bytes = [0u8; 32];
        rng.fill_bytes(&mut private_key_bytes);

        let secret_key = SecretKey::from_bytes(&private_key_bytes.into())?;
        let public_key = secret_key.public_key();

        let address = self.public_key_to_address(&public_key);
        let private_key = format!("0x{}", hex::encode(private_key_bytes));

        Ok(EthereumAccount {
            address,
            private_key,
        })
    }

    fn public_key_to_address(&self, public_key: &PublicKey) -> String {
        let public_key_bytes = public_key.to_encoded_point(false);
        let public_key_slice = public_key_bytes.as_bytes();
        // Skip the first byte (0x04) for uncompressed format
        let hash = Keccak256::digest(&public_key_slice[1..]);
        // Take the last 20 bytes and format as hex with 0x prefix
        format!("0x{}", hex::encode(&hash[12..]))
    }

    fn encrypt_key(&self, private_key: &str) -> Result<EncryptedKey> {
        let key = Key::<Aes256Gcm>::from_slice(&self.secret);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let private_key_clean = private_key.strip_prefix("0x").unwrap_or(private_key);
        let private_key_bytes = hex::decode(private_key_clean)?;

        let ciphertext = cipher
            .encrypt(nonce, private_key_bytes.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedKey {
            ciphertext: hex::encode(ciphertext),
            nonce: hex::encode(nonce_bytes),
        })
    }

    fn decrypt_key(&self, encrypted_key: &EncryptedKey) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.secret);
        let cipher = Aes256Gcm::new(key);

        let nonce_bytes = hex::decode(&encrypted_key.nonce)?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = hex::decode(&encrypted_key.ciphertext)?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(format!("0x{}", hex::encode(plaintext)))
    }

    fn store_key(&mut self, address: &str, encrypted_key: &EncryptedKey) -> Result<()> {
        self.keystore
            .lock()
            .unwrap()
            .insert(address.to_string(), encrypted_key.clone());
        Ok(())
    }

    fn get_key(&self, address: &str) -> Result<Option<EncryptedKey>> {
        Ok(self.keystore.lock().unwrap().get(address).cloned())
    }

    pub fn list_addresses(&self) -> Result<Vec<String>> {
        Ok(self.keystore.lock().unwrap().keys().cloned().collect())
    }

    pub fn sign_message(&self, message: &str, address: &str) -> Result<Option<String>> {
        let encrypted_key = match self.get_key(address)? {
            Some(key) => key,
            None => return Ok(None),
        };

        let private_key_hex = match self.decrypt_key(&encrypted_key) {
            Ok(key) => key,
            Err(_) => return Ok(None),
        };

        let private_key_clean = private_key_hex
            .strip_prefix("0x")
            .unwrap_or(&private_key_hex);
        let private_key_bytes = hex::decode(private_key_clean)?;

        // Convert Vec<u8> to [u8; 32] array
        let private_key_array: [u8; 32] = private_key_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid private key length"))?;

        let secret_key = SecretKey::from_bytes(&private_key_array.into())?;
        let signing_key = SigningKey::from(secret_key);

        // Create EIP-191 message hash
        let message_hash = self.hash_message(message);

        let (signature, recovery_id) = signing_key.sign_prehash_recoverable(&message_hash)?;

        // Convert to Ethereum signature format (r, s, v)
        let signature_bytes = signature.to_bytes();
        let mut eth_signature = [0u8; 65];
        eth_signature[..64].copy_from_slice(&signature_bytes);
        eth_signature[64] = recovery_id.to_byte() + 27; // Ethereum v value

        Ok(Some(format!("0x{}", hex::encode(eth_signature))))
    }

    fn hash_message(&self, message: &str) -> [u8; 32] {
        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut hasher = Keccak256::new();
        hasher.update(prefix.as_bytes());
        hasher.update(message.as_bytes());
        hasher.finalize().into()
    }

    pub fn verify_message(&self, message: &str, signature: &str, address: &str) -> Result<bool> {
        let signature_clean = signature.strip_prefix("0x").unwrap_or(signature);
        let signature_bytes = hex::decode(signature_clean)?;

        if signature_bytes.len() != 65 {
            return Ok(false);
        }

        let recovery_id = RecoveryId::from_byte(signature_bytes[64] - 27)
            .ok_or_else(|| anyhow!("Invalid recovery ID"))?;

        // Convert &[u8] to [u8; 64] array
        let signature_array: [u8; 64] = signature_bytes[..64]
            .try_into()
            .map_err(|_| anyhow!("Invalid signature length"))?;

        let signature = Signature::from_bytes(&signature_array.into())?;

        let message_hash = self.hash_message(message);

        let recovered_key =
            VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id)?;
        let recovered_pubkey = PublicKey::from(&recovered_key);
        let recovered_address = self.public_key_to_address(&recovered_pubkey);

        Ok(recovered_address.to_lowercase() == address.to_lowercase())
    }

    pub fn handle_keygen(&mut self) -> Result<EthereumAccount> {
        let account = self.generate_ethereum_account()?;

        // Encrypt and store the key
        let encrypted_key = self.encrypt_key(&account.private_key)?;
        self.store_key(&account.address, &encrypted_key)?;

        Ok(account)
    }

    /// Sign an Ethereum transaction using the stored private key
    pub fn sign_transaction(
        &mut self,
        wallet_address: &str,
        tx: &LegacyTransaction,
        chain_id: u64,
    ) -> Result<String> {
        // Retrieve and decrypt the private key
        let encrypted_key = self
            .get_key(wallet_address)?
            .ok_or_else(|| anyhow!("Key not found for wallet"))?;
        let private_key_hex = self.decrypt_key(&encrypted_key)?;

        // Parse private key
        let private_key_clean = private_key_hex
            .strip_prefix("0x")
            .unwrap_or(&private_key_hex);
        let private_key_bytes = hex::decode(private_key_clean)?;
        if private_key_bytes.len() != 32 {
            return Err(anyhow!("Invalid private key length"));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&private_key_bytes);
        let secret_key = SecretKey::from_bytes(&key_array.into())?;
        let signing_key = SigningKey::from(secret_key);

        // Sign the transaction
        let tx_hash = self.compute_transaction_hash(tx, chain_id)?;
        let (signature, recovery_id) = signing_key.sign_prehash_recoverable(&tx_hash)?;

        // Helper function to convert minimal big-endian bytes back to u64
        let bytes_to_u64 = |bytes: &[u8]| -> u64 {
            if bytes.is_empty() {
                return 0;
            }
            let mut result = 0u64;
            for &byte in bytes {
                result = (result << 8) | (byte as u64);
            }
            result
        };

        // Encode the signed transaction using RLP
        let mut rlp_stream = RlpStream::new_list(9);
        rlp_stream.append(&tx.nonce);
        rlp_stream.append(&bytes_to_u64(&tx.gas_price));
        rlp_stream.append(&bytes_to_u64(&tx.gas_limit));

        // Handle the 'to' field properly
        if let Some(to_addr) = &tx.to {
            rlp_stream.append(to_addr);
        } else {
            rlp_stream.append(&""); // Empty for contract creation
        }

        rlp_stream.append(&bytes_to_u64(&tx.value));
        rlp_stream.append(&tx.data);

        // Add signature components with EIP-155
        let v = recovery_id.to_byte() as u64 + 35 + 2 * chain_id;
        rlp_stream.append(&v);

        // Convert signature components to bytes
        let r_bytes = signature.r().to_bytes();
        let s_bytes = signature.s().to_bytes();
        rlp_stream.append(&r_bytes.as_slice());
        rlp_stream.append(&s_bytes.as_slice());

        let encoded = rlp_stream.out();
        Ok(format!("0x{}", hex::encode(encoded)))
    }

    /// Compute transaction hash for signing (EIP-155)
    fn compute_transaction_hash(&self, tx: &LegacyTransaction, chain_id: u64) -> Result<[u8; 32]> {
        // Helper function to convert minimal big-endian bytes back to u64
        let bytes_to_u64 = |bytes: &[u8]| -> u64 {
            if bytes.is_empty() {
                return 0;
            }
            let mut result = 0u64;
            for &byte in bytes {
                result = (result << 8) | (byte as u64);
            }
            result
        };

        let mut rlp_stream = RlpStream::new_list(9);
        rlp_stream.append(&tx.nonce);
        rlp_stream.append(&bytes_to_u64(&tx.gas_price));
        rlp_stream.append(&bytes_to_u64(&tx.gas_limit));

        if let Some(to_addr) = &tx.to {
            rlp_stream.append(to_addr);
        } else {
            rlp_stream.append(&"");
        }

        rlp_stream.append(&bytes_to_u64(&tx.value));
        rlp_stream.append(&tx.data);
        rlp_stream.append(&chain_id);
        rlp_stream.append(&0u8); // Empty r for EIP-155
        rlp_stream.append(&0u8); // Empty s for EIP-155

        let encoded = rlp_stream.out();
        let hash = Keccak256::digest(&encoded);
        Ok(hash.into())
    }
}

/// Legacy Ethereum transaction structure
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub nonce: u64,
    pub gas_price: Vec<u8>,  // Big-endian bytes
    pub gas_limit: Vec<u8>,  // Big-endian bytes
    pub to: Option<Vec<u8>>, // 20-byte address
    pub value: Vec<u8>,      // Big-endian bytes
    pub data: Vec<u8>,
}
