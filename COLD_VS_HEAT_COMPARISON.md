# COLD vs HEAT: Key Differences

**Date:** 2026-01-17

---

## 🔥 **HEAT System (main branch)**

### **Purpose:** Burn XFG → Mint HEAT gas token

| Feature | Details |
|---------|---------|
| **Action** | Burn XFG on Fuego |
| **Token** | HEAT (ERC-20) |
| **Use Case** | Gas token for C0DL3 rollup |
| **Tiers** | 4 amount-based tiers |
| **Tier 0** | 0.8 XFG → 8M HEAT |
| **Tier 1** | 8 XFG → 80M HEAT |
| **Tier 2** | 80 XFG → 800M HEAT |
| **Tier 3** | 800 XFG → 8B HEAT |
| **Network IDs** | Mainnet only |
| **L2 Verifier** | HEATBurnProofVerifier |
| **L1 Token** | EmbersTokenHEAT |
| **Commitment Version** | 1 or 2 |

---

## ❄️ **COLD System (cold-starks branch)**

### **Purpose:** Lock XFG → Mint CD interest token

| Feature | Details |
|---------|---------|
| **Action** | Lock XFG on Fuego (unlocks later) |
| **Token** | CD (ERC-1155, multi-edition) |
| **Use Case** | DAO voting power + interest earnings |
| **Tiers** | 8 amount×time tiers (4 amounts × 2 terms) |
| **Tier 0** | 0.8 XFG × 3mo @ 8% → 640,000 atomic units |
| **Tier 1** | 0.8 XFG × 12mo @ 27% → 2,160,000 atomic units |
| **Tier 2** | 8 XFG × 3mo @ 18% → 14,400,000 atomic units |
| **Tier 3** | 8 XFG × 12mo @ 33% → 26,400,000 atomic units |
| **Tier 4** | 80 XFG × 3mo @ 27% → 216,000,000 atomic units |
| **Tier 5** | 80 XFG × 12mo @ 42% → 336,000,000 atomic units |
| **Tier 6** | 800 XFG × 3mo @ 33% → 2,640,000,000 atomic units |
| **Tier 7** | 800 XFG × 12mo @ 69% → 5,520,000,000 atomic units |
| **Legacy** | 800 XFG (tier 6-7) before 2026 @ 80% → 6,400,000,000 atomic units |
| **Network IDs** | Mainnet + Testnet |
| **L2 Verifier** | COLDDepositProofVerifier |
| **L1 Token** | FuegoCOLDAOToken |
| **Commitment Version** | 3 |

---

## 🔑 **Key Differences**

### **1. XFG Treatment:**
- **HEAT:** XFG is **burned** (destroyed permanently)
- **COLD:** XFG is **locked** (unlocks after time period)

### **2. Tier Structure:**
- **HEAT:** Amount-based (how much XFG) - 4 tiers
- **COLD:** Amount×time-based (how much + how long) - 8 tiers

### **3. Token Type:**
- **HEAT:** ERC-20 (fungible)
- **COLD:** ERC-1155 (semi-fungible, editions)

### **4. Minting:**
- **HEAT:** 1:1 scaled (1 XFG = 10M HEAT)
- **COLD:** Interest only (principal locked)

### **5. Use Case:**
- **HEAT:** Gas token for C0DL3 rollup
- **COLD:** DAO governance + yield

### **6. Network Support:**
- **HEAT:** Mainnet only
- **COLD:** Mainnet + Testnet

---

## 📊 **Network IDs**

```solidity
// HEAT (mainnet only)
FUEGO_NETWORK_ID = 93385046440755750514194170694064996624;

// COLD (both networks)
FUEGO_MAINNET_NETWORK_ID = 93385046440755750514194170694064996624;
FUEGO_TESTNET_NETWORK_ID = 112015110234323138517908755257434054688; // "TEST FUEGO NET  "
```

---

## 🎯 **When to Use Which**

### **Use HEAT when:**
- You want to **permanently burn** XFG
- You need **gas tokens** for C0DL3
- You want **immediate liquidity** (ERC-20)
- You're on **mainnet only**

### **Use COLD when:**
- You want to **earn interest** on XFG
- You can **lock funds** for a period
- You want **DAO voting power**
- You're **testing** (testnet support)
- You prefer **longer-term deposits**

---

## 🔄 **Common Elements**

Both systems share:
- ✅ STARK proof generation
- ✅ API verification (usexfg.org)
- ✅ Arbitrum L2 verifier
- ✅ L2→L1 bridge via ARB_SYS
- ✅ Nullifier protection
- ✅ Commitment replay protection
- ✅ Privacy-focused design

---

## 📝 **Contract Addresses**

### **HEAT System:**
```
HEATBurnProofVerifier (Arbitrum):  TBD
HEATToken (Ethereum):              TBD
```

### **COLD System:**
```
COLDDepositProofVerifier (Arbitrum): TBD
FuegoCOLDAOToken (Ethereum):         TBD
COLDAOGovernor (Ethereum):           TBD
```

---

## 🚀 **Future Plans**

### **Eventually:**
- Merge both systems into unified `xfg-stark-cli`
- User selects: "Burn (HEAT) or Deposit (COLD)?"
- Single proof generator, different contracts
- Same STARK proof structure, different metadata

### **For Now:**
- Separate branches for stability
- HEAT on `main`
- COLD on `cold-starks`
- Prevents breaking either system

---

**Winter is coming. ❄️**
