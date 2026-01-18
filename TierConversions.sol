// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Tier Conversions Library
 * @dev Shared tier constants for XFG ↔ HEAT/CD conversions across all contracts
 * @dev Used by HEATBurnProofVerifier, COLDProofVerifier, and LPRewardsManager
 */
library TierConversions {

    /* -------------------------------------------------------------------------- */
    /*                          XFG Tier Constants (4 Tiers)                     */
    /* -------------------------------------------------------------------------- */

    /// @dev XFG has 7 decimals (1 XFG = 10,000,000 atomic units)
    uint256 public constant TIER0_XFG = 8_000_000;        // 0.8 XFG
    uint256 public constant TIER1_XFG = 80_000_000;       // 8 XFG
    uint256 public constant TIER2_XFG = 800_000_000;      // 80 XFG
    uint256 public constant TIER3_XFG = 8_000_000_000;    // 800 XFG

    /* -------------------------------------------------------------------------- */
    /*                          HEAT Tier Constants (4 Tiers)                    */
    /* -------------------------------------------------------------------------- */

    /// @dev HEAT has 18 decimals (standard ERC-20)
    uint256 public constant TIER0_HEAT = 8_000_000 * 10**18;        // 8M HEAT
    uint256 public constant TIER1_HEAT = 80_000_000 * 10**18;       // 80M HEAT
    uint256 public constant TIER2_HEAT = 800_000_000 * 10**18;      // 800M HEAT
    uint256 public constant TIER3_HEAT = 8_000_000_000 * 10**18;    // 8B HEAT

    /* -------------------------------------------------------------------------- */
    /*                          Conversion Functions                             */
    /* -------------------------------------------------------------------------- */

    /**
     * @dev Get HEAT amount for tier
     * @param tier Tier index (0-3)
     * @return heatAmount HEAT amount for tier
     */
    function getHEATForTier(uint8 tier) internal pure returns (uint256 heatAmount) {
        if (tier == 0) return TIER0_HEAT;
        if (tier == 1) return TIER1_HEAT;
        if (tier == 2) return TIER2_HEAT;
        if (tier == 3) return TIER3_HEAT;
        revert("Invalid tier: must be 0-3");
    }

    /**
     * @dev Get XFG amount for tier
     * @param tier Tier index (0-3)
     * @return xfgAmount XFG amount for tier
     */
    function getXFGForTier(uint8 tier) internal pure returns (uint256 xfgAmount) {
        if (tier == 0) return TIER0_XFG;
        if (tier == 1) return TIER1_XFG;
        if (tier == 2) return TIER2_XFG;
        if (tier == 3) return TIER3_XFG;
        revert("Invalid tier: must be 0-3");
    }

    /**
     * @dev Validate tier index
     * @param tier Tier to validate
     * @return valid True if tier is valid (0-3)
     */
    function isValidTier(uint8 tier) internal pure returns (bool valid) {
        return tier <= 3;
    }

    /**
     * @dev Get tier name (for display/logging)
     * @param tier Tier index
     * @return name Human-readable tier name
     */
    function getTierName(uint8 tier) internal pure returns (string memory name) {
        if (tier == 0) return "0.8 XFG";
        if (tier == 1) return "8 XFG";
        if (tier == 2) return "80 XFG";
        if (tier == 3) return "800 XFG";
        revert("Invalid tier: must be 0-3");
    }
}
