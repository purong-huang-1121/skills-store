---
name: okx-wallet-security
description: "Use this skill when the user asks to scan a transaction, check transaction safety, is this transaction safe, pre-execution check, security scan, tx risk check, check if this approve is safe, scan this swap tx, is this token safe, check token security, honeypot check, is this URL a scam, check if this dapp is safe, phishing site check, is this signature safe, check this signing request, check my approvals, show risky approvals, revoke approval, token authorization, ERC20 allowance, Permit2, or mentions transaction security scanning, token risk scanning, DApp/URL phishing detection, message signature safety, pre-execution risk analysis, malicious transaction detection, approval safety checks, or token approval management. Covers token-scan (batch token risk detection), dapp-scan (URL/domain phishing detection), tx-scan (EVM + Solana transaction pre-execution), sig-scan (EIP-712/personal_sign message scanning), and approvals (ERC-20 allowance and Permit2 authorization queries). Chinese: 安全扫描, 代币安全, 蜜罐检测, 貔貅盘, 钓鱼网站, 交易安全, 签名安全, 代币风险, 授权管理, 授权查询, 风险授权, 代币授权. Do NOT use for wallet balance, send, or history — use okx-wallet. Do NOT use for general programming questions about security."
license: Apache-2.0
metadata:
  author: okx
  version: "1.0.0"
  homepage: "https://web3.okx.com"
---

# Onchain OS Security

4 commands for token risk analysis, DApp phishing detection, transaction pre-execution security, and signature safety.

## Pre-flight Checks

Every time before running any `onchainos` command, follow these steps:

1. **Check binary**: Run `which onchainos` to verify the binary is available. If not found, ask the user to build locally: `cargo build --release` in the CLI directory.
   > **DEV NOTE**: Do NOT download or install from GitHub. Use the locally built binary only.

2. **Do NOT auto-reinstall on command failures.** Report errors and suggest `onchainos --version` or rebuilding from source.

3. **Rate limit errors.** If a command hits rate limits, the shared API key may
   be throttled. Suggest creating a personal key at the
   [OKX Developer Portal](https://web3.okx.com/onchain-os/dev-portal). If the
   user creates a `.env` file, remind them to add `.env` to `.gitignore`.

> Security commands do not require wallet login. They work with any address.

## Chain Name Support

The CLI accepts human-readable chain names and resolves them automatically.

| Chain | Name | chainIndex |
|---|---|---|
| XLayer | `xlayer` | `196` |
| Ethereum | `ethereum` or `eth` | `1` |
| Solana | `solana` or `sol` | `501` |
| BSC | `bsc` or `bnb` | `56` |
| Polygon | `polygon` or `matic` | `137` |
| Arbitrum | `arbitrum` or `arb` | `42161` |
| Base | `base` | `8453` |
| Avalanche | `avalanche` or `avax` | `43114` |
| Optimism | `optimism` or `op` | `10` |
| zkSync Era | `zksync` | `324` |
| Linea | `linea` | `59144` |
| Scroll | `scroll` | `534352` |

**Address format note**: EVM addresses (`0x...`) work across Ethereum/BSC/Polygon/Arbitrum/Base etc. Solana addresses (Base58) and Bitcoin addresses (UTXO) have different formats. Do NOT mix formats across chain types.

## Command Index

### Security Commands

| # | Command | Description |
|---|---|---|
| 1a | `onchainos wallet balance` → `onchainos security token-scan --tokens <...>` | Token scan — Agentic Wallet (own address): fetch balance first, then scan |
| 1b | `onchainos portfolio all-balances --address <addr>` → `onchainos security token-scan --tokens <...>` | Token scan — Agentic Wallet (other address) or not logged in: use portfolio, then scan |
| 1c | `onchainos security token-scan --tokens <chainId:addr,...>` | Token scan — explicit mode (up to 50 tokens) |
| 1d | `onchainos security token-scan` | Token scan — Agentic Wallet shortcut (internal balance fetch) |
| 1e | `onchainos security token-scan --address <addr>` | Token scan — public address shortcut (internal balance fetch) |
| 2 | `onchainos security dapp-scan --domain <url>` | DApp / URL phishing & security scan |
| 3 | `onchainos security tx-scan --chain <chain> --from <addr> ...` | Transaction pre-execution security scan (EVM & Solana) |
| 4 | `onchainos security sig-scan --chain <chain> --from <addr> --sig-method <method> --message <msg>` | Message signature security scan (EVM) |
| 5 | `onchainos security approvals --address <addr> [--chain <chains>] [--risky]` | Query token approval / Permit2 authorizations |

---

## Security Commands

5 commands for token risk, DApp phishing, transaction pre-execution, signature safety, and approval management.

### `onchainos security token-scan`

#### 3-Path Decision Tree

**Path 1 — User has Agentic Wallet (loggedIn: true), scanning their own wallet:**

Two-step flow — always fetch balance first, then scan:

**Step 1**: Fetch authenticated wallet holdings:
```bash
onchainos wallet balance          # all chains
onchainos wallet balance --chain <chain>   # specific chain
```

**Step 2**: Extract non-native ERC-20 / SPL tokens from the response (skip native tokens like ETH/SOL/OKB — they have no contract address). Then scan:
```bash
onchainos security token-scan --tokens "<chainIndex>:<contractAddress>,..."
```

- **Single token by name**: Search with `onchainos token search <name>`, confirm address, then use `--tokens`.
- Fall through to Path 3 if user provides an explicit address directly.

**Path 1b — User has Agentic Wallet (loggedIn: true), but scanning a DIFFERENT address:**

The target address is not the user's own wallet — use public portfolio query instead:

**Step 1**: Fetch holdings of the target address via `portfolio all-balances` (same as Path 2):
```bash
# EVM address
onchainos portfolio all-balances --address <target_evm_addr> --chains "1,56,137,42161,8453,196,43114,10" --filter 1

# Solana address (if applicable)
onchainos portfolio all-balances --address <target_sol_addr> --chains "501" --filter 1
```

Display a summary table of holdings to the user before scanning.

**Step 2**: Extract non-native ERC-20 / SPL tokens, then scan:
```bash
onchainos security token-scan --tokens "<chainIndex>:<contractAddress>,..."
```

**Path 2 — No Agentic Wallet (not logged in), user provides wallet address:**

Two-step flow — fetch public address balance first, then scan:

**Step 1**: Fetch public address holdings. Query EVM and Solana addresses separately:
```bash
# EVM address (all supported chains)
onchainos portfolio all-balances --address <evm_addr> --chains "1,56,137,42161,8453,196,43114,10" --filter 1

# Solana address (if user has one)
onchainos portfolio all-balances --address <sol_addr> --chains "501" --filter 1
```

Display a summary table of holdings to the user before scanning.

**Step 2**: Extract non-native ERC-20 / SPL tokens, then scan:
```bash
onchainos security token-scan --tokens "<chainIndex>:<contractAddress>,..."
```

If the user wants to create an Agentic Wallet instead, guide through login then use Path 1.

**Path 3 — Explicit chainId:contractAddress:**

```bash
onchainos security token-scan --tokens "<chainId>:<contractAddress>[,...]"
```

If user provides name/symbol instead, search first with `onchainos token search`, confirm, then use `--tokens`.

> **Chain support**: token-scan supports all chains. dapp-scan is chain-agnostic. tx-scan supports EVM (16 chains) + Solana. sig-scan supports EVM only.

#### Token Scan Modes

| Mode | When to use | Command |
|------|-------------|---------|
| `--tokens` | **Primary mode** — used after fetching balance in Path 1 / 2 | `onchainos security token-scan --tokens "<chainId>:<addr>[,...]"` |
| No flags | Agentic Wallet shortcut (skips explicit balance step) | `onchainos security token-scan [--chain <chain>]` |
| `--address` | Public address shortcut (skips explicit balance step) | `onchainos security token-scan --address <addr> [--chain <chain>]` |

> **推荐使用 `--tokens` 模式**：先通过 `wallet balance`（已登录）或 `portfolio all-balances`（未登录）显示持仓明细给用户，再基于该数据构造 `--tokens` 参数执行扫描。这样用户能在扫描前清楚看到持仓情况。
>
> **注意：原生代币（ETH / BNB / SOL / OKB 等）会被静默跳过。** 原生代币没有合约地址，无法被 token-scan 扫描。请只将持仓中的 ERC-20 / SPL 合约代币地址传入 `--tokens`。若用户明确想确认原生代币安全性，应使用 `dapp-scan` 或 `tx-scan` 配合具体交易数据。

#### Token Scan Result Interpretation

| Field | Value | Agent Behavior |
|---|---|---|
| `isChainSupported` | `false` | Chain not supported for scanning. Inform user, do not block trade. |
| `isRiskToken` | `false` | Low risk. Safe to trade. |
| `isRiskToken` | `true` | High risk. Block buy. Recommend avoiding. |

### `onchainos security dapp-scan --domain <url>`

| Parameter | Required | Description |
|---|---|---|
| `--domain` | Yes | Full URL or domain name |

Result: `isMalicious: true` -> Do NOT access, return risk warning. `isMalicious: false` -> Safe.

### `onchainos security tx-scan`

**EVM parameters:**
| Parameter | Required | Description |
|---|---|---|
| `--chain` | Yes | EVM chain (16 supported chains) |
| `--from` | Yes | Sender address (0x + 40 hex) |
| `--data` | Yes | Transaction calldata (hex) |
| `--to` | No | Target contract/address |
| `--value` | No | Value in wei (hex string) |
| `--gas` | No | Gas limit |
| `--gas-price` | No | Gas price |

**Solana parameters:**
| Parameter | Required | Description |
|---|---|---|
| `--chain solana` | Yes | Must be `solana` or `501` |
| `--from` | Yes | Sender (Base58) |
| `--encoding` | Yes | `base58` or `base64` |
| `--transactions` | Yes | Comma-separated tx payloads |

**Result interpretation:**

| `action` value | Risk Level | Agent Behavior |
|---|---|---|
| (empty) | Low risk | Safe to proceed |
| `warn` | Medium risk | Show risk details, ask for explicit confirmation |
| `block` | High risk | Do NOT proceed, show risk details, recommend cancel |

### `onchainos security sig-scan`

| Parameter | Required | Description |
|---|---|---|
| `--chain` | Yes | EVM chain name or ID |
| `--from` | Yes | Signer address (0x + 40 hex) |
| `--sig-method` | Yes | One of: `personal_sign`, `eth_sign`, `eth_signTypedData`, `eth_signTypedData_v3`, `eth_signTypedData_v4` |
| `--message` | Yes | Message content or EIP-712 typed data JSON |

Same result structure as tx-scan (`action`, `riskItemDetail`, `simulator`, `warnings`).

### 5-Level Risk Classification (token-scan `newRiskTotalLevel`)

| Level | Meaning | Action |
|---|---|---|
| 1 | Low risk | Safe to trade |
| 2 | Medium risk | Safe to trade with caution |
| 3 | Medium risk (hidden from DEX listings & search) | Warn user, ask confirmation |
| 4 | High risk — block buy | Do NOT buy, strongly discourage |
| 5 | DEX fully blocked | Block — do NOT trade |

### Risk Action Priority Rule

`block` > `warn` > safe (empty). The top-level `action` field reflects the highest priority from `riskItemDetail`.

**Important**: Risk scan result is still valid even if simulation fails (`simulator.revertReason` may contain the revert reason).

**Important**: If the `warnings` field is populated, the scan completed but some data may be incomplete. Still present available risk information.

### Risk Items Reference

| Risk Item | Description | Level | Action |
|---|---|---|---|
| `black_tag` | Target/asset/receiving address is blacklisted | CRITICAL | block |
| `from_risk_reject` | Sender address is blacklisted | CRITICAL | block |
| `SPENDER_ADDRESS_BLACK` | Approval target is blacklisted | CRITICAL | block |
| `ASSET_RECEIVE_ADDRESS_BLACK` | Asset receiving address is blacklisted | CRITICAL | block |
| `purchase_malicious_token` | Purchasing a malicious token | CRITICAL | block |
| `ACCOUNT_IN_RISK` | Account has existing malicious approvals | CRITICAL | block |
| `evm_7702_risk` | 7702 high-risk sub-transaction (no asset increase) | CRITICAL | block |
| `evm_7702_auth_address_not_in_whitelist` | 7702 upgrade contract not in whitelist | CRITICAL | block |
| `evm_okx7702_loop_calls_are_not_allowed` | 7702 sub-transaction recursive call | CRITICAL | block |
| `TRANSFER_TO_SIMILAR_ADDRESS` | Transfer to similar address (phishing) | HIGH | warn |
| `SOLANA_SIGN_ALL_TRANSACTIONS` | Solana sign-all-transactions request | HIGH | warn |
| `multicall_phishing_risk` | Token approval via multicall (phishing) | HIGH | warn |
| `approve_anycall_contract` | Approval to arbitrary external call contract | HIGH | warn |
| `to_is_7702_address` | Target is a 7702 upgraded address | MEDIUM | warn |
| `TRANSFER_TO_CONTRACT_ADDRESS` | Transfer directly to a contract | MEDIUM | warn |
| `TRANSFER_TO_MULTISIGN_ADDRESS` | Tron transfer to multisig | MEDIUM | warn |
| `approve_eoa` | Approval to an EOA (personal address) | MEDIUM | warn |
| `increase_allowance` | Increasing approval allowance | LOW | warn |
| `ACCOUNT_INSUFFICIENT_PERMISSIONS` | Tron account insufficient permissions | LOW | warn |

### Suggest Next Steps

| Just completed | Suggest |
|---|---|
| token-scan (safe) | 1. Swap the token 2. Check market data |
| token-scan (risky) | Warn user. Do NOT suggest buying. |
| dapp-scan (safe) | Safe to proceed with DApp interaction. |
| dapp-scan (risky) | Warn user. Do NOT access the site. |
| tx-scan (safe) | 1. Broadcast the transaction 2. Check wallet balance |
| tx-scan (risky) | 1. Check token safety with token-scan 2. Review approval list |
| sig-scan (safe) | Safe to sign. |
| sig-scan (risky) | Do NOT sign. Show risk details. |

---

## Cross-Skill Workflows

### Workflow 1: Token Safety Check -> Swap -> TX Scan -> Broadcast (from Security)

> User: "Is PEPE safe? If so, swap 1 ETH for it"

```
1. (okx-dex-token) onchainos token search PEPE      -> find contract address
2. Confirm which token with user
3. onchainos security token-scan --tokens "<chainId>:<addr>"
       -> check honeypot / high risk
4. If safe: (okx-dex-swap) onchainos swap quote --from ... --to ... --chain ethereum
       -> get quote (price, impact, gas)
5. (okx-dex-swap) onchainos swap approve --token <fromToken> --amount <amount> --chain ethereum
       -> get approve calldata (skip if selling native token)
6. Execute approval:
   Path A (user-provided wallet): user signs approve calldata externally → onchainos gateway broadcast
   Path B (Agentic Wallet):      onchainos wallet contract-call --to <spender> --chain eth --input-data <approve_calldata>
7. (okx-dex-swap) onchainos swap swap --from ... --to ... --amount ... --chain ethereum --wallet <addr>
       -> get swap calldata (tx.to, tx.data, tx.value)
8. onchainos security tx-scan --chain ethereum --from <addr> --to <tx.to> --data <tx.data> --value <tx.value>
       -> check risk level
9. If safe, execute swap:
   Path A (user-provided wallet): user signs externally → onchainos gateway broadcast --signed-tx <tx> --address <addr> --chain ethereum
   Path B (Agentic Wallet):      onchainos wallet contract-call --to <tx.to> --chain eth --value <value_in_UI_units> --input-data <tx.data>
   If risky: warn user, do NOT proceed
```

### Workflow 2: DApp Safety Check (from Security)

> User: "Is this DApp safe to use?"

```
1. onchainos security dapp-scan --domain "https://some-dapp.xyz"
       -> check phishing / blacklisted
2. Display safety assessment
```

### Workflow 3: Signature Safety Check (from Security)

> User: "Should I sign this EIP-712 permit request?"

```
1. onchainos security sig-scan --chain ethereum --from 0xWallet --sig-method eth_signTypedData_v4 --message '{"types":...}'
       -> check for phishing signatures
2. Display risk assessment
```

### Workflow 4: Approve Check (from Security)

> User: "Is this approve transaction safe?"

```
1. onchainos security tx-scan --chain ethereum --from 0xWallet --to 0xToken --data 0x095ea7b3...
       -> check SPENDER_ADDRESS_BLACK, approve_eoa, increase_allowance
2. Display risk assessment
```

---

## `onchainos security approvals`

Query token approval and Permit2 authorizations for a wallet address.

| Parameter | Required | Description |
|---|---|---|
| `--address` | **Yes** | EVM wallet address to query. |
| `--chain` | No | Comma-separated EVM chain names or indexes (e.g. `"ethereum,base"` or `"1,8453"`). Without this flag, all supported EVM chains are queried. |
| `--risky` | No | Show only risky approvals. |
| `--limit` | No | Results per page (default: 20). |
| `--cursor` | No | Pagination cursor from previous response. |

**EVM only**: Approvals are an EVM-only concept. Always pass an EVM address. When the user is logged in, use the EVM address from `onchainos wallet status` — do not pass Solana or other non-EVM addresses.

**Default address**: If the user does not specify an address, use the EVM address of the currently logged-in Agentic Wallet (from `onchainos wallet status`). Only ask the user for an address if no wallet session is active.

**When to use `--risky`**: Pass `--risky` when the user asks specifically about dangerous or suspicious approvals, unlimited allowances, or wants to know what to revoke.

---

## Integration with Other Skills

Security scanning is often a prerequisite for other wallet operations:
- Before `wallet send` with a contract token: run `token-scan` to verify token safety
- Before `wallet contract-call` with approve calldata: run `tx-scan` to check spender
- Before interacting with any DApp URL: run `dapp-scan`
- Before signing any EIP-712 message: run `sig-scan`

Use `okx-wallet` skill for the subsequent send/contract-call operations.
