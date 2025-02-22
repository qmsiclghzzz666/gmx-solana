use std::{collections::HashMap, path::PathBuf, rc::Rc};

use eyre::OptionExt;
use solana_remote_wallet::remote_wallet::RemoteWalletManager;
use solana_sdk::signature::{read_keypair_file, Keypair};
use url::Url;

/// Parse url or path.
fn parse_url_or_path(source: &str) -> eyre::Result<Url> {
    let url = match Url::parse(source) {
        Ok(url) => url,
        Err(_) => {
            let path = shellexpand::tilde(source);
            let path: PathBuf = path.parse()?;
            let path = std::fs::canonicalize(path)?;
            Url::from_file_path(&path).expect("must be valid file path")
        }
    };

    Ok(url)
}

/// Load keypair.
pub fn load_keypair(source: &str) -> eyre::Result<Keypair> {
    let url = parse_url_or_path(source)?;

    match url.scheme() {
        "file" => read_keypair_file(url.path()).map_err(|err| eyre::eyre!("{err}")),
        other => {
            eyre::bail!("{other} scheme is not support");
        }
    }
}

/// Load signer from url.
pub fn signer_from_source(
    source: &str,
    confirm_key: bool,
    keypair_name: &str,
    wallet_manager: &mut Option<Rc<RemoteWalletManager>>,
) -> eyre::Result<crate::utils::LocalSignerRef> {
    const QUERY_KEY: &str = "key";

    use crate::utils::local_signer;
    use anchor_client::solana_sdk::derivation_path::DerivationPath;
    use solana_remote_wallet::{
        locator::Locator, remote_keypair::generate_remote_keypair,
        remote_wallet::maybe_wallet_manager,
    };

    let url = parse_url_or_path(source)?;

    match url.scheme() {
        "file" => {
            let keypair = read_keypair_file(url.path()).map_err(|err| eyre::eyre!("{err}"))?;
            Ok(local_signer(keypair))
        }
        "usb" => {
            let manufacturer = url.host_str().ok_or_eyre("missing manufacturer")?;
            let path = url.path();
            let path = path.strip_prefix('/').unwrap_or(path);
            let pubkey = (!path.is_empty()).then_some(path);
            let locator = Locator::new_from_parts(manufacturer, pubkey)?;
            let query = url.query_pairs().collect::<HashMap<_, _>>();
            if query.len() > 1 {
                eyre::bail!("invalid query string, extra fields not supported");
            }
            let derivation_path = query
                .get(QUERY_KEY)
                .map(|value| DerivationPath::from_key_str(value))
                .transpose()?;
            if wallet_manager.is_none() {
                *wallet_manager = maybe_wallet_manager()?;
            }
            let wallet_manager = wallet_manager.as_ref().ok_or_eyre("no device found")?;
            let keypair = generate_remote_keypair(
                locator,
                derivation_path.unwrap_or_default(),
                wallet_manager,
                confirm_key,
                keypair_name,
            )?;
            Ok(local_signer(keypair))
        }
        scheme => Err(eyre::eyre!("unsupported scheme: {scheme}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_or_path() -> eyre::Result<()> {
        let path = "~/.config/solana/id.json";
        assert!(parse_url_or_path(path).is_ok());
        Ok(())
    }
}
