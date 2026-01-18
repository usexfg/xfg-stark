// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./FuegoCOLDAOToken.sol";
import "./interfaces/IArbSys.sol";
import "./interfaces/ICOLDAOGovernor.sol";
import "./TierConversions.sol";

/**
 * @title COLD Deposit Proof Verifier (v3 - API Verified)
 * @dev Verifies XFG deposit proofs and mints CD INTEREST tokens on Arbitrum
 * @dev Simplified MVP: API verification instead of on-chain Eldernode verification
 * @dev XFG principal is LOCKED (not burned) - unlocks after 3 months on Fuego
 * @dev Only CD INTEREST is minted to depositor (principal stays locked)
 * @dev 4 deposit tiers: 0.8, 8, 80, 800 XFG
 * @dev Interest calculation: Supply ratio first (1:100,000), then APY applied
 * @dev Trusted API at usexfg.org validates STARK proofs off-chain before submission
 */
contract COLDProofVerifier is Ownable, Pausable, ReentrancyGuard {

    /* -------------------------------------------------------------------------- */
    /*                                   Events                                   */
    /* -------------------------------------------------------------------------- */

    event ProofVerified(
        bytes32 indexed depositTxHash,
        address indexed recipient,
        uint256 xfgPrincipal,
        uint256 cdInterest,
        uint8 tier,
        bytes32 indexed nullifier
    );

    event L1GasPaid(
        address indexed user,
        uint256 gasAmount,
        uint256 ticketId,
        bytes32 indexed commitment
    );

    event InterestCalculated(
        uint256 xfgPrincipal,
        uint256 baseAmount,
        uint256 apyBps,
        uint256 cdInterest
    );

    event APIVerifierUpdated(
        address indexed oldVerifier,
        address indexed newVerifier
    );

    /* -------------------------------------------------------------------------- */
    /*                                   State                                    */
    /* -------------------------------------------------------------------------- */

    /// @dev Fuego COLDAO token contract (CD)
    FuegoCOLDAOToken public immutable cdToken;

    /// @dev COLDAO governor contract (provides current APY)
    ICOLDAOGovernor public coldaoGovernor;

    /// @dev Trusted API verifier address (usexfg.org backend)
    address public apiVerifier;

    /// @dev Arbitrum messenger precompile (0x64) – used to send L2→L1 message
    IArbSys public constant ARB_SYS = IArbSys(address(0x64));

    /// @dev Supply ratio: 1 COLD : 100,000 XFG
    uint256 public constant SUPPLY_RATIO_DENOMINATOR = 100_000;

    /// @dev CD token decimals (12)
    uint256 public constant CD_DECIMALS = 12;

    /// @dev XFG decimals (7)
    uint256 public constant XFG_DECIMALS = 7;

    /// @dev Fuego network ID (chain ID)
    uint256 public constant FUEGO_NETWORK_ID = 93385046440755750514194170694064996624;

    /// @dev Used nullifiers to prevent double-spending
    mapping(bytes32 => bool) public nullifiersUsed;

    /// @dev Statistics
    uint256 public totalProofsVerified;
    uint256 public totalCDInterestMinted;
    uint256 public totalXFGPrincipalLocked;
    uint256 public totalClaims;

    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    constructor(
        address _cdToken,
        address _coldaoGovernor,
        address _apiVerifier,
        address initialOwner
    ) Ownable(initialOwner) {
        require(_cdToken != address(0), "Invalid CD token address");
        require(_coldaoGovernor != address(0), "Invalid COLDAO governor address");
        require(_apiVerifier != address(0), "Invalid API verifier address");

        cdToken = FuegoCOLDAOToken(_cdToken);
        coldaoGovernor = ICOLDAOGovernor(_coldaoGovernor);
        apiVerifier = _apiVerifier;
    }

    /* -------------------------------------------------------------------------- */
    /*                              Core Functions                                */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Claim CD interest tokens by providing API-verified deposit proof
     * @dev API verifier (usexfg.org) validates STARK proof off-chain and calls this
     * @param recipient Address to receive CD tokens
     * @param depositTier Tier index: 0=0.8 XFG, 1=8 XFG, 2=80 XFG, 3=800 XFG
     * @param nullifier Unique nullifier from STARK proof
     * @param commitment Commitment hash from STARK proof
     */
    function claimCDInterest(
        address recipient,
        uint8 depositTier,
        bytes32 nullifier,
        bytes32 commitment
    ) external payable whenNotPaused nonReentrant {
        require(msg.sender == apiVerifier, "Only API verifier can submit proofs");
        require(recipient != address(0), "Invalid recipient address");
        require(TierConversions.isValidTier(depositTier), "Invalid tier");

        // Verify nullifier hasn't been used
        require(!nullifiersUsed[nullifier], "Nullifier already used");

        // Mark nullifier used (prevent replay on L2)
        nullifiersUsed[nullifier] = true;

        // Get XFG principal for tier
        uint256 xfgPrincipal = TierConversions.getXFGForTier(depositTier);

        // Calculate CD interest amount
        uint256 cdInterest = calculateInterest(xfgPrincipal);
        require(cdInterest > 0, "Interest amount must be greater than 0");

        // Get current edition ID from CD token
        uint256 editionId = cdToken.currentEditionId() - 1; // Current active edition

        // ------------------------------------------------------------------
        // 📤  SEND MESSAGE TO L1 CD TOKEN CONTRACT VIA ARB SYS
        // ------------------------------------------------------------------

        // Compose calldata for L1 mint function with version=3
        bytes memory data = abi.encodeWithSignature(
            "mintFromL2(bytes32,address,uint256,uint256,uint256,uint32)",
            commitment,
            recipient,
            editionId,
            cdInterest,
            xfgPrincipal,
            3 // commitment_version = 3 for COLD deposits
        );

        // Send cross-chain message to L1
        uint256 ticketId = ARB_SYS.sendTxToL1{value: msg.value}(address(cdToken), data);

        emit L1GasPaid(msg.sender, msg.value, ticketId, commitment);
        emit ProofVerified(
            depositTxHashFromCommitment(commitment),
            recipient,
            xfgPrincipal,
            cdInterest,
            depositTier,
            nullifier
        );

        // Update statistics
        totalProofsVerified += 1;
        totalCDInterestMinted += cdInterest;
        totalXFGPrincipalLocked += xfgPrincipal;
        totalClaims += 1;
    }

    /* -------------------------------------------------------------------------- */
    /*                          Interest Calculation                              */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Calculate CD interest from XFG principal
     * @dev Formula: (XFG / 100,000) × APY
     * @dev Step 1: Apply supply ratio (1 COLD : 100,000 XFG)
     * @dev Step 2: Apply APY from COLDAO governor
     * @param xfgPrincipal XFG principal amount (in atomic units with 7 decimals)
     * @return cdInterest CD interest amount (in atomic units with 12 decimals)
     *
     * Example: 0.8 XFG at 8% APY
     *   xfgPrincipal = 8,000,000 (0.8 XFG in atomic units)
     *   baseAmount = 8,000,000 / 100,000 = 80 (base COLD atomic units)
     *   Convert to 12 decimals: 80 * 10^5 = 8,000,000 (0.000008 COLD)
     *   Apply 8% APY: 8,000,000 * 800 / 10,000 = 640 (0.00000064 CD)
     */
    function calculateInterest(uint256 xfgPrincipal)
        public
        view
        returns (uint256 cdInterest)
    {
        // Get current APY from COLDAO governor (in basis points, e.g., 800 = 8%)
        uint256 apyBps = coldaoGovernor.getCurrentAPY();
        require(apyBps > 0, "APY must be greater than 0");
        require(apyBps <= 10000, "APY cannot exceed 100%");

        // Step 1: Apply supply ratio (1 COLD : 100,000 XFG)
        uint256 baseAmount = xfgPrincipal / SUPPLY_RATIO_DENOMINATOR;

        // Step 2: Convert from XFG decimals (7) to CD decimals (12)
        uint256 baseAmountCD = baseAmount * 10**(CD_DECIMALS - XFG_DECIMALS);

        // Step 3: Apply APY
        cdInterest = (baseAmountCD * apyBps) / 10000;

        emit InterestCalculated(xfgPrincipal, baseAmountCD, apyBps, cdInterest);

        return cdInterest;
    }

    /* -------------------------------------------------------------------------- */
    /*                          Gas Estimation Functions                          */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Estimate L1 gas fees for cross-chain CD minting
     * @param recipient Address to receive CD tokens
     * @param depositTier Tier index (0-3)
     * @return estimatedGasFee Estimated L1 gas fee in wei
     */
    function estimateL1GasFee(address recipient, uint8 depositTier)
        external
        view
        returns (uint256 estimatedGasFee)
    {
        require(TierConversions.isValidTier(depositTier), "Invalid tier");

        uint256 xfgPrincipal = TierConversions.getXFGForTier(depositTier);
        uint256 cdInterest = calculateInterest(xfgPrincipal);
        uint256 editionId = cdToken.currentEditionId() - 1;

        // Compose the same calldata that will be sent
        bytes memory data = abi.encodeWithSignature(
            "mintFromL2(bytes32,address,uint256,uint256,uint256,uint32)",
            bytes32(0), // dummy commitment
            recipient,
            editionId,
            cdInterest,
            xfgPrincipal,
            3 // version 3
        );

        // Estimate L1 gas fee
        uint256 calldataSize = data.length;
        uint256 estimatedL1GasPrice = 20 gwei; // Conservative estimate

        // Base cost for L2→L1 message + calldata cost
        estimatedGasFee = (21000 + calldataSize * 16) * estimatedL1GasPrice;

        return estimatedGasFee;
    }

    /**
     * @dev Get recommended L1 gas fee with 20% buffer
     * @param recipient Address to receive CD tokens
     * @param depositTier Tier index (0-3)
     * @return recommendedFee Recommended L1 gas fee with 20% buffer
     */
    function getRecommendedGasFee(address recipient, uint8 depositTier)
        external
        view
        returns (uint256 recommendedFee)
    {
        uint256 baseFee = this.estimateL1GasFee(recipient, depositTier);
        recommendedFee = (baseFee * 120) / 100; // 20% buffer
        return recommendedFee;
    }

    /**
     * @dev Derive deposit transaction hash from commitment
     * @param commitment Commitment hash
     * @return txHash Derived transaction hash
     */
    function depositTxHashFromCommitment(bytes32 commitment) internal pure returns (bytes32 txHash) {
        return keccak256(abi.encodePacked("COLD_DEPOSIT:", commitment));
    }

    /* -------------------------------------------------------------------------- */
    /*                          Admin Functions                                   */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Update API verifier address (owner only)
     * @param newVerifier New API verifier address
     */
    function updateAPIVerifier(address newVerifier) external onlyOwner {
        require(newVerifier != address(0), "Invalid verifier address");
        address oldVerifier = apiVerifier;
        apiVerifier = newVerifier;
        emit APIVerifierUpdated(oldVerifier, newVerifier);
    }

    /**
     * @dev Update COLDAO governor contract
     * @param newGovernor New governor contract address
     */
    function updateCOLDAOGovernor(address newGovernor) external onlyOwner {
        require(newGovernor != address(0), "Invalid governor address");
        coldaoGovernor = ICOLDAOGovernor(newGovernor);
    }

    /**
     * @dev Pause the contract (emergency use only)
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpause the contract
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @dev Rescue accidentally sent ETH
     */
    function rescueETH() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }

    /* -------------------------------------------------------------------------- */
    /*                          View Functions                                    */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Check if nullifier has been used
     * @param nullifier Nullifier to check
     * @return used True if nullifier has been used
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool used) {
        return nullifiersUsed[nullifier];
    }

    /**
     * @dev Get total XFG locked in human-readable format
     * @return xfgLocked Total XFG locked (with 7 decimal places)
     */
    function getTotalXFGLockedReadable() external view returns (uint256 xfgLocked) {
        // XFG has 7 decimal places, so divide by 10^7
        return totalXFGPrincipalLocked / 10_000_000;
    }

    /**
     * @dev Get contract statistics
     * @return stats Array of statistics [totalProofs, totalCD, totalXFG, totalClaims]
     */
    function getStatistics() external view returns (uint256[4] memory stats) {
        stats[0] = totalProofsVerified;
        stats[1] = totalCDInterestMinted;
        stats[2] = totalXFGPrincipalLocked;
        stats[3] = totalClaims;
    }

    /**
     * @dev Get tier information
     * @param tier Tier index (0-3)
     * @return xfgAmount XFG amount for tier
     * @return cdInterest Estimated CD interest (at current APY)
     * @return tierName Human-readable tier name
     */
    function getTierInfo(uint8 tier) external view returns (
        uint256 xfgAmount,
        uint256 cdInterest,
        string memory tierName
    ) {
        require(TierConversions.isValidTier(tier), "Invalid tier");
        xfgAmount = TierConversions.getXFGForTier(tier);
        cdInterest = calculateInterest(xfgAmount);
        tierName = TierConversions.getTierName(tier);
    }

    /* -------------------------------------------------------------------------- */
    /*                          Receive Function                                  */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Receive function to accept ETH for L1 gas fees
     */
    receive() external payable {}

} /** winter is coming */
