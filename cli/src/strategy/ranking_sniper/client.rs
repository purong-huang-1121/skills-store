//! SniperClient — OKX DEX API calls for token data + swap execution on Solana.
//!
//! All network traffic goes through OKX HTTP API (ApiClient).
//! Swap flow: get unsigned tx from OKX → sign locally → broadcast via OKX.
//! No direct Solana RPC calls.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use super::engine::{safe_float, CHAIN_INDEX, SLIPPAGE_PCT, SOL_DECIMALS, SOL_NATIVE};
use crate::client::ApiClient;

pub struct SniperClient {
    api: ApiClient,
    pub wallet: String,
}

pub struct SwapResult {
    pub tx_hash: Option<String>,
    pub amount_out: f64,
    pub raw_response: Value,
}

impl SniperClient {
    /// Create a fully authenticated client.
    /// Requires SOL_ADDRESS + OKX API keys (via ApiClient).
    /// SOL_PRIVATE_KEY is only needed for live swap execution.
    pub fn new() -> Result<Self> {
        let api = ApiClient::new(None)?;
        let wallet = std::env::var("SOL_ADDRESS")
            .context("SOL_ADDRESS not set — required for ranking sniper")?;
        Ok(Self { api, wallet })
    }

    /// Create client for read-only operations (no wallet needed for data queries).
    pub fn new_read_only() -> Result<Self> {
        let api = ApiClient::new(None)?;
        let wallet = std::env::var("SOL_ADDRESS").unwrap_or_default();
        Ok(Self { api, wallet })
    }

    // ── Data queries ────────────────────────────────────────────────

    /// Fetch Solana top tokens by 24h price change (trending).
    pub async fn fetch_ranking(&self, top_n: usize) -> Result<Vec<Value>> {
        let data = self
            .api
            .get(
                "/api/v6/dex/market/token/toplist",
                &[
                    ("chains", CHAIN_INDEX),
                    ("sortBy", "2"),    // sort by price change
                    ("timeFrame", "1"), // 5 minutes
                ],
            )
            .await?;

        let tokens = match data {
            Value::Array(arr) => arr,
            _ => data.as_array().cloned().unwrap_or_default(),
        };

        Ok(tokens.into_iter().take(top_n).collect())
    }

    /// Fetch advanced token info for safety checks.
    pub async fn fetch_advanced_info(&self, token_addr: &str) -> Result<Value> {
        let data = self
            .api
            .get(
                "/api/v6/dex/market/token/advanced-info",
                &[
                    ("tokenContractAddress", token_addr),
                    ("chainIndex", CHAIN_INDEX),
                ],
            )
            .await?;

        match data {
            Value::Array(arr) if !arr.is_empty() => Ok(arr[0].clone()),
            Value::Object(_) => Ok(data),
            _ => bail!("unexpected advanced-info response format"),
        }
    }

    /// Fetch current token price in USD.
    pub async fn fetch_price(&self, token_addr: &str) -> Result<f64> {
        let body = json!([{
            "tokenContractAddress": token_addr,
            "chainIndex": CHAIN_INDEX,
        }]);
        let data = self
            .api
            .post("/api/v6/dex/market/price-info", &body)
            .await?;

        let item = match &data {
            Value::Array(arr) if !arr.is_empty() => &arr[0],
            _ => &data,
        };

        let price = safe_float(&item["price"], 0.0);
        if price <= 0.0 {
            bail!("invalid price for {token_addr}");
        }
        Ok(price)
    }

    /// Fetch holder data filtered by tag (6=Suspicious, 8=Phishing).
    pub async fn fetch_holder_risk(
        &self,
        token_addr: &str,
        tag_filter: &str,
    ) -> Result<Vec<Value>> {
        let data = self
            .api
            .get(
                "/api/v6/dex/market/token/holder",
                &[
                    ("tokenContractAddress", token_addr),
                    ("chainIndex", CHAIN_INDEX),
                    ("tagFilter", tag_filter),
                ],
            )
            .await?;

        match data {
            Value::Array(arr) => Ok(arr),
            _ => Ok(data.as_array().cloned().unwrap_or_default()),
        }
    }

    // ── Swap execution ──────────────────────────────────────────────

    /// Execute a swap via OKX DEX aggregator on Solana.
    /// Flow: get unsigned tx from OKX → sign locally → broadcast via OKX.
    pub async fn execute_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount_raw: &str,
    ) -> Result<SwapResult> {
        if self.wallet.is_empty() {
            bail!("SOL_ADDRESS not set — cannot execute swap");
        }

        // Step 1: Get swap transaction from OKX API
        let data = self
            .api
            .get(
                "/api/v6/dex/aggregator/swap",
                &[
                    ("chainIndex", CHAIN_INDEX),
                    ("fromTokenAddress", from_token),
                    ("toTokenAddress", to_token),
                    ("amount", amount_raw),
                    ("slippagePercent", SLIPPAGE_PCT),
                    ("userWalletAddress", &self.wallet),
                ],
            )
            .await?;

        let swap_data = match &data {
            Value::Array(arr) if !arr.is_empty() => arr[0].clone(),
            _ => data,
        };

        let amount_out = safe_float(&swap_data["routerResult"]["toTokenAmount"], 0.0);

        // Step 2: Extract the unsigned transaction (base58-encoded)
        let tx_data_b58 = swap_data["tx"]["data"]
            .as_str()
            .context("missing tx.data in swap response")?;

        // Step 3: Sign and broadcast via OKX
        let tx_hash = self.sign_and_broadcast(tx_data_b58).await?;

        Ok(SwapResult {
            tx_hash: Some(tx_hash),
            amount_out,
            raw_response: swap_data,
        })
    }

    /// Sign a base58-encoded Solana transaction and broadcast via OKX broadcast API.
    async fn sign_and_broadcast(&self, tx_data_b58: &str) -> Result<String> {
        let pk_b58 = std::env::var("SOL_PRIVATE_KEY")
            .context("SOL_PRIVATE_KEY not set — required for swap execution")?;

        let pk_bytes = bs58::decode(&pk_b58)
            .into_vec()
            .context("invalid SOL_PRIVATE_KEY (not valid base58)")?;

        let signing_key = if pk_bytes.len() == 64 {
            ed25519_dalek::SigningKey::from_keypair_bytes(
                pk_bytes
                    .as_slice()
                    .try_into()
                    .context("invalid keypair length")?,
            )
            .context("invalid ed25519 keypair")?
        } else if pk_bytes.len() == 32 {
            ed25519_dalek::SigningKey::from_bytes(
                pk_bytes
                    .as_slice()
                    .try_into()
                    .context("invalid key length")?,
            )
        } else {
            bail!(
                "SOL_PRIVATE_KEY must be 32 or 64 bytes (got {})",
                pk_bytes.len()
            );
        };

        let tx_bytes = bs58::decode(tx_data_b58)
            .into_vec()
            .context("invalid base58 transaction data")?;

        // Sign with OKX's blockhash (already recent)
        let signed = sign_solana_transaction(&tx_bytes, &signing_key)?;
        let signed_b58 = bs58::encode(&signed).into_string();

        // Broadcast via OKX
        let body = json!({
            "chainIndex": CHAIN_INDEX,
            "signedTx": signed_b58,
            "address": self.wallet,
        });
        let data = self
            .api
            .post("/api/v6/dex/pre-transaction/broadcast-transaction", &body)
            .await?;

        let result = match &data {
            Value::Array(arr) if !arr.is_empty() => arr[0].clone(),
            _ => data,
        };

        // If we got a direct txHash, return it
        if let Some(hash) = result["txHash"]
            .as_str()
            .or_else(|| result["orderHash"].as_str())
            .or_else(|| result["hash"].as_str())
        {
            if !hash.is_empty() {
                return Ok(hash.to_string());
            }
        }

        // Otherwise poll OKX order status for txHash
        let order_id = result["orderId"]
            .as_str()
            .context("broadcast returned neither txHash nor orderId")?
            .to_string();

        eprintln!("[broadcast] polling orderId: {}", order_id);
        for attempt in 0..20 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
            if let Ok(data) = self
                .api
                .get(
                    "/api/v6/dex/post-transaction/orders",
                    &[
                        ("address", &*self.wallet),
                        ("chainIndex", CHAIN_INDEX),
                        ("orderId", &order_id),
                    ],
                )
                .await
            {
                let orders = match &data {
                    Value::Array(arr) => arr.clone(),
                    _ => data["orders"].as_array().cloned().unwrap_or_default(),
                };
                for order in &orders {
                    let tx_status = order["txStatus"].as_str().unwrap_or("");
                    let confirmed_hash = order["txHash"].as_str().unwrap_or("");
                    if attempt % 5 == 0 {
                        eprintln!(
                            "[broadcast] poll {}: status={} txHash={}",
                            attempt, tx_status, confirmed_hash
                        );
                    }
                    if tx_status == "2" && !confirmed_hash.is_empty() {
                        return Ok(confirmed_hash.to_string());
                    }
                    if tx_status == "3" {
                        let reason = order["failReason"].as_str().unwrap_or("unknown");
                        bail!("transaction failed on-chain: {}", reason);
                    }
                }
            }
        }

        bail!("transaction not confirmed after 60s (orderId={})", order_id)
    }

    /// Buy a token with SOL.
    pub async fn buy_token(&self, token_addr: &str, sol_amount: f64) -> Result<SwapResult> {
        let amount_raw = format!("{}", (sol_amount * 10f64.powi(SOL_DECIMALS as i32)) as u64);
        self.execute_swap(SOL_NATIVE, token_addr, &amount_raw).await
    }

    /// Sell a token for SOL. `amount_raw` is the raw token amount.
    pub async fn sell_token(&self, token_addr: &str, amount_raw: &str) -> Result<SwapResult> {
        self.execute_swap(token_addr, SOL_NATIVE, amount_raw).await
    }
}

/// Sign a Solana serialized transaction.
///
/// Solana wire format:
///   [compact-u16 num_signatures] [64-byte signature × num_signatures] [message...]
///
/// The OKX API returns a transaction with placeholder (zero) signatures.
/// We replace the first signature with our ed25519 signature over the message.
fn sign_solana_transaction(
    tx_bytes: &[u8],
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<Vec<u8>> {
    use ed25519_dalek::Signer;

    if tx_bytes.is_empty() {
        bail!("empty transaction data");
    }

    let (num_sigs, offset) = decode_compact_u16(tx_bytes)?;
    if num_sigs == 0 {
        bail!("transaction has 0 signatures slots");
    }

    let sigs_end = offset + (num_sigs as usize) * 64;
    if sigs_end > tx_bytes.len() {
        bail!("transaction too short for {} signatures", num_sigs);
    }

    let message = &tx_bytes[sigs_end..];
    let signature = signing_key.sign(message);

    let mut signed = Vec::with_capacity(tx_bytes.len());
    signed.extend_from_slice(&tx_bytes[..offset]);
    signed.extend_from_slice(&signature.to_bytes());
    if num_sigs > 1 {
        signed.extend_from_slice(&tx_bytes[offset + 64..sigs_end]);
    }
    signed.extend_from_slice(message);

    Ok(signed)
}

/// Decode a Solana compact-u16 from a byte slice.
/// Returns (value, bytes_consumed).
fn decode_compact_u16(data: &[u8]) -> Result<(u16, usize)> {
    if data.is_empty() {
        bail!("empty data for compact-u16");
    }
    let first = data[0] as u16;
    if first < 0x80 {
        return Ok((first, 1));
    }
    if data.len() < 2 {
        bail!("truncated compact-u16");
    }
    let second = data[1] as u16;
    if second < 0x80 {
        return Ok(((first & 0x7f) | (second << 7), 2));
    }
    if data.len() < 3 {
        bail!("truncated compact-u16");
    }
    let third = data[2] as u16;
    Ok(((first & 0x7f) | ((second & 0x7f) << 7) | (third << 14), 3))
}
