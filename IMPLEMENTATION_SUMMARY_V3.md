# XFG-STARK v3 Implementation Summary
## HEAT Burns + COLD Deposits with API Verification

**Date:** 2026-01-17
**Version:** v3 (MVP with API verification)

---

## 🎯 **What Changed**

### **1. Added 4th Tier Across All Systems**
- **Tier 0**: 0.8 XFG → 8M HEAT or 0.000008 COLD equivalent
- **Tier 1**: 8 XFG → 80M HEAT or 0.00008 COLD equivalent **(NEW)**
- **Tier 2**: 80 XFG → 800M HEAT or 0.0008 COLD equivalent
- **Tier 3**: 800 XFG → 8B HEAT or 0.008 COLD equivalent

### **2. Simplified Verification (MVP Approach)**
- **Removed**: On-chain Eldernode verification
- **Added**: Trusted API verifier (usexfg.org backend)
- **Flow**:
  1. User generates STARK proof locally
  2. Submits to usexfg.org API
  3. API validates proof off-chain
  4. API calls contract to mint tokens
  5. Prevents on-chain proof verification costs

### **3. Fixed LP Rewards Authorization**
- **Problem**: LPRewardsManager couldn't mint CD tokens
- **Solution**: Multi-minter authorization in FuegoCOLDAOToken
- **Minters**: COLDProofVerifier + LPRewardsManager

### **4. Separated Accounting**
- **XFG deposits**: Tracked separately in `totalXFGPrincipalLocked`
- **HEAT LP rewards**: Tracked separately in `totalHEATInLPRewards`
- **Prevents**: Mixing XFG and HEAT in statistics

---

## 📁 **New & Updated Files**

### **New Contracts:**
1. `TierConversions.sol` - Shared tier constants library
2. `HEATBurnProofVerifier_v3.sol` - API-verified HEAT minting (4 tiers)
3. `COLDProofVerifier_v3.sol` - API-verified CD minting (4 tiers)

### **Updated Contracts:**
4. `FuegoCOLDAOToken.sol` - Multi-minter + dual accounting
5. `LPRewardsManager.sol` - Fixed minting calls

### **Rust Updates Needed:**
6. `src/bin/xfg-stark-cli.rs` - Add proof type selection (HEAT vs COLD)
7. `src/proof_data_schema.rs` - Add 4th tier + proof type enum

---

## 🔄 **Migration Path**

### **From v2 → v3:**

1. **Deploy new contracts:**
   ```
   TierConversions (library, no deployment)
   HEATBurnProofVerifier_v3
   COLDProofVerifier_v3
   FuegoCOLDAOToken (updated)
   ```

2. **Authorize minters:**
   ```solidity
   cdToken.addAuthorizedMinter(coldProofVerifier_v3);
   cdToken.addAuthorizedMinter(lpRewardsManager);
   ```

3. **Set API verifier:**
   ```solidity
   heatVerifier_v3.updateAPIVerifier(usexfgBackend);
   coldVerifier_v3.updateAPIVerifier(usexfgBackend);
   ```

4. **Update HEAT token:**
   ```solidity
   heatToken.updateMinter(heatVerifier_v3);
   ```

5. **Keep v2 contracts running** (parallel operation)

---

## 🏗️ **Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                    User (Fuego L1)                          │
│                                                             │
│  1. Burns/Deposits XFG on Fuego                            │
│  2. Generates STARK proof locally (xfg-stark-cli)         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              usexfg.org API Backend                         │
│                                                             │
│  3. Receives proof + validates STARK                       │
│  4. Checks nullifier not used                              │
│  5. Calls verifier contract on Arbitrum                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│        Arbitrum L2 Verifier Contract                        │
│                                                             │
│  HEAT: HEATBurnProofVerifier_v3                            │
│  COLD: COLDProofVerifier_v3                                │
│                                                             │
│  6. Validates caller is API verifier                       │
│  7. Checks nullifier not used (prevent replay)             │
│  8. Sends L2→L1 message via ARB_SYS                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│           Ethereum L1 Token Contract                        │
│                                                             │
│  HEAT: EmbersTokenHEAT (mintFromL2)                        │
│  COLD: FuegoCOLDAOToken (mintFromL2)                       │
│                                                             │
│  9. Receives message from Arbitrum Outbox                  │
│  10. Checks commitment not used                            │
│  11. Mints tokens to recipient                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔐 **Security Model**

### **Trust Assumptions:**
- **API Backend (usexfg.org)** is trusted to:
  - Validate STARK proofs correctly
  - Not submit fake proofs
  - Rate limit / prevent spam
  - Maintain availability

### **On-Chain Guarantees:**
- **Nullifier uniqueness**: Prevents double-spending
- **Commitment uniqueness**: Prevents replay attacks
- **Tier validation**: Only valid tiers accepted
- **Authorization**: Only API backend can call verifier

### **Future Enhancements:**
- **Phase 2**: Add on-chain STARK verification
- **Phase 3**: Add Eldernode consensus
- **Phase 4**: Decentralize API backend

---

## 📊 **Tier Conversions Reference**

| Tier | XFG Amount | HEAT Amount | COLD Base | CD Interest (8% APY) |
|------|------------|-------------|-----------|---------------------|
| 0    | 0.8 XFG    | 8M HEAT     | 0.000008  | 0.00000064 CD       |
| 1    | 8 XFG      | 80M HEAT    | 0.00008   | 0.0000064 CD        |
| 2    | 80 XFG     | 800M HEAT   | 0.0008    | 0.000064 CD         |
| 3    | 800 XFG    | 8B HEAT     | 0.008     | 0.00064 CD          |

**Formula for CD Interest:**
```
CD = (XFG / 100,000) × (APY / 100)
```

---

## 🚀 **Next Steps**

### **Immediate (Complete Rust Integration):**
1. ✅ Update proof schema for 4 tiers
2. ✅ Add proof type enum (HEAT vs COLD)
3. ✅ Update CLI to ask "Burn or Deposit?"
4. ⏳ Update validation to support all 4 tiers
5. ⏳ Update proof generation for both types

### **API Backend (usexfg.org):**
1. Implement STARK proof verification endpoint
2. Integrate with Arbitrum RPC
3. Call verifier contracts
4. Add rate limiting / anti-spam
5. Monitor nullifier database

### **Testing:**
1. Test 4 tier proof generation
2. Test API verification flow
3. Test L2→L1 message relay
4. Test minting on L1
5. Test LP rewards claiming

### **Deployment:**
1. Deploy to Arbitrum Sepolia (testnet)
2. Deploy API backend to staging
3. End-to-end testing
4. Mainnet deployment
5. Monitor & optimize

---

## 📝 **Contract Addresses (To Be Deployed)**

### **Arbitrum Sepolia:**
```
HEATBurnProofVerifier_v3: TBD
COLDProofVerifier_v3: TBD
```

### **Ethereum Sepolia:**
```
HEATToken: TBD
FuegoCOLDAOToken: TBD
COLDAOGovernor: TBD
LPRewardsManager: TBD
```

### **Arbitrum One (Mainnet):**
```
HEATBurnProofVerifier_v3: TBD
COLDProofVerifier_v3: TBD
```

### **Ethereum Mainnet:**
```
HEATToken: TBD
FuegoCOLDAOToken: TBD
COLDAOGovernor: TBD
LPRewardsManager: TBD
```

---

## 🤝 **API Integration Guide**

### **Endpoint: POST /api/v1/verify-proof**

**Request:**
```json
{
  "proof_type": "HEAT" | "COLD",
  "tier": 0 | 1 | 2 | 3,
  "nullifier": "0x...",
  "commitment": "0x...",
  "recipient": "0x...",
  "stark_proof": "0x...",
  "public_inputs": { ... }
}
```

**Response (Success):**
```json
{
  "status": "verified",
  "transaction_hash": "0x...",
  "minting_initiated": true,
  "estimated_confirmation_blocks": 12
}
```

**Response (Error):**
```json
{
  "status": "error",
  "error_code": "NULLIFIER_USED" | "INVALID_PROOF" | "INVALID_TIER",
  "message": "..."
}
```

---

## 📞 **Support & Resources**

- **Documentation**: `/docs/`
- **Rust CLI Guide**: `/docs/XFG_STARK_PROOF_USER_GUIDE.md`
- **API Docs**: `https://usexfg.org/api/docs`
- **Contract Source**: This repository

---

**Winter is coming. ❄️**
