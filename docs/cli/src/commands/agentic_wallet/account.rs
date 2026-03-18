use anyhow::{bail, Result};
use serde_json::json;

use crate::keyring_store;
use crate::output;
use crate::wallet_store::{self, WalletsJson};

use super::auth::{is_session_key_expired_in, is_token_expired};

// ── switch ───────────────────────────────────────────────────────────

/// onchainos wallet switch <account_id>
pub(super) async fn cmd_switch(account_id: &str) -> Result<()> {
    if cfg!(feature = "debug-log") {
        eprintln!("[DEBUG] cmd_switch: account_id={}", account_id);
    }

    if account_id.is_empty() {
        bail!("account_id is required");
    }

    let mut wallets = wallet_store::load_wallets()?
        .ok_or_else(|| anyhow::anyhow!("not logged in: wallets.json not found"))?;

    if cfg!(feature = "debug-log") {
        eprintln!(
            "[DEBUG] cmd_switch: loaded wallets, selected_account_id={}, accounts_map keys={:?}",
            wallets.selected_account_id,
            wallets.accounts_map.keys().collect::<Vec<_>>()
        );
    }

    if !wallets.accounts_map.contains_key(account_id) {
        bail!("account_id not found");
    }

    wallets.selected_account_id = account_id.to_string();
    wallet_store::save_wallets(&wallets)?;

    if cfg!(feature = "debug-log") {
        eprintln!("[DEBUG] cmd_switch: switched to account_id={}", account_id);
    }

    output::success_empty();
    Ok(())
}

// ── status ───────────────────────────────────────────────────────────

/// onchainos wallet status
pub(super) async fn cmd_status() -> Result<()> {
    if cfg!(feature = "debug-log") {
        eprintln!("[DEBUG] cmd_status: start");
    }

    let wallets = match wallet_store::load_wallets()? {
        Some(w) => w,
        None => {
            if cfg!(feature = "debug-log") {
                eprintln!("[DEBUG] cmd_status: wallets.json not found, returning not logged in");
            }
            output::success(json!({
                "email": "",
                "loggedIn": false,
                "currentAccountId": "",
                "currentAccountName": "",
            }));
            return Ok(());
        }
    };

    if cfg!(feature = "debug-log") {
        eprintln!(
            "[DEBUG] cmd_status: loaded wallets, email={}, selected_account_id={}, accounts_count={}",
            wallets.email, wallets.selected_account_id, wallets.accounts.len()
        );
    }

    let blob = keyring_store::read_blob().unwrap_or_default();

    if cfg!(feature = "debug-log") {
        eprintln!(
            "[DEBUG] cmd_status: keyring blob keys={:?}, refresh_token_len={}",
            blob.keys().collect::<Vec<_>>(),
            blob.get("refresh_token").map(|t| t.len()).unwrap_or(0)
        );
    }

    let logged_in = !is_session_key_expired_in(&blob)
        && blob
            .get("refresh_token")
            .map(|t| !t.is_empty() && !is_token_expired(t))
            .unwrap_or(false);

    if cfg!(feature = "debug-log") {
        eprintln!(
            "[DEBUG] cmd_status: session_key_expired={}, logged_in={}",
            is_session_key_expired_in(&blob),
            logged_in
        );
    }

    let current_account_name = wallets
        .accounts
        .iter()
        .find(|a| a.account_id == wallets.selected_account_id)
        .map(|a| a.account_name.clone())
        .unwrap_or_default();

    if cfg!(feature = "debug-log") {
        eprintln!(
            "[DEBUG] cmd_status: result email={}, logged_in={}, account_id={}, account_name={}",
            wallets.email, logged_in, wallets.selected_account_id, current_account_name
        );
    }

    output::success(json!({
        "email": wallets.email,
        "loggedIn": logged_in,
        "currentAccountId": wallets.selected_account_id,
        "currentAccountName": current_account_name,
    }));
    Ok(())
}

// ── resolve_active_account_id ─────────────────────────────────────────

/// Resolve the active account ID: selected_account_id → is_default → first key.
/// `pub` so that sibling modules (balance, history, transfer) and external
/// modules (security) can call it.
pub fn resolve_active_account_id(wallets: &WalletsJson) -> Result<String> {
    if !wallets.selected_account_id.is_empty() {
        return Ok(wallets.selected_account_id.clone());
    }
    if let Some(acct) = wallets.accounts.iter().find(|a| a.is_default) {
        return Ok(acct.account_id.clone());
    }
    wallets
        .accounts_map
        .keys()
        .next()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no wallet accounts found in wallets.json"))
}
