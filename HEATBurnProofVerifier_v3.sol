// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./HEATToken.sol";
import "./interfaces/IArbSys.sol";
import "./TierConversions.sol";

/**
 * @title HEAT Burn Proof Verifier (v3 - API Verified)
 * @dev Verifies XFG burn proofs and mints HEAT tokens on Arbitrum
 * @dev Simplified MVP: API verification instead of on-chain Eldernode verification
 * @dev 4 burn tiers: 0.8, 8, 80, 800 XFG → 8M, 80M, 800M, 8B HEAT
 * @dev Trusted API at usexfg.org validates STARK proofs off-chain before submission
 */
contract HEATBurnProofVerifier is Ownable, Pausable, ReentrancyGuard {

    /* -------------------------------------------------------------------------- */
    /*                                   Events                                   */
    /* -------------------------------------------------------------------------- */

    event ProofVerified(
        bytes32 indexed burnTxHash,
        address indexed recipient,
        uint256 amount,
        uint8 tier,
        bytes32 indexed nullifier
    );

    event L1GasPaid(
        address indexed user,
        uint256 gasAmount,
        uint256 ticketId,
        bytes32 indexed commitment
    );

    event APIVerifierUpdated(
        address indexed oldVerifier,
        address indexed newVerifier
    );

    /* -------------------------------------------------------------------------- */
    /*                                   State                                    */
    /* -------------------------------------------------------------------------- */

    /// @dev HEAT token contract
    EmbersTokenHEAT public immutable heatToken;

    /// @dev Trusted API verifier address (usexfg.org backend)
    address public apiVerifier;

    /// @dev Arbitrum messenger precompile (0x64) – used to send L2→L1 message
    IArbSys public constant ARB_SYS = IArbSys(address(0x64));

    /// @dev Fuego network ID (chain ID)
    uint256 public constant FUEGO_NETWORK_ID = 93385046440755750514194170694064996624;

    /// @dev Used nullifiers to prevent double-spending
    mapping(bytes32 => bool) public nullifiersUsed;

    /// @dev Statistics
    uint256 public totalProofsVerified;
    uint256 public totalHEATMinted;
    uint256 public totalClaims;

    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    constructor(
        address _heatToken,
        address _apiVerifier,
        address initialOwner
    ) Ownable(initialOwner) {
        require(_heatToken != address(0), "Invalid HEAT token address");
        require(_apiVerifier != address(0), "Invalid API verifier address");

        heatToken = EmbersTokenHEAT(_heatToken);
        apiVerifier = _apiVerifier;
    }

    /* -------------------------------------------------------------------------- */
    /*                              Core Functions                                */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Claim HEAT tokens by providing API-verified burn proof
     * @dev API verifier (usexfg.org) validates STARK proof off-chain and calls this
     * @param recipient Address to receive HEAT tokens
     * @param burnTier Tier index: 0=0.8 XFG, 1=8 XFG, 2=80 XFG, 3=800 XFG
     * @param nullifier Unique nullifier from STARK proof
     * @param commitment Commitment hash from STARK proof
     */
    function claimHEAT(
        address recipient,
        uint8 burnTier,
        bytes32 nullifier,
        bytes32 commitment
    ) external payable whenNotPaused nonReentrant {
        require(msg.sender == apiVerifier, "Only API verifier can submit proofs");
        require(recipient != address(0), "Invalid recipient address");
        require(TierConversions.isValidTier(burnTier), "Invalid tier");

        // Verify nullifier hasn't been used
        require(!nullifiersUsed[nullifier], "Nullifier already used");

        // Mark nullifier used (prevent replay on L2)
        nullifiersUsed[nullifier] = true;

        // Get HEAT amount for tier
        uint256 heatAmount = TierConversions.getHEATForTier(burnTier);

        // ------------------------------------------------------------------
        // 📤  SEND MESSAGE TO L1 HEAT TOKEN CONTRACT VIA ARB SYS
        // ------------------------------------------------------------------

        // Compose calldata for L1 mint function (version 3)
        bytes memory data = abi.encodeWithSignature(
            "mintFromL2(bytes32,address,uint256,uint32)",
            commitment,
            recipient,
            heatAmount,
            3  // commitment_version = 3 for API-verified proofs
        );

        // Enqueue call via ArbSys with L1 gas fees – returns ticket ID
        uint256 ticketId = ARB_SYS.sendTxToL1{value: msg.value}(address(heatToken), data);

        emit L1GasPaid(msg.sender, msg.value, ticketId, commitment);
        emit ProofVerified(
            burnTxHashFromCommitment(commitment),
            recipient,
            heatAmount,
            burnTier,
            nullifier
        );

        totalProofsVerified += 1;
        totalHEATMinted += heatAmount;
        totalClaims += 1;
    }

    /**
     * @dev Estimate L1 gas fees for cross-chain minting
     * @param recipient Address to receive HEAT tokens
     * @param burnTier Tier index (0-3)
     * @return estimatedGasFee Estimated L1 gas fee in wei
     */
    function estimateL1GasFee(address recipient, uint8 burnTier)
        external
        view
        returns (uint256 estimatedGasFee)
    {
        require(TierConversions.isValidTier(burnTier), "Invalid tier");

        uint256 heatAmount = TierConversions.getHEATForTier(burnTier);

        // Compose calldata for L1 mint function
        bytes memory data = abi.encodeWithSignature(
            "mintFromL2(bytes32,address,uint256,uint32)",
            bytes32(0), // placeholder commitment
            recipient,
            heatAmount,
            3  // version 3
        );

        // Estimate L1 gas fee based on calldata size and current L1 gas price
        uint256 calldataSize = data.length;
        uint256 estimatedL1GasPrice = 20 gwei; // Conservative estimate

        // Base cost for L2→L1 message + calldata cost
        estimatedGasFee = (21000 + calldataSize * 16) * estimatedL1GasPrice;

        return estimatedGasFee;
    }

    /**
     * @dev Get recommended L1 gas fee with 20% buffer
     * @param recipient Address to receive HEAT tokens
     * @param burnTier Tier index (0-3)
     * @return recommendedFee Recommended L1 gas fee with 20% buffer
     */
    function getRecommendedGasFee(address recipient, uint8 burnTier)
        external
        view
        returns (uint256 recommendedFee)
    {
        uint256 baseFee = this.estimateL1GasFee(recipient, burnTier);
        recommendedFee = (baseFee * 120) / 100; // 20% buffer
        return recommendedFee;
    }

    /**
     * @dev Extract burn transaction hash from commitment (for events)
     * @param commitment Commitment from STARK proof
     * @return Burn transaction hash
     */
    function burnTxHashFromCommitment(bytes32 commitment) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("HEAT_BURN:", commitment));
    }

    /* -------------------------------------------------------------------------- */
    /*                              Admin Functions                               */
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
     * @dev Pause the contract (owner only)
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpause the contract (owner only)
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
    /*                              View Functions                                */
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
     * @dev Get contract statistics
     * @return stats Array of statistics [totalProofs, totalHEAT, totalClaims]
     */
    function getStatistics() external view returns (uint256[3] memory stats) {
        stats[0] = totalProofsVerified;
        stats[1] = totalHEATMinted;
        stats[2] = totalClaims;
    }

    /**
     * @dev Get tier information
     * @param tier Tier index (0-3)
     * @return xfgAmount XFG amount for tier
     * @return heatAmount HEAT amount for tier
     * @return tierName Human-readable tier name
     */
    function getTierInfo(uint8 tier) external pure returns (
        uint256 xfgAmount,
        uint256 heatAmount,
        string memory tierName
    ) {
        require(TierConversions.isValidTier(tier), "Invalid tier");
        return (
            TierConversions.getXFGForTier(tier),
            TierConversions.getHEATForTier(tier),
            TierConversions.getTierName(tier)
        );
    }

    /* -------------------------------------------------------------------------- */
    /*                              Receive Function                              */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Receive function to accept ETH for L1 gas fees
     */
    receive() external payable {}

} /** winter is coming */
