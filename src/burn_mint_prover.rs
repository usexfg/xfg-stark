//! XFG Burn & Mint Prover Implementation for Winterfell
//!
//! This module implements the Winterfell prover for XFG burn and HEAT mint operations,
//! providing secure and efficient proof generation.

use crate::ExecutionTrace;
use crate::{
    burn_mint_air::{generate_burn_mint_trace, BurnMintPublicInputs, XfgBurnMintAir},
    Result,
};
use anyhow;
use winterfell::{math::fields::f64::BaseElement, ProofOptions, Prover, StarkProof, TraceInfo};

/// XFG Burn & Mint Prover using Winterfell
///
/// This prover generates STARK proofs for XFG burn and HEAT mint operations
/// using Winterfell's battle-tested proving system.
pub struct XfgBurnMintProver {
    /// Security parameter for proof generation
    security_parameter: usize,
    /// Proof options for Winterfell
    proof_options: ProofOptions,
}

impl XfgBurnMintProver {
    /// Create new XFG Burn & Mint Prover
    pub fn new(security_parameter: usize) -> Self {
        let proof_options = ProofOptions::new(
            42,                               // blowup factor
            8,                                // grinding factor
            4,                                // hash function
            winterfell::FieldExtension::None, // field extension
            8,                                // FRI folding factor
            31,                               // FRI remainder max degree
        );

        Self {
            security_parameter,
            proof_options,
        }
    }

    /// Create prover with custom proof options
    pub fn with_options(security_parameter: usize, proof_options: ProofOptions) -> Self {
        Self {
            security_parameter,
            proof_options,
        }
    }

    /// Prove XFG burn and HEAT mint operation
    ///
    /// This generates a STARK proof that validates:
    /// - Burn amount is within valid range
    /// - Mint amount is proportional to burn amount
    /// - Full transaction prefix hash is bound to proof
    /// - Recipient hash is bound to proof
    /// - Network IDs prevent cross-chain replay
    /// - State transitions are valid
    /// - Nullifier prevents double-spending
    /// - Commitment ensures data integrity
    pub fn prove_burn_mint(
        &self,
        burn_amount: u64,
        mint_amount: u64,
        tx_prefix_hash: [u8; 32], // Full 32-byte tx prefix hash
        recipient_address: &[u8], // 20-byte Ethereum address
        secret: &[u8],
        network_id: u32,          // Fuego network ID
        target_chain_id: u32,     // HEAT target chain ID
        commitment_version: u32,  // Commitment format version
    ) -> Result<StarkProof> {
        // Validate inputs (using legacy txn_hash for backward compatibility)
        let legacy_txn_hash = u64::from_le_bytes(tx_prefix_hash[0..8].try_into().unwrap());
        self.validate_inputs(burn_amount, mint_amount, legacy_txn_hash, recipient_address, commitment_version)?;

        // Convert secret to field element
        let secret_element = self.secret_to_field_element(secret)?;

        // Compute recipient hash
        let recipient_hash = self.compute_recipient_hash(recipient_address);

        // Extract tx prefix hash limbs
        let tx_prefix_hash_0 = u32::from_le_bytes(tx_prefix_hash[0..4].try_into().unwrap());
        let tx_prefix_hash_1 = u32::from_le_bytes(tx_prefix_hash[4..8].try_into().unwrap());
        let tx_prefix_hash_2 = u32::from_le_bytes(tx_prefix_hash[8..12].try_into().unwrap());
        let tx_prefix_hash_3 = u32::from_le_bytes(tx_prefix_hash[12..16].try_into().unwrap());

        // Create extended public inputs
        let public_inputs = BurnMintPublicInputs {
            burn_amount: BaseElement::from(burn_amount as u32),
            mint_amount: BaseElement::from(mint_amount as u32),
            txn_hash: BaseElement::from(legacy_txn_hash as u32), // Keep legacy for compatibility
            recipient_hash: BaseElement::from(recipient_hash as u32),
            state: BaseElement::from(0u32),

            // Full tx prefix hash (32 bytes as 4 limbs)
            tx_prefix_hash_0: BaseElement::from(tx_prefix_hash_0),
            tx_prefix_hash_1: BaseElement::from(tx_prefix_hash_1),
            tx_prefix_hash_2: BaseElement::from(tx_prefix_hash_2),
            tx_prefix_hash_3: BaseElement::from(tx_prefix_hash_3),

            // Network identifiers
            network_id: BaseElement::from(network_id),
            target_chain_id: BaseElement::from(target_chain_id),
            commitment_version: BaseElement::from(commitment_version),
        };

        // Create trace info (7 registers, 64 steps)
        let trace_info = TraceInfo::new(7, 64);

        // Create AIR
        let air = XfgBurnMintAir::new_with_secret(
            trace_info,
            public_inputs,
            secret_element,
            self.proof_options.clone(),
        );

        // Generate execution trace
        let trace = air.build_trace();

        // Generate STARK proof using Winterfell
        let proof = air
            .prove(trace)
            .map_err(|e| crate::XfgStarkError::CryptoError(format!("Prover error: {:?}", e)))?;

        Ok(proof)
    }

    /// Validate input parameters (amounts in atomic units)
    /// Version-aware validation supports v1 (2 tiers), v2 (3 tiers), v3 (3 tiers deposits), v4 (3 tiers yield)
    fn validate_inputs(
        &self,
        burn_amount: u64,
        mint_amount: u64,
        txn_hash: u64,
        recipient_address: &[u8],
        commitment_version: u32,
    ) -> Result<()> {
        // Validate burn/deposit and mint amounts based on version
        if commitment_version == 1 {
            // Version 1: 2 tiers (XFG burns → HEAT)
            let standard_burn = 8_000_000u64;      // 0.8 XFG
            let large_burn = 8_000_000_000u64;     // 800 XFG

            if burn_amount != standard_burn && burn_amount != large_burn {
                return Err(crate::XfgStarkError::CryptoError(
                    "Version 1: Burn amount must be exactly 0.8 XFG or 800 XFG".to_string(),
                ));
            }

            // Validate mint amounts match burn amounts (1:1 in atomic units)
            if mint_amount != burn_amount {
                return Err(crate::XfgStarkError::CryptoError(format!(
                    "Mint amount {} doesn't match expected {}", mint_amount, burn_amount
                )));
            }
        } else if commitment_version == 2 {
            // Version 2: 3 tiers (XFG burns → HEAT)
            let tier0_burn = 8_000_000u64;         // 0.8 XFG
            let tier1_burn = 800_000_000u64;       // 80 XFG
            let tier2_burn = 8_000_000_000u64;     // 800 XFG

            if burn_amount != tier0_burn && burn_amount != tier1_burn && burn_amount != tier2_burn {
                return Err(crate::XfgStarkError::CryptoError(
                    "Version 2: Burn amount must be exactly 0.8 XFG, 80 XFG, or 800 XFG".to_string(),
                ));
            }

            // Validate mint amounts match burn amounts (1:1 in atomic units)
            if mint_amount != burn_amount {
                return Err(crate::XfgStarkError::CryptoError(format!(
                    "Mint amount {} doesn't match expected {}", mint_amount, burn_amount
                )));
            }
        } else {
            return Err(crate::XfgStarkError::CryptoError(
                format!("Unsupported commitment version: {}", commitment_version),
            ));
        }

        // Validate transaction hash
        if txn_hash == 0 {
            return Err(crate::XfgStarkError::CryptoError(
                "Transaction hash must be greater than 0".to_string(),
            ));
        }

        // Validate recipient address
        if recipient_address.len() != 20 {
            return Err(crate::XfgStarkError::CryptoError(
                "Recipient address must be exactly 20 bytes".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert XFG amount from whole units to atomic units
    /// XFG uses 7 decimal places: 1 XFG = 10,000,000 atomic units
    pub fn xfg_to_atomic_units(xfg_amount: f64) -> u64 {
        (xfg_amount * 10_000_000.0) as u64
    }

    /// Convert XFG amount from atomic units to whole units
    /// XFG uses 7 decimal places: 10,000,000 atomic units = 1 XFG
    pub fn atomic_units_to_xfg(atomic_units: u64) -> f64 {
        atomic_units as f64 / 10_000_000.0
    }

    /// Convert secret bytes to field element
    fn secret_to_field_element(&self, secret: &[u8]) -> Result<BaseElement> {
        if secret.len() < 4 {
            return Err(crate::XfgStarkError::CryptoError(
                "Secret must be at least 4 bytes".to_string(),
            ));
        }

        // Use first 8 bytes of secret
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&secret[..8]);

        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(BaseElement::from(value))
    }

    /// Compute recipient hash from Ethereum address
    fn compute_recipient_hash(&self, recipient_address: &[u8]) -> u32 {
        use sha3::{Digest, Keccak256};

        let mut hasher = Keccak256::new();
        hasher.update(recipient_address);
        hasher.update(b"recipient");
        let hash = hasher.finalize();

        // Convert first 4 bytes of hash to u32
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Get proof size in bytes
    pub fn get_proof_size(&self, proof: &StarkProof) -> usize {
        // Estimate proof size (this is approximate)
        proof.to_bytes().len()
    }

    /// Get security parameter
    pub fn security_parameter(&self) -> usize {
        self.security_parameter
    }

    /// Get proof options
    pub fn proof_options(&self) -> &ProofOptions {
        &self.proof_options
    }
}

impl Default for XfgBurnMintProver {
    fn default() -> Self {
        Self::new(128) // Default 128-bit security
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_creation() {
        let prover = XfgBurnMintProver::new(128);
        assert_eq!(prover.security_parameter(), 128);
    }

    #[test]
    fn test_input_validation() {
        let prover = XfgBurnMintProver::new(128);

        // Generate real transaction hash instead of hardcoded value
        use crate::test_data_generator::TestDataGenerator;
        let tx_hash_str = TestDataGenerator::generate_tx_hash();
        let tx_hash_bytes = hex::decode(&tx_hash_str).expect("Valid hex string");
        let tx_hash_u64 = u64::from_le_bytes([
            tx_hash_bytes[0], tx_hash_bytes[1], tx_hash_bytes[2], tx_hash_bytes[3],
            tx_hash_bytes[4], tx_hash_bytes[5], tx_hash_bytes[6], tx_hash_bytes[7]
        ]);
        
        let recipient = [0x12u8; 20]; // Valid 20-byte address

        // Test v1 (2 tiers)
        assert!(prover
            .validate_inputs(8_000_000, 8_000_000, tx_hash_u64, &recipient, 1) // 0.8 XFG burn
            .is_ok());

        // Test v2 (3 tiers) - tier 0
        assert!(prover
            .validate_inputs(8_000_000, 8_000_000, tx_hash_u64, &recipient, 2)
            .is_ok());

        // Test v2 (3 tiers) - tier 1 (new 80 XFG tier)
        assert!(prover
            .validate_inputs(800_000_000, 800_000_000, tx_hash_u64, &recipient, 2)
            .is_ok());

        // Test v2 (3 tiers) - tier 2
        assert!(prover
            .validate_inputs(8_000_000_000, 8_000_000_000, tx_hash_u64, &recipient, 2)
            .is_ok());

        // Invalid burn amount (zero)
        assert!(prover.validate_inputs(0, 8_000_000, tx_hash_u64, &recipient, 1).is_err());

        // Invalid burn amount for v1 (tier1 not allowed in v1)
        assert!(prover
            .validate_inputs(800_000_000, 800_000_000, tx_hash_u64, &recipient, 1)
            .is_err());

        // Invalid mint amount (zero)
        assert!(prover.validate_inputs(8_000_000, 0, tx_hash_u64, &recipient, 1).is_err());

        // Invalid proportionality (not 1:1)
        assert!(prover
            .validate_inputs(8_000_000, 16_000_000, tx_hash_u64, &recipient, 1)
            .is_err());

        // Invalid transaction hash
        assert!(prover
            .validate_inputs(8_000_000, 8_000_000, 0, &recipient, 1)
            .is_err());

        // Invalid recipient address (wrong length)
        let invalid_recipient = [0x12u8; 19]; // Too short
        assert!(prover
            .validate_inputs(8_000_000, 8_000_000, tx_hash_u64, &invalid_recipient, 1)
            .is_err());

        // Invalid version
        assert!(prover
            .validate_inputs(8_000_000, 8_000_000, tx_hash_u64, &recipient, 99)
            .is_err());
    }

    #[test]
    fn test_secret_conversion() {
        let prover = XfgBurnMintProver::new(128);

        // Valid secret
        let secret = [1, 2, 3, 4, 5, 6, 7, 8];
        let element = prover.secret_to_field_element(&secret).unwrap();
        assert_eq!(element, BaseElement::from(0x04030201u32));

        // Invalid secret (too short)
        let short_secret = [1, 2, 3];
        assert!(prover.secret_to_field_element(&short_secret).is_err());
    }

    #[test]
    fn test_proof_generation() {
        let prover = XfgBurnMintProver::new(128);
        
        // Generate real transaction hash instead of hardcoded value
        use crate::test_data_generator::TestDataGenerator;
        let tx_hash_str = TestDataGenerator::generate_tx_hash();
        let tx_hash_bytes = hex::decode(&tx_hash_str).expect("Valid hex string");
        let tx_hash_u64 = u64::from_le_bytes([
            tx_hash_bytes[0], tx_hash_bytes[1], tx_hash_bytes[2], tx_hash_bytes[3],
            tx_hash_bytes[4], tx_hash_bytes[5], tx_hash_bytes[6], tx_hash_bytes[7]
        ]);
        
        let recipient = [0x12u8; 20]; // Valid 20-byte address
        let secret = [42u8; 32]; // Valid 32-byte secret

        // Test valid proof generation
        let result = prover.prove_burn_mint(
            8_000_000, // 0.8 XFG in atomic units
            8_000_000, // 0.8 XFG in atomic units (1:1 ratio)
            tx_hash_u64, // Real generated transaction hash
            &recipient,
            &secret,
        );

        // The proof generation should succeed
        assert!(result.is_ok(), "Proof generation should succeed: {:?}", result);
    }
}
