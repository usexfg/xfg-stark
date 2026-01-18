# COLD Deposits V3 - Final Update

**Date:** 2026-01-18
**Status:** ✅ Ready for deployment
**Branch:** `cold-starks`

---

## 🎯 **What Changed**

### **From:**
- 3 time-based tiers (3mo, 7mo, 12mo)
- Simple tier → CD amount lookup
- No legacy deposit support

### **To:**
- 8 amount×time tiers (4 amounts × 2 terms)
- Combined XFG amount + lock period matrix
- Legacy deposit support for 800 XFG before 2026 @ 80% APY

---

## 📊 **New Tier Structure**

### **Standard Tiers (Post-2026):**

| Tier | Amount | Term | APY | CD Interest (atomic) |
|------|--------|------|-----|---------------------|
| 0 | 0.8 XFG | 3mo | 8% | 640,000 |
| 1 | 0.8 XFG | 12mo | 27% | 2,160,000 |
| 2 | 8 XFG | 3mo | 18% | 14,400,000 |
| 3 | 8 XFG | 12mo | 33% | 26,400,000 |
| 4 | 80 XFG | 3mo | 27% | 216,000,000 |
| 5 | 80 XFG | 12mo | 42% | 336,000,000 |
| 6 | 800 XFG | 3mo | 33% | 2,640,000,000 |
| 7 | 800 XFG | 12mo | 69% | 5,520,000,000 |

### **Legacy Tiers (Pre-2026, 800 XFG only):**

| Tier | Amount | Term | APY | CD Interest (atomic) |
|------|--------|------|-----|---------------------|
| 6 | 800 XFG | 3mo | **80%** | 6,400,000,000 |
| 7 | 800 XFG | 12mo | **80%** | 6,400,000,000 |

**Legacy Cutoff:** 2026-01-01 00:00:00 UTC (timestamp: `1735689600`)

---

## 🔧 **Contract Changes**

### **COLDDepositProofVerifier.sol**

#### **Updated Constants:**
```solidity
// Added legacy cutoff timestamp
uint256 public constant LEGACY_CUTOFF_TIMESTAMP = 1735689600;

// Expanded from 3 tiers to 8 tiers (12 decimals)
TIER0_CD_INTEREST = 640_000;          // 0.00000064 CD
TIER1_CD_INTEREST = 2_160_000;        // 0.00000216 CD
TIER2_CD_INTEREST = 14_400_000;       // 0.0000144 CD
TIER3_CD_INTEREST = 26_400_000;       // 0.0000264 CD
TIER4_CD_INTEREST = 216_000_000;      // 0.000216 CD
TIER5_CD_INTEREST = 336_000_000;      // 0.000336 CD
TIER6_CD_INTEREST = 2_640_000_000;    // 0.00264 CD
TIER7_CD_INTEREST = 5_520_000_000;    // 0.00552 CD

// Added legacy tier amounts (only tier 6-7)
LEGACY_TIER6_CD = 6_400_000_000;      // 0.0064 CD
LEGACY_TIER7_CD = 6_400_000_000;      // 0.0064 CD
```

#### **Updated Function Signature:**
```solidity
// OLD:
function claimCD(
    address recipient,
    uint8 lockTier,  // 0-2
    bytes32 nullifier,
    bytes32 commitment,
    uint256 networkId
)

// NEW:
function claimCD(
    address recipient,
    uint8 tier,  // 0-7
    bytes32 nullifier,
    bytes32 commitment,
    uint256 networkId,
    uint256 depositTimestamp  // Added for legacy detection
)
```

#### **New Internal Functions:**
```solidity
function _getStandardCDAmount(uint8 tier) internal pure returns (uint256)
function _getLegacyCDAmount(uint8 tier) internal pure returns (uint256)
```

#### **Updated View Functions:**
```solidity
// Enhanced tier info with APY and legacy support
function getTierInfo(uint8 tier, bool isLegacy) external pure returns (
    uint256 cdAmount,
    string memory xfgAmount,
    string memory lockPeriod,
    uint256 apyBps
)

// New: Check legacy eligibility
function isLegacyDeposit(uint256 depositTimestamp, uint8 tier)
    external pure returns (bool)

// New: Get legacy tier amounts
function getLegacyTierAmounts() external pure returns (
    uint256 tier6,
    uint256 tier7
)

// Updated: Gas estimation with legacy support
function estimateL1GasFee(address recipient, uint8 tier, bool isLegacy)
function getRecommendedGasFee(address recipient, uint8 tier, bool isLegacy)
```

---

## 📝 **APY Design Rationale**

### **Why These APY Rates?**

**Amount-Based Progression:**
- Larger deposits earn higher APY to incentivize capital commitment
- 800 XFG earns 69% vs 0.8 XFG earns 27% (12mo term)

**Term-Based Progression:**
- 12-month locks earn significantly more than 3-month
- Rewards long-term commitment to COLDAO

**Legacy 80% APY:**
- Special early adopter bonus for 800 XFG deposits only
- Limited to deposits before 2026-01-01
- Creates urgency and rewards early supporters

### **APY Comparison:**

| Tier | 3mo APY | 12mo APY | Long-term Premium |
|------|---------|----------|-------------------|
| 0.8 XFG | 8% | 27% | +19% |
| 8 XFG | 18% | 33% | +15% |
| 80 XFG | 27% | 42% | +15% |
| 800 XFG | 33% | 69% | +36% |
| **Legacy** | **80%** | **80%** | **- ** |

---

## 🧮 **Tier Encoding**

**Formula:** `tier = (amountIndex * 2) + termIndex`

Where:
- `amountIndex`: 0 (0.8 XFG), 1 (8 XFG), 2 (80 XFG), 3 (800 XFG)
- `termIndex`: 0 (3mo), 1 (12mo)

**Examples:**
- Tier 0 = (0 * 2) + 0 = 0.8 XFG × 3mo
- Tier 1 = (0 * 2) + 1 = 0.8 XFG × 12mo
- Tier 6 = (3 * 2) + 0 = 800 XFG × 3mo
- Tier 7 = (3 * 2) + 1 = 800 XFG × 12mo

**Pattern Recognition:**
- Even tiers (0, 2, 4, 6) = 3-month lock
- Odd tiers (1, 3, 5, 7) = 12-month lock

---

## 🔐 **Legacy Deposit Logic**

```solidity
// Check if deposit qualifies for legacy rate
bool isLegacy = depositTimestamp < LEGACY_CUTOFF_TIMESTAMP;

// Get CD amount
uint256 cdAmount = isLegacy
    ? _getLegacyCDAmount(tier)   // 80% for tier 6-7
    : _getStandardCDAmount(tier); // Standard rates

// Legacy function only applies bonus to tier 6-7
function _getLegacyCDAmount(uint8 tier) internal pure returns (uint256) {
    if (tier == 6) return LEGACY_TIER6_CD;  // 6,400,000
    if (tier == 7) return LEGACY_TIER7_CD;  // 6,400,000

    // All other tiers use standard rates even if before 2026
    return _getStandardCDAmount(tier);
}
```

**Key Points:**
- Legacy bonus ONLY applies to tier 6 and tier 7 (800 XFG)
- Tiers 0-5 use standard APY even if deposited before 2026
- Timestamp check: `depositTimestamp < 1735689600`

---

## 📚 **Updated Documentation**

### **Created:**
- ✅ `COLD_TIER_REFERENCE.md` - Complete tier guide with examples
- ✅ `COLD_V3_FINAL_UPDATE.md` - This document

### **Updated:**
- ✅ `COLDDepositProofVerifier.sol` - Full tier structure + legacy support
- ✅ `COLD_VS_HEAT_COMPARISON.md` - Updated tier comparison
- ✅ `COLD_BRANCH_SUMMARY.md` - Updated tier constants
- ✅ `COLD_TESTNET_DEPLOYMENT.md` - Updated test examples

---

## 🧪 **Testing Requirements**

### **Test Cases:**

1. **Standard Deposits:**
   - [ ] All 8 tiers (0-7) with post-2026 timestamp
   - [ ] Verify correct CD amounts minted
   - [ ] Verify APY calculations

2. **Legacy Deposits:**
   - [ ] Tier 6 with pre-2026 timestamp → 6,400,000 CD
   - [ ] Tier 7 with pre-2026 timestamp → 6,400,000 CD
   - [ ] Tiers 0-5 with pre-2026 timestamp → standard rates (not legacy)

3. **Edge Cases:**
   - [ ] Timestamp exactly at cutoff (1735689600)
   - [ ] Invalid tiers (> 7) should revert
   - [ ] Legacy detection for non-800 XFG tiers

4. **View Functions:**
   - [ ] `getTierInfo(tier, isLegacy)` returns correct data
   - [ ] `isLegacyDeposit(timestamp, tier)` works correctly
   - [ ] `getAllTierAmounts()` returns all 8 values
   - [ ] `getLegacyTierAmounts()` returns tier 6-7 legacy values

5. **Gas Estimation:**
   - [ ] `estimateL1GasFee(recipient, tier, isLegacy)` accurate
   - [ ] Different gas costs for different tiers

---

## 🚀 **Deployment Checklist**

### **Pre-Deployment:**
- [ ] Review all contract changes
- [ ] Verify tier constants calculations
- [ ] Check legacy cutoff timestamp (1735689600 = 2026-01-01)
- [ ] Test on local environment

### **Testnet Deployment:**
- [ ] Deploy FuegoCOLDAOToken (Sepolia)
- [ ] Deploy COLDAOGovernor (Sepolia)
- [ ] Deploy COLDDepositProofVerifier (Arbitrum Sepolia)
- [ ] Configure minter authorization
- [ ] Set API verifier address
- [ ] Verify all contracts on explorers

### **Testing:**
- [ ] Test all 8 standard tiers
- [ ] Test 2 legacy tiers (6-7)
- [ ] Test nullifier protection
- [ ] Test commitment replay protection
- [ ] Test timestamp validation
- [ ] Measure actual gas costs

### **Mainnet Deployment:**
- [ ] Security audit
- [ ] Deploy to mainnet (same order as testnet)
- [ ] Configure production API verifier
- [ ] Monitor first transactions
- [ ] Document contract addresses

---

## 📞 **Key Contract Addresses**

### **Testnet (Sepolia/Arbitrum Sepolia):**
```
FuegoCOLDAOToken (Sepolia):           TBD
COLDAOGovernor (Sepolia):             TBD
COLDDepositProofVerifier (Arb Sep):   TBD
API Verifier:                         TBD
```

### **Mainnet (Ethereum/Arbitrum):**
```
FuegoCOLDAOToken (Ethereum):          TBD
COLDAOGovernor (Ethereum):            TBD
COLDDepositProofVerifier (Arbitrum):  TBD
API Verifier:                         TBD
```

---

## 💡 **Integration Notes**

### **For API Backend (usexfg.org):**

```javascript
// Extract from STARK proof
const {
  recipient,
  tier,           // 0-7
  nullifier,
  commitment,
  networkId,      // Fuego mainnet or testnet
  depositTimestamp // From Fuego transaction
} = proof;

// Call COLDDepositProofVerifier.claimCD
await coldVerifier.claimCD(
  recipient,
  tier,
  nullifier,
  commitment,
  networkId,
  depositTimestamp,
  { value: l1GasFee }
);
```

### **For Fuego Deposit Transaction:**

```rust
// Encode tier in tx_extra
let amount_index = match xfg_amount {
    0.8 => 0,
    8.0 => 1,
    80.0 => 2,
    800.0 => 3,
    _ => panic!("Invalid XFG amount")
};

let term_index = match lock_months {
    3 => 0,
    12 => 1,
    _ => panic!("Invalid lock period")
};

let tier = (amount_index * 2) + term_index; // 0-7
```

---

## ⚠️ **Important Notes**

1. **XFG Principal:** Still locked on Fuego, unlock handled separately
2. **CD Token:** Only interest is minted (not principal)
3. **Legacy Bonus:** Only tier 6-7 (800 XFG) before 2026-01-01
4. **Tier Validation:** Must be 0-7, reverts otherwise
5. **Timestamp Required:** depositTimestamp must be provided for all claims
6. **Nullifier Protection:** Each deposit can only be claimed once
7. **Network Support:** Both mainnet and testnet Fuego network IDs

---

## 📈 **Expected Outcomes**

### **User Experience:**
- Clear tier selection based on amount + term
- Higher rewards for larger deposits and longer locks
- Legacy bonus creates early adoption incentive

### **COLDAO Benefits:**
- Deeper liquidity with larger deposit tiers
- Longer commitment periods with 12-month option
- Early adopter rewards build community

### **Technical Benefits:**
- Single tier parameter (0-7) simplifies proof structure
- Pre-calculated CD amounts reduce gas costs
- Legacy detection via timestamp is simple and secure

---

## 🎯 **Next Steps**

1. ✅ Contract implementation complete
2. ✅ Documentation updated
3. ⏳ Deploy to testnet
4. ⏳ End-to-end testing
5. ⏳ Security audit
6. ⏳ Mainnet deployment
7. ⏳ Frontend integration
8. ⏳ API backend development

---

**Winter is coming. ❄️**

**Status:** Ready for testnet deployment and testing.
