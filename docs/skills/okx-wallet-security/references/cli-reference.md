# Onchain OS Security — CLI Reference

Parameter tables, return schemas, and examples for all security commands.

---

## Security Commands (4 commands)

### `onchainos security token-scan`

Batch security scan for tokens. Detects honeypots, high buy/sell tax, mint risk, and other token-level risks. Three modes (mutually compatible with `--chain`):

| Mode | When to use | Command |
|---|---|---|
| No flags | Has Agentic Wallet, scan all tokens | `onchainos security token-scan [--chain <chain>]` |
| `--address` | No wallet, user provides an address | `onchainos security token-scan --address <addr> [--chain <chain>]` |
| `--tokens` | User provides explicit contract address(es) | `onchainos security token-scan --tokens "<chainId>:<addr>[,...]"` |

**底层机制**：三种模式最终调用同一接口 `/api/v6/security/token-scan`，请求体均为 `{tokenList: [{chainId, contractAddress}]}`。`--address` 和 no-flags 模式先通过余额 API 查询持仓的合约地址列表，再批量扫描（每批最多 50 个，并发执行）；`--tokens` 模式直接传入合约地址，跳过查询步骤。**原生代币（ETH/BNB/SOL/OKB 等）在所有模式下均被跳过**，因为其 `tokenContractAddress` 为空，无法扫描。

**Parameters (explicit mode):**

| Param | Required | Description |
|---|---|---|
| `--tokens` | Yes | Comma-separated `chainId:contractAddress` pairs (max 50). Chain can be name or ID (e.g. `ethereum:0x...` or `1:0x...`) |

**Return fields:**

| Field | Type | Description |
|---|---|---|
| `chainId` | String | Chain ID |
| `tokenAddress` | String | Token contract address |
| `isChainSupported` | Boolean | Whether the chain supports security scanning |
| `buyTaxes` | String | Buy tax percentage |
| `sellTaxes` | String | Sell tax percentage |
| `isRiskToken` | Boolean | Whether the token is high-risk (newRiskTotalLevel=4) |
| `newRiskTotalLevel` | Integer | Risk level 1-5 (see table below) |

**Risk levels (`newRiskTotalLevel`):**

| Value | Meaning |
|---|---|
| 1 | Low risk |
| 2 | Medium risk |
| 3 | Medium risk (hidden from DEX listings & search) |
| 4 | High risk — block buy |
| 5 | DEX fully blocked |

### `onchainos security dapp-scan`

Check a URL or domain for phishing, blacklisting, and other security risks.

```bash
onchainos security dapp-scan --domain <url_or_domain>
```

| Param | Required | Description |
|---|---|---|
| `--domain` | Yes | Full URL or domain name to check (e.g. `https://app.uniswap.org` or `uniswap.org`) |

**Return fields:**

| Field | Type | Description |
|---|---|---|
| `isMalicious` | Boolean | Whether the URL/domain is malicious |

**Agent behavior by result:**

| isMalicious | Agent Behavior |
|---|---|
| false | Allow access |
| true | Refuse access, return risk warning |

### `onchainos security tx-scan`

Pre-execution security scan for a transaction. Automatically routes to EVM or Solana endpoint based on `--chain`.

**EVM usage:**

```bash
onchainos security tx-scan \
  --chain <chain> \
  --from <address> \
  --data <calldata_hex> \
  [--to <address>] \
  [--value <hex_wei>] \
  [--gas <number>] \
  [--gas-price <number>]
```

**Solana usage:**

```bash
onchainos security tx-scan \
  --chain solana \
  --from <base58_address> \
  --encoding <base58|base64> \
  --transactions <payload1,payload2,...>
```

**EVM Parameters:**

| Param | Required | Default | Description |
|---|---|---|---|
| `--chain` | Yes | - | EVM chain name or ID (e.g. `ethereum`, `1`, `bsc`, `56`) |
| `--from` | Yes | - | Sender address (0x + 40 hex chars) |
| `--data` | Yes | - | Transaction calldata (hex-encoded) |
| `--to` | No | - | Target contract or recipient address |
| `--value` | No | - | Transaction value in wei (hex string, e.g. `0xde0b6b3a7640000`) |
| `--gas` | No | - | Gas limit |
| `--gas-price` | No | - | Gas price |

**Solana Parameters:**

| Param | Required | Default | Description |
|---|---|---|---|
| `--chain` | Yes | - | Must be `solana` or `501` |
| `--from` | Yes | - | Sender address (Base58, 32-44 chars) |
| `--encoding` | Yes | - | Encoding format: `base58` or `base64` |
| `--transactions` | Yes | - | Comma-separated transaction payloads |

**Return fields (shared by tx-scan and sig-scan):**

| Field | Type | Description |
|---|---|---|
| `action` | String | Risk action: `""` (safe), `"warn"` (medium risk), `"block"` (high risk) |
| `riskItemDetail` | Array | List of detected risk items |
| `riskItemDetail[].name` | String | Risk item identifier (e.g. `black_tag`, `approve_eoa`) |
| `riskItemDetail[].description` | Map | Risk description (multi-language) |
| `riskItemDetail[].reason` | Array | List of trigger reasons |
| `riskItemDetail[].action` | String | `"block"` or `"warn"` |
| `simulator` | Object | Transaction simulation result |
| `simulator.gasLimit` | Integer | Estimated gas (EVM: `gasLimit`, Solana: `gasUsed`) |
| `simulator.revertReason` | String | Revert reason if simulation failed (null if success) |
| `warnings` | Array | Warning messages (result is still usable but may be incomplete) |

### `onchainos security sig-scan`

Security scan for message signing requests. Detects phishing EIP-712 permits, malicious personal_sign requests, and other signature-based attacks.

```bash
onchainos security sig-scan \
  --chain <chain> \
  --from <address> \
  --sig-method <personal_sign|eth_sign|eth_signTypedData|eth_signTypedData_v3|eth_signTypedData_v4> \
  --message <message_or_typed_data_json>
```

| Param | Required | Description |
|---|---|---|
| `--chain` | Yes | EVM chain name or ID (e.g. `ethereum`, `1`, `bsc`, `56`) |
| `--from` | Yes | Signer address (0x + 40 hex chars) |
| `--sig-method` | Yes | Signing method: `personal_sign`, `eth_sign`, `eth_signTypedData`, `eth_signTypedData_v3`, or `eth_signTypedData_v4` |
| `--message` | Yes | Message content (string for personal_sign / eth_sign) or EIP-712 typed data JSON (for eth_signTypedData variants) |

**Return fields:** Same as tx-scan (`action`, `riskItemDetail`, `simulator`, `warnings`).

### Risk Items Reference

| Risk Item | Description | Level | Action |
|---|---|---|---|
| `black_tag` | Target / asset / receiving address is blacklisted | CRITICAL | block |
| `from_risk_reject` | Sender address is blacklisted | CRITICAL | block |
| `SPENDER_ADDRESS_BLACK` | Approval target is blacklisted | CRITICAL | block |
| `ASSET_RECEIVE_ADDRESS_BLACK` | Asset receiving address is blacklisted | CRITICAL | block |
| `purchase_malicious_token` | Purchasing a malicious token | CRITICAL | block |
| `ACCOUNT_IN_RISK` | Account has existing malicious approvals | CRITICAL | block |
| `evm_7702_risk` | 7702 high-risk sub-transaction (no asset increase) | CRITICAL | block |
| `evm_7702_auth_address_not_in_whitelist` | 7702 upgrade contract not in whitelist | CRITICAL | block |
| `evm_okx7702_loop_calls_are_not_allowed` | 7702 sub-transaction recursive call | CRITICAL | block |
| `TRANSFER_TO_SIMILAR_ADDRESS` | Transfer to a highly similar address (phishing) | HIGH | warn |
| `SOLANA_SIGN_ALL_TRANSACTIONS` | Solana sign-all-transactions request | HIGH | warn |
| `multicall_phishing_risk` | Token approval via multicall (phishing) | HIGH | warn |
| `approve_anycall_contract` | Approval to arbitrary external call contract | HIGH | warn |
| `to_is_7702_address` | Target address is a 7702 upgraded address | MEDIUM | warn |
| `TRANSFER_TO_CONTRACT_ADDRESS` | Transfer directly to a contract address | MEDIUM | warn |
| `TRANSFER_TO_MULTISIGN_ADDRESS` | Tron transfer to a multisig address | MEDIUM | warn |
| `approve_eoa` | Approval to an EOA (personal address) | MEDIUM | warn |
| `increase_allowance` | Increasing approval allowance | LOW | warn |
| `ACCOUNT_INSUFFICIENT_PERMISSIONS` | Tron account has insufficient permissions | LOW | warn |

### G — Input / Output Examples

**User says:** "Is PEPE safe to buy?" (token name, no address)

```
Agent workflow:
1. Search:  onchainos token search PEPE
   -> Returns multiple results across chains
2. Ask user: "I found these tokens matching 'PEPE':
   1. PEPE on Ethereum (0x6982508145454Ce325dDbE47a25d4ec3d2311933)
   2. PEPE on BSC (0x25d887Ce7a35172C62FeBFD67a1856F20FaEbB00)
   Which one do you want to check?"
3. User confirms: "The first one"
4. Scan:   onchainos security token-scan --tokens "1:0x6982508145454Ce325dDbE47a25d4ec3d2311933"
5. Display:
   Token: PEPE on Ethereum
   Risk Level: LOW (1)
   Buy Tax: 0%, Sell Tax: 0%
   Verdict: Safe to trade.
```

**User says:** "Is this token safe to buy?" (provides address directly)

```bash
onchainos security token-scan --tokens "1:0xdAC17F958D2ee523a2206206994597C13D831ec7"
# -> Display:
#   Token: USDT on Ethereum
#   Risk Level: LOW (1)
#   Buy Tax: 0%, Sell Tax: 0%
#   Verdict: Safe to trade.
```

**User says:** "Check if this DApp URL is safe"

```bash
onchainos security dapp-scan --domain "https://suspicious-defi.xyz"
# -> Display:
#   URL: https://suspicious-defi.xyz
#   Result: MALICIOUS
#   Recommendation: Do NOT access this site. It has been flagged as a phishing/scam domain.
```

**User says:** "Check if this approve transaction is safe on Ethereum"

```bash
onchainos security tx-scan --chain ethereum --from 0xabc123... --to 0xTokenContract --data 0x095ea7b3000000000000000000000000def456...00000000000000000000000000000000000000000000000000000000ffffffff
# -> Display:
#   Risk Level: HIGH (block)
#   Risk: SPENDER_ADDRESS_BLACK - The approval target address is on the blacklist
#   Recommendation: Do NOT approve this transaction. The spender address has been flagged as malicious.
```

**User says:** "Is this signing request safe?"

```bash
onchainos security sig-scan --chain ethereum --from 0xMyWallet --sig-method eth_signTypedData_v4 --message '{"types":{"EIP712Domain":[{"name":"name","type":"string"}],"Permit":[{"name":"owner","type":"address"},{"name":"spender","type":"address"},{"name":"value","type":"uint256"}]},"primaryType":"Permit","domain":{"name":"USDC"},"message":{"owner":"0xMyWallet","spender":"0xMalicious","value":"115792089237316195423570985008687907853269984665640564039457584007913129639935"}}'
# -> Display:
#   Risk Level: HIGH (block)
#   Risk: SPENDER_ADDRESS_BLACK - The permit spender address is blacklisted
#   Recommendation: Do NOT sign this request. The spender is flagged as malicious.
```

**User says:** "Scan this Solana transaction"

```bash
onchainos security tx-scan --chain solana --from EeBCkp5j17U5Fg4bEiboHvRrUvQ4LP9AdioQwPg5wF43 --encoding base64 --transactions "CAurZp2HY+l9yM1By3HbAABCA=="
# -> Display:
#   Risk Level: LOW (safe)
#   No risks detected. Transaction simulation successful.
#   Estimated gas: 5000
```

### `onchainos security approvals`

Query token approval and Permit2 authorizations for a wallet address. Returns all active ERC-20 allowances and Permit2 authorizations across supported chains.

```bash
onchainos security approvals \
  --address <wallet_address> \
  [--chain <chains>] \
  [--risky] \
  [--limit <n>] \
  [--cursor <cursor>]
```

| Param | Required | Description |
|---|---|---|
| `--address` | Yes | Wallet address to query (EVM `0x…`) |
| `--chain` | No | Comma-separated chain names or indexes (e.g. `"ethereum,base"` or `"1,8453"`). Omit to query all supported chains. |
| `--risky` | No | Show only risky/high-risk approvals |
| `--limit` | No | Results per page (default: 20) |
| `--cursor` | No | Pagination cursor from previous response |

**Return fields:**

| Field | Type | Description |
|---|---|---|
| `approvalList` | Array | List of approval entries |
| `approvalList[].tokenSymbol` | String | Token symbol (e.g. `USDC`) |
| `approvalList[].tokenAddress` | String | Token contract address |
| `approvalList[].chainIndex` | String | Chain index |
| `approvalList[].spenderAddress` | String | Address that holds the allowance |
| `approvalList[].allowance` | String | Approved amount (raw, `"unlimited"` if max uint256) |
| `approvalList[].riskLevel` | String | Risk level of the approval |
| `cursor` | String | Pagination cursor for next page |

**User says:** "Show my token approvals on Ethereum"

```bash
onchainos security approvals --address 0xMyWallet --chain ethereum
# -> Display:
#   Chain: Ethereum
#   USDC: approved unlimited to 0xSpender1 (LOW risk)
#   WETH: approved 1000 to 0xSpender2 (LOW risk)
```

**User says:** "Show me risky approvals across all my chains"

```bash
onchainos security approvals --address 0xMyWallet --risky
# -> Display:
#   Chain: Ethereum
#   USDT: approved unlimited to 0xMaliciousSpender (HIGH risk — SPENDER_ADDRESS_BLACK)
#   Recommendation: Revoke this approval immediately.
```
