use anyhow::Result;
use clap::Subcommand;
use serde_json::Value;

use super::Context;
use crate::client::ApiClient;
use crate::output;

/// All aggregator endpoints are GET requests.
#[derive(Subcommand)]
pub enum SwapCommand {
    /// Get swap quote (read-only price estimate)
    Quote {
        /// Source token contract address
        #[arg(long)]
        from: String,
        /// Destination token contract address
        #[arg(long)]
        to: String,
        /// Amount in minimal units (wei/lamports)
        #[arg(long)]
        amount: String,
        /// Chain (e.g. ethereum, solana, xlayer)
        #[arg(long)]
        chain: String,
        /// Slippage tolerance in percent (e.g. "1" for 1%). Omit to use autoSlippage.
        #[arg(long)]
        slippage: Option<String>,
        /// Swap mode: exactIn or exactOut
        #[arg(long, default_value = "exactIn")]
        swap_mode: String,
    },
    /// Get swap transaction data (quote → sign → broadcast)
    Swap {
        /// Source token contract address
        #[arg(long)]
        from: String,
        /// Destination token contract address
        #[arg(long)]
        to: String,
        /// Amount in minimal units
        #[arg(long)]
        amount: String,
        /// Chain
        #[arg(long)]
        chain: String,
        /// Slippage tolerance in percent (e.g. "1" for 1%). Omit to use autoSlippage.
        #[arg(long)]
        slippage: Option<String>,
        /// User wallet address
        #[arg(long)]
        wallet: String,
        /// Gas priority: slow, average, fast (default: average)
        #[arg(long, default_value = "average")]
        gas_level: String,
        /// Swap mode: exactIn or exactOut
        #[arg(long, default_value = "exactIn")]
        swap_mode: String,
    },
    /// Get ERC-20 approval transaction data
    Approve {
        /// Token contract address to approve
        #[arg(long)]
        token: String,
        /// Approval amount in minimal units
        #[arg(long)]
        amount: String,
        /// Chain
        #[arg(long)]
        chain: String,
    },
    /// Get supported chains for DEX aggregator
    Chains,
    /// Get available liquidity sources on a chain
    Liquidity {
        /// Chain
        #[arg(long)]
        chain: String,
    },
}

pub async fn execute(ctx: &Context, cmd: SwapCommand) -> Result<()> {
    let client = ctx.client()?;
    match cmd {
        SwapCommand::Quote {
            from,
            to,
            amount,
            chain,
            slippage,
            swap_mode,
        } => {
            let chain_index = crate::chains::resolve_chain(&chain);
            output::success(
                fetch_quote(
                    &client,
                    &chain_index,
                    &from,
                    &to,
                    &amount,
                    &swap_mode,
                    slippage.as_deref(),
                )
                .await?,
            );
        }
        SwapCommand::Swap {
            from,
            to,
            amount,
            chain,
            slippage,
            wallet,
            gas_level,
            swap_mode,
        } => {
            let chain_index = crate::chains::resolve_chain(&chain);
            output::success(
                fetch_swap(
                    &client,
                    &chain_index,
                    &from,
                    &to,
                    &amount,
                    slippage.as_deref(),
                    &wallet,
                    &swap_mode,
                    &gas_level,
                )
                .await?,
            );
        }
        SwapCommand::Approve {
            token,
            amount,
            chain,
        } => {
            let chain_index = crate::chains::resolve_chain(&chain);
            output::success(fetch_approve(&client, &chain_index, &token, &amount).await?);
        }
        SwapCommand::Chains => {
            output::success(fetch_chains(&client).await?);
        }
        SwapCommand::Liquidity { chain } => {
            let chain_index = crate::chains::resolve_chain(&chain);
            output::success(fetch_liquidity(&client, &chain_index).await?);
        }
    }
    Ok(())
}

/// GET /api/v6/dex/aggregator/quote
pub async fn fetch_quote(
    client: &ApiClient,
    chain_index: &str,
    from: &str,
    to: &str,
    amount: &str,
    swap_mode: &str,
    slippage: Option<&str>,
) -> Result<Value> {
    let mut params = vec![
        ("chainIndex", chain_index),
        ("fromTokenAddress", from),
        ("toTokenAddress", to),
        ("amount", amount),
        ("swapMode", swap_mode),
    ];
    if let Some(s) = slippage {
        params.push(("slippagePercent", s));
    } else {
        params.push(("autoSlippage", "true"));
    }
    client.get("/api/v6/dex/aggregator/quote", &params).await
}

/// GET /api/v6/dex/aggregator/swap
#[allow(clippy::too_many_arguments)]
pub async fn fetch_swap(
    client: &ApiClient,
    chain_index: &str,
    from: &str,
    to: &str,
    amount: &str,
    slippage: Option<&str>,
    wallet: &str,
    swap_mode: &str,
    gas_level: &str,
) -> Result<Value> {
    let mut params = vec![
        ("chainIndex", chain_index),
        ("fromTokenAddress", from),
        ("toTokenAddress", to),
        ("amount", amount),
        ("userWalletAddress", wallet),
        ("swapMode", swap_mode),
        ("gasLevel", gas_level),
    ];
    if let Some(s) = slippage {
        params.push(("slippagePercent", s));
    } else {
        params.push(("autoSlippage", "true"));
    }
    client.get("/api/v6/dex/aggregator/swap", &params).await
}

/// GET /api/v6/dex/aggregator/approve-transaction
pub async fn fetch_approve(
    client: &ApiClient,
    chain_index: &str,
    token: &str,
    amount: &str,
) -> Result<Value> {
    client
        .get(
            "/api/v6/dex/aggregator/approve-transaction",
            &[
                ("chainIndex", chain_index),
                ("tokenContractAddress", token),
                ("approveAmount", amount),
            ],
        )
        .await
}

/// GET /api/v6/dex/aggregator/supported/chain
pub async fn fetch_chains(client: &ApiClient) -> Result<Value> {
    client
        .get("/api/v6/dex/aggregator/supported/chain", &[])
        .await
}

/// GET /api/v6/dex/aggregator/get-liquidity
pub async fn fetch_liquidity(client: &ApiClient, chain_index: &str) -> Result<Value> {
    client
        .get(
            "/api/v6/dex/aggregator/get-liquidity",
            &[("chainIndex", chain_index)],
        )
        .await
}
