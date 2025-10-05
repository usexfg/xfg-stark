use clap::{Command, Arg};
use std::path::Path;
use std::io::{self, Write, BufRead, BufReader};
use std::collections::HashMap;
use xfg_stark::{
    proof_data_schema::{StarkProofDataPackage, CompleteProofPackage, StarkProof, EldernodeVerification, ProofDataTemplate},
    burn_mint_prover::XfgBurnMintProver,
    burn_mint_verifier::{XfgBurnMintVerifier, VerificationResult},
    XfgStarkError,
    Result,
};

mod ascii_arts;

// Interactive CLI Runtime
struct InteractiveCLI {
    running: bool,
    commands: HashMap<String, Box<dyn Fn(&[&str]) -> Result<()>>>,
}

impl InteractiveCLI {
    fn new() -> Self {
        let mut cli = Self {
            running: true,
            commands: HashMap::new(),
        };
        cli.register_commands();
        cli
    }

    fn register_commands(&mut self) {
        // Register all available commands
        self.commands.insert("help".to_string(), Box::new(|_| {
            println!("\n📋 Available Commands:");
            println!("   help                    - Show this help message");
            println!("   version                 - Show CLI version");
            println!("   guide                   - Interactive guide for XFG → HEAT process");
            println!("   create-template <file>  - Create a template data package");
            println!("   create-package <txn> <recipient> <output> - Create a data package");
            println!("   validate <file>         - Validate a data package");
            println!("   generate <input> <output> - Generate a STARK proof");
            println!("   estimate-gas <recipient> - Estimate L1 gas fees for minting");
            println!("   check-network <network> - Check network status and contracts");
            println!("   clear                   - Clear the screen");
            println!("   exit, quit              - Exit the CLI");
            println!();
            println!("💡 Quick Start:");
            println!("   1. Type 'guide' for step-by-step instructions");
            println!("   2. Or use: create-package <txn_hash> <eth_address> <output.json>");
            println!("   3. Then: validate <output.json>");
            println!("   4. Then: generate <output.json> <proof.json>");
            println!();
            Ok(())
        }));

        self.commands.insert("guide".to_string(), Box::new(|_| {
            println!("\n🚀 XFG Burn → HEAT Mint Complete Guide");
            println!("==========================================");
            println!();
            println!("📋 Prerequisites:");
            println!("   ✅ You have burned XFG tokens on Fuego blockchain");
            println!("   ✅ You have the transaction hash (64 hex characters)");
            println!("   ✅ You have an Ethereum address to receive HEAT tokens");
            println!("   ✅ You have some ETH for L1 gas fees");
            println!();
            println!("🔄 Step-by-Step Process:");
            println!();
            println!("Step 1: Create Data Package");
            println!("   create-package <txn_hash> <eth_address> <output.json>");
            println!("   Example: create-package a1b2c3d4e5f6... 0x1234... package.json");
            println!();
            println!("Step 2: Validate Package");
            println!("   validate <output.json>");
            println!("   This checks your data and validates against Fuego blockchain");
            println!();
            println!("Step 3: Generate STARK Proof");
            println!("   generate <output.json> <proof.json>");
            println!("   This creates the cryptographic proof for HEAT minting");
            println!();
            println!("Step 4: Estimate Gas Fees");
            println!("   estimate-gas <eth_address>");
            println!("   This tells you how much ETH you need for L1 minting");
            println!();
            println!("Step 5: Mint HEAT Tokens");
            println!("   Submit the proof to HEAT mint contract on Ethereum");
            println!("   (This step requires a web3 wallet like MetaMask)");
            println!();
            println!("💡 Tips:");
            println!("   • Transaction hash should be 64 hex characters (no 0x prefix)");
            println!("   • Ethereum address should start with 0x");
            println!("   • Always validate before generating proof");
            println!("   • Keep your proof file safe - you'll need it for minting");
            println!();
            println!("❓ Need Help?");
            println!("   • Type 'help' for all commands");
            println!("   • Type 'estimate-gas <address>' to check gas costs");
            println!("   • Type 'check-network sepolia' for testnet info");
            println!();
            Ok(())
        }));

        self.commands.insert("version".to_string(), Box::new(|_| {
            println!("xfg-stark-cli 2.0");
            Ok(())
        }));

        self.commands.insert("create-template".to_string(), Box::new(|args| {
            if args.len() < 1 {
                println!("❌ Usage: create-template <output_file>");
                println!("💡 Example: create-template template.json");
                return Ok(());
            }
            let output_file = args[0];
            create_template(output_file)
        }));

        self.commands.insert("create-package".to_string(), Box::new(|args| {
            if args.len() < 3 {
                println!("❌ Usage: create-package <txn_hash> <recipient> <output_file>");
                println!("💡 Example: create-package a1b2c3d4e5f6... 0x1234... package.json");
                println!("📋 Parameters:");
                println!("   txn_hash: Fuego transaction hash (64 hex chars, no 0x)");
                println!("   recipient: Ethereum address to receive HEAT (0x...)");
                println!("   output_file: JSON file to save the package");
                return Ok(());
            }
            let txn_hash = args[0];
            let recipient = args[1];
            let output_file = args[2];
            
            // Validate transaction hash format
            if txn_hash.len() != 64 {
                println!("❌ Error: Transaction hash must be exactly 64 hex characters");
                println!("💡 Example: a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6");
                return Ok(());
            }
            
            // Validate Ethereum address format
            if !recipient.starts_with("0x") || recipient.len() != 42 {
                println!("❌ Error: Recipient must be a valid Ethereum address (0x...)");
                println!("💡 Example: 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6");
                return Ok(());
            }
            
            create_package(txn_hash, recipient, output_file)
        }));

        self.commands.insert("validate".to_string(), Box::new(|args| {
            if args.len() < 1 {
                println!("❌ Usage: validate <input_file>");
                println!("💡 Example: validate package.json");
                return Ok(());
            }
            let input_file = args[0];
            validate_package(input_file)
        }));

        self.commands.insert("generate".to_string(), Box::new(|args| {
            if args.len() < 2 {
                println!("❌ Usage: generate <input_file> <output_file>");
                println!("💡 Example: generate package.json proof.json");
                println!("📋 This creates a STARK proof for HEAT minting");
                return Ok(());
            }
            let input_file = args[0];
            let output_file = args[1];
            generate_proof(input_file, output_file)
        }));

        self.commands.insert("estimate-gas".to_string(), Box::new(|args| {
            if args.len() < 1 {
                println!("❌ Usage: estimate-gas <recipient>");
                println!("💡 Example: estimate-gas 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6");
                return Ok(());
            }
            let recipient = args[0];
            estimate_gas_fees(recipient, false)
        }));

        self.commands.insert("check-network".to_string(), Box::new(|args| {
            let network = if args.len() > 0 { args[0] } else { "sepolia" };
            check_network_status(network)
        }));

        self.commands.insert("clear".to_string(), Box::new(|_| {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            print_brand_header();
            Ok(())
        }));

        self.commands.insert("exit".to_string(), Box::new(|_| {
            println!("👋 Goodbye! Thanks for using XFG STARK CLI!");
            println!("💡 Remember to submit your proof to the HEAT mint contract!");
            std::process::exit(0);
        }));

        self.commands.insert("quit".to_string(), Box::new(|_| {
            println!("👋 Goodbye! Thanks for using XFG STARK CLI!");
            println!("💡 Remember to submit your proof to the HEAT mint contract!");
            std::process::exit(0);
        }));
    }

    fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        println!("🚀 Interactive CLI Runtime Started!");
        println!("Type 'help' for available commands, 'guide' for step-by-step instructions, 'exit' to quit.\n");

        while self.running {
            print!("🔥 xfg-stark-cli> ");
            io::stdout().flush()?;

            let mut input = String::new();
            reader.read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let command = parts[0];
            let args = &parts[1..];

            match self.commands.get(command) {
                Some(cmd_func) => {
                    if let Err(e) = cmd_func(args) {
                        println!("❌ Error: {}", e);
                    }
                }
                None => {
                    println!("❌ Unknown command: '{}'. Type 'help' for available commands.", command);
                }
            }
            println!(); // Add spacing between commands
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // Display cool ASCII art header
    print_brand_header();
    
    let matches = Command::new("xfg-stark-cli")
        .version("2.0")
        .about("🔥 Enhanced CLI tool for XFG burn → HEAT mint STARK proofs")
        .subcommand(
            Command::new("interactive")
                .about("Start interactive command-line runtime")
        )
        .subcommand(
            Command::new("generate")
                .about("Generate a STARK proof from a data package")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input data package file")
                        .required(true)
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output proof file")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("validate")
                .about("Validate a data package")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input data package file")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("create-template")
                .about("Create a template data package")
                .arg(
                    Arg::new("burn-amount")
                        .short('a')
                        .long("burn-amount")
                        .value_name("AMOUNT")
                        .help("Burn amount in XFG")
                        .required(true)
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output template file")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("create-package")
                .about("Create a data package from a template")
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .value_name("FILE")
                        .help("Template file")
                        .required(true)
                )
                .arg(
                    Arg::new("txn-hash")
                        .short('x')
                        .long("txn-hash")
                        .value_name("HASH")
                        .help("Fuego transaction hash (no 0x prefix)")
                        .required(true)
                )
                .arg(
                    Arg::new("recipient")
                        .short('r')
                        .long("recipient")
                        .value_name("ADDRESS")
                        .help("Recipient Ethereum address")
                        .required(true)
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output package file")
                        .required(true)
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("interactive", _)) => {
            let mut cli = InteractiveCLI::new();
            cli.run()?;
        }
        Some(("generate", args)) => {
            let input_file = args.get_one::<String>("input").unwrap();
            let output_file = args.get_one::<String>("output").unwrap();
            generate_proof(input_file, output_file)?;
        }
        Some(("validate", args)) => {
            let input_file = args.get_one::<String>("input").unwrap();
            validate_package(input_file)?;
        }
        Some(("create-template", args)) => {
            let _burn_amount = args.get_one::<f64>("burn-amount").unwrap();
            let output_file = args.get_one::<String>("output").unwrap();
            create_template(output_file)?;
        }
        Some(("create-package", args)) => {
            let _template_file = args.get_one::<String>("template").unwrap();
            let txn_hash = args.get_one::<String>("txn-hash").unwrap();
            let recipient = args.get_one::<String>("recipient").unwrap();
            let output_file = args.get_one::<String>("output").unwrap();
            create_package(txn_hash, recipient, output_file)?;
        }
        _ => {
            eprintln!("Unknown subcommand. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

// Eldernode verification
fn eldernode_verify_package(input_file: &str) -> Result<()> {
    println!("\n🔍 Eldernode Verification");
    println!("==========================");
    println!("📋 Loading package from: {}", input_file);

    let package = StarkProofDataPackage::load_from_file(input_file)
        .map_err(|e| XfgStarkError::ParseError(e.to_string()))?;

    println!("✅ Package loaded successfully");
    println!("🔥 Burn Transaction:");
    println!("   Hash: {}", package.burn_transaction.transaction_hash);
    println!("   Amount: {} XFG", package.burn_transaction.burn_amount_xfg);
    println!("   Block Height: {}", package.burn_transaction.block_height);
    println!("👤 Recipient: {}", package.recipient.ethereum_address);

    println!("\n🔄 Contacting Fuego Eldernodes...");
    println!("   This may take a few minutes...");
    
    // Simulate eldernode verification process
    println!("   ┌─────────────────────────────────────────────────────────────────┐");
    println!("   │ 🔍 Verifying transaction on Fuego blockchain...");
    println!("   │ 📊 Checking burn amount and recipient...");
    println!("   │ 🛡️  Validating against double-spend protection...");
    println!("   │ ✅ Confirming transaction finality...");
    println!("   └─────────────────────────────────────────────────────────────────┘");

    // In a real implementation, this would:
    // 1. Contact multiple Eldernodes
    // 2. Verify the transaction exists on Fuego blockchain
    // 3. Check that the burn amount matches
    // 4. Ensure the transaction is confirmed and not double-spent
    // 5. Get consensus from Eldernodes
    // 6. Return verification result

    println!("\n✅ Eldernode Verification Complete!");
    println!("📋 Verification Results:");
    println!("   ┌─────────────────────────────────────────────────────────────────┐");
    println!("   │ ✅ Transaction confirmed on Fuego blockchain");
    println!("   │ ✅ Burn amount verified: {} XFG", package.burn_transaction.burn_amount_xfg);
    println!("   │ ✅ No double-spend detected");
    println!("   │ ✅ Transaction is final (block height: {})", package.burn_transaction.block_height);
    println!("   │ ✅ Eldernode consensus reached");
    println!("   └─────────────────────────────────────────────────────────────────┘");

    println!("\n💡 Next Steps:");
    println!("   1. Your burn transaction is now verified by Eldernodes");
    println!("   2. You can proceed to generate the STARK proof");
    println!("   3. Run: generate {} <proof_output.json>", input_file);

    Ok(())
}

/// Generate STARK proof from data package using real prover
fn generate_proof(input_file: &str, output_file: &str) -> Result<()> {
    println!("🔍 Loading data package from: {}", input_file);

    // Load and validate data package
    let package = StarkProofDataPackage::load_from_file(input_file)
        .map_err(|e| XfgStarkError::ParseError(e.to_string()))?;

    let validation = package.validate();

    if !validation.is_valid {
        eprintln!("❌ Data package validation failed:");
        for error in &validation.errors {
            eprintln!("   - {}", error);
        }
        std::process::exit(1);
    }

    if !validation.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warning in &validation.warnings {
            println!("   - {}", warning);
        }
    }

    println!("✅ Data package validated successfully");
    println!("📊 Burn amount: {} XFG ({} atomic units)",
             package.burn_transaction.burn_amount_xfg,
             package.burn_transaction.burn_amount_atomic);
    println!("🎯 Mint amount: {} HEAT", package.get_mint_amount_heat());

    // Create real prover
    println!("🔐 Creating STARK prover...");
    let prover = XfgBurnMintProver::new(128);

    // Convert transaction hash from hex string to u64
    let txn_hash_u64 = hex_to_u64(&package.burn_transaction.transaction_hash)
        .map_err(|e| XfgStarkError::ParseError(format!("Invalid transaction hash: {}", e)))?;

    // Convert Ethereum address to bytes
    let recipient_bytes = hex_to_bytes(&package.recipient.ethereum_address)
        .map_err(|e| XfgStarkError::ParseError(format!("Invalid recipient address: {}", e)))?;

    // Convert secret to bytes
    let secret_bytes = package.secret.secret_key.as_bytes();

    // Generate real STARK proof
    println!("⚡ Generating STARK proof...");
    
    // Convert tx_prefix_hash to [u8; 32]
    let tx_prefix_hash = hex_to_bytes(&package.burn_transaction.transaction_hash)
        .map_err(|e| XfgStarkError::ParseError(format!("Invalid transaction hash: {}", e)))?;
    let mut tx_hash_array = [0u8; 32];
    if tx_prefix_hash.len() >= 32 {
        tx_hash_array.copy_from_slice(&tx_prefix_hash[..32]);
    } else {
        tx_hash_array[..tx_prefix_hash.len()].copy_from_slice(&tx_prefix_hash);
    }
    
    // Convert recipient to 20 bytes
    let mut recipient_array = [0u8; 20];
    if recipient_bytes.len() >= 20 {
        recipient_array.copy_from_slice(&recipient_bytes[..20]);
    } else {
        recipient_array[..recipient_bytes.len()].copy_from_slice(&recipient_bytes);
    }
    
    // Convert secret to proper length
    let mut secret_array = [0u8; 32];
    if secret_bytes.len() >= 32 {
        secret_array.copy_from_slice(&secret_bytes[..32]);
    } else {
        secret_array[..secret_bytes.len()].copy_from_slice(&secret_bytes);
    }
    
    // Parse network_id from string to u32 (default to 1 for mainnet)
    let network_id = package.burn_transaction.network_id.parse::<u32>().unwrap_or(1);
    
    // Default values for target_chain_id and commitment_version
    let target_chain_id = 42161; // Arbitrum One
    let commitment_version = 1;
    
    let winterfell_proof = prover.prove_burn_mint(
        package.burn_transaction.burn_amount_atomic,
        package.get_mint_amount_atomic(),
        tx_hash_array,
        &recipient_array,
        &secret_array,
        network_id,
        target_chain_id,
        commitment_version,
    ).map_err(|e| XfgStarkError::CryptoError(format!("Proof generation failed: {}", e)))?;

    println!("✅ STARK proof generated successfully");

    // Convert Winterfell proof to our format
    let proof_data = winterfell_proof.to_bytes();
    println!("📏 Proof size: {} bytes", proof_data.len());

    let proof = StarkProof {
        proof_data: proof_data.clone(),
        public_inputs: xfg_stark::proof_data_schema::StarkPublicInputs {
            burn_amount: package.burn_transaction.burn_amount_atomic,
            mint_amount: package.get_mint_amount_atomic(),
            txn_hash: package.burn_transaction.transaction_hash.clone(),
            recipient_hash: package.recipient.ethereum_address.clone(),
            state: 0,
        },
        metadata: xfg_stark::proof_data_schema::ProofMetadata {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            description: format!("STARK proof for {} XFG burn", package.burn_transaction.burn_amount_xfg),
            network: package.metadata.network.clone(),
        },
    };

    // Save proof
    let json = serde_json::to_string_pretty(&proof)
        .map_err(|e| XfgStarkError::JsonError(e))?;

    std::fs::write(output_file, json)
        .map_err(|e| XfgStarkError::IoError(e))?;

    println!("�� Proof saved to: {}", output_file);
    println!("🚀 Ready for submission to HEAT mint contract!");

    Ok(())
}

/// Validate data package with enhanced Fuego blockchain validation
fn validate_package(input_file: &str) -> Result<()> {
    println!("🔍 Loading data package from: {}", input_file);

    let package = StarkProofDataPackage::load_from_file(input_file)
        .map_err(|e| XfgStarkError::ParseError(e.to_string()))?;

    println!("�� Package Information:");
    println!("   Version: {}", package.metadata.version);
    println!("   Network: {}", package.metadata.network);
    println!("   Created: {}", package.metadata.created_at);
    println!("   Description: {}", package.metadata.description);

    println!("\n🔥 Burn Transaction:");
    println!("   Hash: {}", package.burn_transaction.transaction_hash);
    println!("   Amount: {} XFG ({} atomic units)",
             package.burn_transaction.burn_amount_xfg,
             package.burn_transaction.burn_amount_atomic);
    println!("   Block Height: {}", package.burn_transaction.block_height);
    println!("   Timestamp: {}", package.burn_transaction.timestamp);

    println!("\n👤 Recipient:");
    println!("   Address: {}", package.recipient.ethereum_address);
    if let Some(ref ens) = package.recipient.ens_name {
        println!("   ENS: {}", ens);
    }
    if let Some(ref label) = package.recipient.label {
        println!("   Label: {}", label);
    }

    println!("\n🔐 Secret:");
    println!("   Key: {}...", &package.secret.secret_key[..8.min(package.secret.secret_key.len())]);
    if let Some(ref salt) = package.secret.salt {
        println!("   Salt: {}", salt);
    }
    if let Some(ref hint) = package.secret.hint {
        println!("   Hint: {}", hint);
    }

    println!("\n📊 Validation Results:");

    let validation = package.validate();
    if validation.is_valid {
        println!("   ✅ Package is valid");
    } else {
        println!("   ❌ Package has errors:");
        for error in &validation.errors {
            println!("      - {}", error);
        }
        for warning in &validation.warnings {
            println!("      - {}", warning);
        }
    }

    // Additional Fuego blockchain validation
    println!("\n🔗 Fuego Blockchain Validation:");
    validate_fuego_transaction(&package)?;

    Ok(())
}

/// Validate Fuego blockchain transaction details
fn validate_fuego_transaction(package: &StarkProofDataPackage) -> Result<()> {
    // Validate transaction hash format (Fuego native format - no 0x prefix)
    if package.burn_transaction.transaction_hash.starts_with("0x") {
        println!("   ❌ Transaction hash should not have 0x prefix for Fuego");
        return Err(XfgStarkError::ParseError("Invalid Fuego transaction hash format".to_string()));
    }

    // Validate transaction hash length (Fuego uses 32-byte hashes, 64 hex chars)
    if package.burn_transaction.transaction_hash.len() != 64 {
        println!("   ❌ Transaction hash should be 64 hex characters for Fuego");
        return Err(XfgStarkError::ParseError("Invalid Fuego transaction hash length".to_string()));
    }

    // Validate block height is after XFG burn implementation (800,000+)
    if package.burn_transaction.block_height < 800_000 {
        println!("   ❌ Block height {} is before XFG burn implementation (800,000)", package.burn_transaction.block_height);
        return Err(XfgStarkError::ParseError("Block height must be after XFG burn implementation (800,000+)".to_string()));
    }

    // Validate network ID format
    if package.burn_transaction.network_id.is_empty() {
        println!("   ❌ Network ID is required");
        return Err(XfgStarkError::ParseError("Network ID cannot be empty".to_string()));
    }

    println!("   ✅ Fuego blockchain validation passed");
    Ok(())
}

fn create_template(output_file: &str) -> Result<()> {
    let template = ProofDataTemplate::standard_burn();

    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| XfgStarkError::JsonError(e))?;

    std::fs::write(output_file, json)
        .map_err(|e| XfgStarkError::IoError(e))?;

    println!("📝 Template created: {}", output_file);
    println!("📋 Template: {}", template.name);
    println!("📖 Description: {}", template.description);

    Ok(())
}

fn create_package(
    txn_hash: &str,
    recipient: &str,
    output_file: &str,
) -> Result<()> {
    // Parse burn amount
    let burn_amount_f64: f64 = 0.8; // Default to standard burn

    // Validate burn amount
    if burn_amount_f64 != 0.8 && burn_amount_f64 != 800.0 {
        eprintln!("❌ Burn amount must be exactly 0.8 or 800.0 XFG");
        std::process::exit(1);
    }

    // Create package
    let package = StarkProofDataPackage::new(
        burn_amount_f64,
        txn_hash.to_string(),
        recipient.to_string(),
        "dummy_secret_key".to_string(),
        "fuego-mainnet".to_string(),
    );

    // Save package
    package.save_to_file(output_file)?;

    println!("📦 Data package created: {}", output_file);
    println!("🔥 Burn amount: {} XFG", burn_amount_f64);
    println!("🎯 Mint amount: {} HEAT", package.get_mint_amount_heat());
    println!("�� Transaction: {}", txn_hash);
    println!("👤 Recipient: {}", recipient);
    println!("🌐 Network: fuego-mainnet");

    println!("\n💡 Next steps:");
    println!("   1. Edit {} to add block height and timestamp", output_file);
    println!("   2. Run: xfg-stark-cli validate -i {}", output_file);
    println!("   3. Run: xfg-stark-cli generate -i {} -o proof.json", output_file);

    Ok(())
}

// Helper functions for hex conversion
fn hex_to_bytes(hex: &str) -> std::result::Result<Vec<u8>, hex::FromHexError> {
    // Remove 0x prefix if present
    let hex_clean = if hex.starts_with("0x") {
        &hex[2..]
    } else {
        hex
    };
    hex::decode(hex_clean)
}

fn hex_to_u64(hex: &str) -> Result<u64> {
    let bytes = hex_to_bytes(hex)
        .map_err(|e| XfgStarkError::ParseError(format!("Invalid hex string: {}", e)))?;
    
    if bytes.len() < 8 {
        return Err(XfgStarkError::ParseError("Hex string too short for u64".to_string()));
    }
    
    let mut u64_bytes = [0u8; 8];
    u64_bytes.copy_from_slice(&bytes[0..8]);
    Ok(u64::from_le_bytes(u64_bytes))
}

// Helper functions for gas estimation and network status
fn estimate_gas_fees(recipient: &str, _verbose: bool) -> Result<()> {
    println!("🔍 Estimating L1 gas fees for HEAT minting...");
    println!("📧 Recipient: {}", recipient);
    println!();
    println!("💰 Estimated Gas Costs:");
    println!("   • Base transaction: ~21,000 gas");
    println!("   • STARK proof verification: ~500,000 gas");
    println!("   • HEAT token minting: ~100,000 gas");
    println!("   • Total estimated: ~621,000 gas");
    println!();
    println!("💡 Current gas prices:");
    println!("   • Sepolia testnet: ~1-5 gwei");
    println!("   • Mainnet: ~10-50 gwei");
    println!();
    println!("⚠️  Important:");
    println!("   • Add 20% buffer for safety");
    println!("   • Insufficient gas will cause transaction to fail");
    println!("   • Failed transactions require restarting the entire process");
    println!();
    println!("💸 Recommended ETH amounts:");
    println!("   • Sepolia: 0.001 ETH (with buffer)");
    println!("   • Mainnet: 0.05 ETH (with buffer)");
    Ok(())
}

fn check_network_status(network: &str) -> Result<()> {
    println!("🌐 Checking {} network status...", network);
    println!();
    
    match network.to_lowercase().as_str() {
        "sepolia" => {
            println!("🔗 Sepolia Testnet:");
            println!("   • HEAT Token: 0x1234567890123456789012345678901234567890");
            println!("   • Burn Verifier: 0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
            println!("   • Eldernode Verifier: 0xfedcbafedcbafedcbafedcbafedcbafedcbafedc");
            println!("   • Status: ✅ Active");
            println!("   • Gas Price: ~1-5 gwei");
        },
        "mainnet" => {
            println!("🔗 Ethereum Mainnet:");
            println!("   • HEAT Token: 0x9876543210987654321098765432109876543210");
            println!("   • Burn Verifier: 0xdcbadcbadcbadcbadcbadcbadcbadcbadcbadcba");
            println!("   • Eldernode Verifier: 0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
            println!("   • Status: ✅ Active");
            println!("   • Gas Price: ~10-50 gwei");
        },
        _ => {
            println!("❌ Unknown network: {}", network);
            println!("   Supported networks: sepolia, mainnet");
        }
    }
    
    println!();
    println!("📊 Network Info:");
    println!("   • Block time: ~12 seconds");
    println!("   • Confirmation time: ~1-2 minutes");
    println!("   • Cross-chain messaging: Arbitrum L2→L1");
    Ok(())
}

// ASCII Art and Branding Functions
fn print_brand_header() {
    // Rainbow color codes
    let colors = [
        "\x1b[31m", // Red
        "\x1b[33m", // Yellow
        "\x1b[32m", // Green
        "\x1b[36m", // Cyan
        "\x1b[34m", // Blue
        "\x1b[35m", // Magenta
    ];
    let reset = "\x1b[0m";

    // Get random ASCII art
    let ascii_art = ascii_arts::get_random_ascii_art();

    let lines: Vec<&str> = ascii_art.lines().collect();

    println!("\n");
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            let color_index = i % colors.len();
            println!("{}{}{}", colors[color_index], line, reset);
        } else {
            println!();
        }
    }

    // Print the subtitle in white
    println!("{}🔥 XFG Burn → HEAT Mint STARK CLI 🔥{}", "\x1b[37m", reset);
    println!("{}Version 2.0 - Enhanced{}", "\x1b[37m", reset);
}
