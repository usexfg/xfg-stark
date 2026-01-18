//! STARK Proof Data Schema for XFG → HEAT Burn & COLD Deposit
//!
//! This module defines the data structures needed for STARK proof generation,
//! with JSON serialization for easy CLI tool integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Proof type: HEAT burn or COLD deposit
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofType {
    /// XFG burn → HEAT mint
    HEAT,
    /// XFG deposit → CD interest mint
    COLD,
}

/// Complete data package for STARK proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkProofDataPackage {
    /// Proof type (HEAT burn or COLD deposit)
    pub proof_type: ProofType,
    /// Metadata about the proof request
    pub metadata: ProofMetadata,
    /// Burn/Deposit transaction details
    pub burn_transaction: BurnTransaction,
    /// Recipient information
    pub recipient: RecipientInfo,
    /// User's secret for proof generation
    pub secret: SecretInfo,
    /// Optional additional data
    #[serde(default)]
    pub additional_data: HashMap<String, String>,
}

/// Enhanced data package that includes both STARK proof and Eldernode verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteProofPackage {
    /// The original STARK proof data package
    pub stark_proof_data: StarkProofDataPackage,
    /// Generated STARK proof (if available)
    #[serde(default)]
    pub stark_proof: Option<StarkProof>,
    /// Eldernode verification proof (if available)
    #[serde(default)]
    pub eldernode_verification: Option<EldernodeVerification>,
    /// Package status
    pub status: PackageStatus,
    /// Timestamps for tracking
    pub timestamps: ProofTimestamps,
}

/// STARK proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkProof {
    /// Proof data in bytes
    pub proof_data: Vec<u8>,
    /// Public inputs used for verification
    pub public_inputs: StarkPublicInputs,
    /// Proof metadata
    pub metadata: ProofMetadata,
}

/// STARK proof public inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkPublicInputs {
    /// Burn amount in atomic units
    pub burn_amount: u64,
    /// Mint amount in atomic units
    pub mint_amount: u64,
    /// Transaction hash
    pub txn_hash: String,
    /// Recipient hash
    pub recipient_hash: String,
    /// State
    pub state: u32,
}

/// Eldernode verification proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EldernodeVerification {
    /// Merkle proof data
    pub merkle_proof: MerkleProof,
    /// Eldernode signatures
    pub eldernode_signatures: Vec<EldernodeSignature>,
    /// Consensus information
    pub consensus: ConsensusInfo,
    /// Verification metadata
    pub metadata: VerificationMetadata,
}

/// Merkle proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Root hash
    pub root_hash: String,
    /// Leaf hash
    pub leaf_hash: String,
    /// Proof path
    pub proof_path: Vec<String>,
    /// Proof indices
    pub proof_indices: Vec<u32>,
}

/// Eldernode signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EldernodeSignature {
    /// Eldernode public key
    pub public_key: String,
    /// Eldernode address
    pub eldernode_address: String,
    /// Signature data
    pub signature: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Consensus information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusInfo {
    /// Number of Eldernodes that verified
    pub eldernode_count: u32,
    /// Consensus threshold met
    pub threshold_met: bool,
    /// Consensus type (2/2 or 3/5)
    pub consensus_type: String,
}

/// Verification metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMetadata {
    /// Verification timestamp
    pub verified_at: String,
    /// Network where verification occurred
    pub network: String,
    /// Version of verification protocol
    pub version: String,
}

/// Package status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageStatus {
    /// Data package created, ready for STARK proof generation
    DataReady,
    /// STARK proof generated, ready for Eldernode verification
    StarkProofReady,
    /// Eldernode verification complete, ready for contract submission
    Complete,
    /// Error occurred during processing
    Error(String),
}

/// Proof timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTimestamps {
    /// When data package was created
    pub created_at: String,
    /// When STARK proof was generated
    #[serde(default)]
    pub stark_proof_generated: Option<String>,
    /// When Eldernode verification was completed
    #[serde(default)]
    pub eldernode_verified: Option<String>,
}

/// Metadata about the proof request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Version of the data package format
    pub version: String,
    /// Timestamp when package was created
    pub created_at: String,
    /// Description of the proof request
    pub description: String,
    /// Network identifier (e.g., "fuego-mainnet", "fuego-testnet")
    pub network: String,
}

/// Burn transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnTransaction {
    /// Transaction hash (hex string)
    pub transaction_hash: String,
    /// Burn amount in XFG (decimal, e.g., "0.8" or "800.0")
    pub burn_amount_xfg: String,
    /// Burn amount in atomic units (integer)
    pub burn_amount_atomic: u64,
    /// Block height where burn occurred
    pub block_height: u64,
    /// Timestamp of burn transaction
    pub timestamp: u64,
    /// Fuego network ID
    pub network_id: String,
}

/// Recipient information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientInfo {
    /// Ethereum address (0x-prefixed hex)
    pub ethereum_address: String,
    /// Optional ENS name
    #[serde(default)]
    pub ens_name: Option<String>,
    /// Optional label for the recipient
    #[serde(default)]
    pub label: Option<String>,
}

/// Secret information for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    /// User's secret key (hex string)
    pub secret_key: String,
    /// Optional salt for additional security
    #[serde(default)]
    pub salt: Option<String>,
    /// Optional hint for secret recovery
    #[serde(default)]
    pub hint: Option<String>,
}

/// Validation result for data package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the package is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
}

impl StarkProofDataPackage {
    /// Create a new data package
    pub fn new(
        burn_amount_xfg: f64,
        transaction_hash: String,
        ethereum_address: String,
        secret_key: String,
        network: String,
    ) -> Self {
        let burn_amount_atomic = Self::xfg_to_atomic_units(burn_amount_xfg);
        
        let network_clone = network.clone();
        Self {
            metadata: ProofMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                description: format!("STARK proof for {} XFG burn", burn_amount_xfg),
                network: network_clone,
            },
            burn_transaction: BurnTransaction {
                transaction_hash,
                burn_amount_xfg: burn_amount_xfg.to_string(),
                burn_amount_atomic,
                block_height: 0, // Will be filled by user
                timestamp: 0,    // Will be filled by user
                network_id: network,
            },
            recipient: RecipientInfo {
                ethereum_address,
                ens_name: None,
                label: None,
            },
            secret: SecretInfo {
                secret_key,
                salt: None,
                hint: None,
            },
            additional_data: HashMap::new(),
        }
    }

    /// Convert XFG amount to atomic units
    pub fn xfg_to_atomic_units(xfg_amount: f64) -> u64 {
        (xfg_amount * 10_000_000.0) as u64
    }

    /// Convert atomic units to XFG amount
    pub fn atomic_units_to_xfg(atomic_units: u64) -> f64 {
        atomic_units as f64 / 10_000_000.0
    }

    /// Validate the data package
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate burn/deposit amount (4 tiers)
        let valid_amounts = [0.8, 8.0, 80.0, 800.0];
        let burn_amount = self.burn_transaction.burn_amount_xfg.parse::<f64>().unwrap_or(0.0);
        if !valid_amounts.contains(&burn_amount) {
            errors.push(format!(
                "Burn/deposit amount must be exactly 0.8, 8, 80, or 800 XFG, got {}",
                burn_amount
            ));
        }

        // Validate transaction hash format (Fuego format: no 0x prefix)
        if self.burn_transaction.transaction_hash.starts_with("0x") {
            errors.push("Fuego transaction hash should not start with 0x".to_string());
        }

        // Validate Ethereum address format
        if !self.recipient.ethereum_address.starts_with("0x") 
           || self.recipient.ethereum_address.len() != 42 {
            errors.push("Ethereum address must be 0x-prefixed 40-character hex".to_string());
        }

        // Validate secret key
        if self.secret.secret_key.len() < 8 {
            errors.push("Secret key must be at least 8 characters".to_string());
        }

        // Warnings
        if self.burn_transaction.block_height == 0 {
            warnings.push("Block height is 0 - please verify this is correct".to_string());
        }

        if self.burn_transaction.timestamp == 0 {
            warnings.push("Timestamp is 0 - please verify this is correct".to_string());
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Save package to JSON file
    pub fn save_to_file(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filepath, json)?;
        Ok(())
    }

    /// Load package from JSON file
    pub fn load_from_file(filepath: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filepath)?;
        let package: StarkProofDataPackage = serde_json::from_str(&json)?;
        Ok(package)
    }

    /// Get mint amount (1:1 ratio with burn)
    pub fn get_mint_amount_atomic(&self) -> u64 {
        self.burn_transaction.burn_amount_atomic
    }

    /// Get mint amount in HEAT
    pub fn get_mint_amount_heat(&self) -> f64 {
        Self::atomic_units_to_xfg(self.burn_transaction.burn_amount_atomic)
    }
}

impl CompleteProofPackage {
    /// Create a new complete proof package
    pub fn new(stark_proof_data: StarkProofDataPackage) -> Self {
        Self {
            stark_proof_data: stark_proof_data.clone(),
            stark_proof: None,
            eldernode_verification: None,
            status: PackageStatus::DataReady,
            timestamps: ProofTimestamps {
                created_at: stark_proof_data.metadata.created_at.clone(),
                stark_proof_generated: None,
                eldernode_verified: None,
            },
        }
    }

    /// Add STARK proof to the package
    pub fn add_stark_proof(&mut self, stark_proof: StarkProof) {
        self.stark_proof = Some(stark_proof);
        self.timestamps.stark_proof_generated = Some(chrono::Utc::now().to_rfc3339());
        self.status = PackageStatus::StarkProofReady;
    }

    /// Add Eldernode verification to the package
    pub fn add_eldernode_verification(&mut self, eldernode_verification: EldernodeVerification) {
        self.eldernode_verification = Some(eldernode_verification);
        self.timestamps.eldernode_verified = Some(chrono::Utc::now().to_rfc3339());
        self.status = PackageStatus::Complete;
    }

    /// Check if package is ready for contract submission
    pub fn is_ready_for_contract(&self) -> bool {
        matches!(self.status, PackageStatus::Complete)
    }

    /// Save complete package to JSON file
    pub fn save_to_file(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filepath, json)?;
        Ok(())
    }

    /// Load complete package from JSON file
    pub fn load_from_file(filepath: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filepath)?;
        let package: CompleteProofPackage = serde_json::from_str(&json)?;
        Ok(package)
    }

    /// Get contract submission data
    pub fn get_contract_submission_data(&self) -> Option<ContractSubmissionData> {
        if !self.is_ready_for_contract() {
            return None;
        }

        Some(ContractSubmissionData {
            stark_proof: self.stark_proof.as_ref().unwrap().clone(),
            eldernode_verification: self.eldernode_verification.as_ref().unwrap().clone(),
            burn_data: self.stark_proof_data.clone(),
        })
    }

    /// Get package status
    pub fn get_status(&self) -> PackageStatus {
        self.status.clone()
    }

    /// Get STARK proof if present
    pub fn get_stark_proof(&self) -> Option<&StarkProof> {
        self.stark_proof.as_ref()
    }

    /// Get Eldernode verification if present
    pub fn get_eldernode_verification(&self) -> Option<&EldernodeVerification> {
        self.eldernode_verification.as_ref()
    }

    /// Export contract data
    pub fn export_contract_data(&self) -> Result<ContractSubmissionData, Box<dyn std::error::Error>> {
        if !self.is_ready_for_contract() {
            return Err("Package not ready for contract submission".into());
        }

        Ok(ContractSubmissionData {
            stark_proof: self.stark_proof.as_ref().unwrap().clone(),
            eldernode_verification: self.eldernode_verification.as_ref().unwrap().clone(),
            burn_data: self.stark_proof_data.clone(),
        })
    }
}
/// Data ready for contract submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSubmissionData {
    /// STARK proof
    pub stark_proof: StarkProof,
    /// Eldernode verification
    pub eldernode_verification: EldernodeVerification,
    /// Burn data
    pub burn_data: StarkProofDataPackage,
}

/// Template for creating data packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDataTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Default values
    pub defaults: HashMap<String, String>,
    /// Required fields
    pub required_fields: Vec<String>,
    /// Optional fields
    pub optional_fields: Vec<String>,
}

impl ProofDataTemplate {
    /// Create standard burn template
    pub fn standard_burn() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert("burn_amount_xfg".to_string(), "0.8".to_string());
        defaults.insert("network".to_string(), "fuego-mainnet".to_string());
        defaults.insert("version".to_string(), "1.0.0".to_string());

        Self {
            name: "Standard Burn (0.8 XFG)".to_string(),
            description: "Template for standard 0.8 XFG burn".to_string(),
            defaults,
            required_fields: vec![
                "transaction_hash".to_string(),
                "ethereum_address".to_string(),
                "secret_key".to_string(),
            ],
            optional_fields: vec![
                "block_height".to_string(),
                "timestamp".to_string(),
                "ens_name".to_string(),
                "label".to_string(),
                "salt".to_string(),
                "hint".to_string(),
            ],
        }
    }

    /// Create large burn template
    pub fn large_burn() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert("burn_amount_xfg".to_string(), "800.0".to_string());
        defaults.insert("network".to_string(), "fuego-mainnet".to_string());
        defaults.insert("version".to_string(), "1.0.0".to_string());

        Self {
            name: "Large Burn (800 XFG)".to_string(),
            description: "Template for large 800 XFG burn".to_string(),
            defaults,
            required_fields: vec![
                "transaction_hash".to_string(),
                "ethereum_address".to_string(),
                "secret_key".to_string(),
            ],
            optional_fields: vec![
                "block_height".to_string(),
                "timestamp".to_string(),
                "ens_name".to_string(),
                "label".to_string(),
                "salt".to_string(),
                "hint".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_creation() {
        let package = StarkProofDataPackage::new(
            0.8,
            "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            "my-secret-key-123".to_string(),
            "fuego-testnet".to_string(),
        );

        assert_eq!(package.burn_transaction.burn_amount_atomic, 8_000_000);
        assert_eq!(package.get_mint_amount_atomic(), 8_000_000);
        assert_eq!(package.get_mint_amount_heat(), 0.8);
    }

    #[test]
    fn test_validation() {
        let package = StarkProofDataPackage::new(
            0.8,
            "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            "my-secret-key-123".to_string(),
            "fuego-testnet".to_string(),
        );

        let validation = package.validate();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_invalid_amount() {
        let package = StarkProofDataPackage::new(
            1.5, // Invalid amount
            "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            "my-secret-key-123".to_string(),
            "fuego-testnet".to_string(),
        );

        let validation = package.validate();
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("Burn amount")));
    }

    #[test]
    fn test_complete_package_workflow() {
        let stark_data = StarkProofDataPackage::new(
            0.8,
            "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
            "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
            "my-secret-key-123".to_string(),
            "fuego-testnet".to_string(),
        );

        let mut complete_package = CompleteProofPackage::new(stark_data);
        assert!(matches!(complete_package.status, PackageStatus::DataReady));

        // Add STARK proof
        let stark_proof = StarkProof {
            proof_data: vec![1, 2, 3, 4],
            public_inputs: StarkPublicInputs {
                burn_amount: 8_000_000,
                mint_amount: 8_000_000,
                txn_hash: "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
                recipient_hash: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
                state: 0,
            },
            metadata: ProofMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                description: "Test proof".to_string(),
                network: "fuego-testnet".to_string(),
            },
        };

        complete_package.add_stark_proof(stark_proof);
        assert!(matches!(complete_package.status, PackageStatus::StarkProofReady));

        // Add Eldernode verification
        let eldernode_verification = EldernodeVerification {
            merkle_proof: MerkleProof {
                root_hash: "root123".to_string(),
                leaf_hash: "leaf123".to_string(),
                proof_path: vec!["path1".to_string(), "path2".to_string()],
                proof_indices: vec![0, 1],
            },
            eldernode_signatures: vec![EldernodeSignature {
                public_key: "pubkey123".to_string(),
                eldernode_address: "fireTgbnrda4nMJF56jAg5ifD1dgmQMSR6HaY8exk9sZQFrDtzyHn2CN6hgkNNg6deT66AhDuA2MJT81yMWAk61w9m8xmurg2z".to_string(),
                signature: "sig123".to_string(),
                timestamp: 1705312200,
            }],
            consensus: ConsensusInfo {
                eldernode_count: 1,
                threshold_met: true,
                consensus_type: "2/2".to_string(),
            },
            metadata: VerificationMetadata {
                verified_at: chrono::Utc::now().to_rfc3339(),
                network: "fuego-testnet".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        complete_package.add_eldernode_verification(eldernode_verification);
        assert!(matches!(complete_package.status, PackageStatus::Complete));
        assert!(complete_package.is_ready_for_contract());
    }
}

impl StarkProof {
    /// Create a dummy STARK proof for testing
    pub fn new_dummy() -> Self {
        StarkProof {
            proof_data: vec![0u8; 32], // Dummy proof data
            public_inputs: StarkPublicInputs {
                burn_amount: 8_000_000, // 0.8 XFG in atomic units
                mint_amount: 8_000_000, // 1:1 ratio
                txn_hash: "7D0725F8E03021B99560ADD456C596FEA7D8DF23529E23765E56923B73236E4D".to_string(),
                recipient_hash: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
                state: 0,
            },
            metadata: ProofMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                description: "Dummy STARK proof for testing".to_string(),
                network: "fuego-mainnet".to_string(),
            },
        }
    }
}

impl EldernodeVerification {
    /// Create a dummy Eldernode verification for testing
    pub fn new_dummy() -> Self {
        EldernodeVerification {
            merkle_proof: MerkleProof {
                root_hash: "0x1234567890abcdef".to_string(),
                leaf_hash: "0xfedcba0987654321".to_string(),
                proof_path: vec!["0xabc123".to_string(), "0xdef456".to_string()],
                proof_indices: vec![0, 1],
            },
            eldernode_signatures: vec![],
            consensus: ConsensusInfo {
                eldernode_count: 1,
                threshold_met: true,
                consensus_type: "dummy".to_string(),
            },
            metadata: VerificationMetadata {
                verified_at: chrono::Utc::now().to_rfc3339(),
                network: "fuego-mainnet".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }
}
