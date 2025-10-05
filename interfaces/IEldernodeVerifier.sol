// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IEldernodeVerifier - Eldernode Consensus Verification Interface
 * @dev Interface for verifying Eldernode consensus proofs on-chain
 * @dev Integrates with Fuego's Eldernode network for multi-layer validation
 */
interface IEldernodeVerifier {
    /**
     * @dev Verify Eldernode consensus proof for a commitment
     * @param commitment Commitment to verify
     * @param eldernodeProof Eldernode consensus proof data
     * @return isValid True if consensus verification passes
     */
    function verifyConsensusProof(
        bytes32 commitment,
        bytes calldata eldernodeProof
    ) external view returns (bool isValid);

    /**
     * @dev Get Eldernode consensus threshold
     * @return threshold Minimum number of Eldernodes required for consensus
     */
    function getConsensusThreshold() external view returns (uint64 threshold);

    /**
     * @dev Get total number of active Eldernodes
     * @return count Total number of active Eldernodes
     */
    function getActiveEldernodeCount() external view returns (uint64 count);

    /**
     * @dev Check if an Eldernode is registered and active
     * @param eldernodeId Eldernode identifier
     * @return isActive True if Eldernode is active
     */
    function isEldernodeActive(bytes32 eldernodeId) external view returns (bool isActive);

    /**
     * @dev Get Eldernode statistics
     * @return totalEldernodes Total registered Eldernodes
     * @return activeEldernodes Active Eldernodes
     * @return consensusThreshold Current consensus threshold
     * @return lastUpdateBlock Block number of last update
     */
    function getEldernodeStats() external view returns (
        uint64 totalEldernodes,
        uint64 activeEldernodes,
        uint64 consensusThreshold,
        uint256 lastUpdateBlock
    );

    /**
     * @dev Verify individual Eldernode signature
     * @param eldernodeId Eldernode identifier
     * @param message Message that was signed
     * @param signature Signature to verify
     * @return isValid True if signature is valid
     */
    function verifyEldernodeSignature(
        bytes32 eldernodeId,
        bytes32 message,
        bytes calldata signature
    ) external view returns (bool isValid);

    /**
     * @dev Parse Eldernode proof data structure
     * @param eldernodeProof Raw proof data
     * @return eldernodeIds Array of Eldernode IDs
     * @return signatures Array of signatures
     * @return messageHash Hash of the message that was signed
     * @return timestamp Timestamp of the consensus
     */
    function parseEldernodeProof(bytes calldata eldernodeProof) external pure returns (
        bytes32[] memory eldernodeIds,
        bytes[] memory signatures,
        bytes32 messageHash,
        uint64 timestamp
    );
}
