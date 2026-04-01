extern crate alloc;

use crate::bip141::try_strip_bip141;
use alloc::vec::Vec;
use bitcoin::hashes::{sha256d::Hash as DHash, Hash};
use tracing::error;

/// Computes the Merkle root from coinbase transaction components and a path of transaction hashes.
///
/// Validates and deserializes a coinbase transaction before building the 32-byte Merkle root.
/// Returns [`None`] is the arguments are invalid.
///
/// ## Components
/// * `coinbase_tx_prefix`: First part of the coinbase transaction (the part before the extranonce).
///   Should be converted from [`binary_sv2::B064K`].
/// * `coinbase_tx_suffix`: Coinbase transaction suffix (the part after the extranonce). Should be
///   converted from [`binary_sv2::B064K`].
/// * `extranonce`: Extra nonce space. Should be converted from [`binary_sv2::B032`] and padded with
///   zeros if not `32` bytes long.
/// * `path`: List of transaction hashes. Should be converted from [`binary_sv2::U256`].
pub fn merkle_root_from_path<T: AsRef<[u8]>>(
    coinbase_tx_prefix: &[u8],
    coinbase_tx_suffix: &[u8],
    extranonce: &[u8],
    path: &[T],
) -> Option<Vec<u8>> {
    let (coinbase_tx_prefix, coinbase_tx_suffix) =
        match try_strip_bip141(coinbase_tx_prefix, coinbase_tx_suffix) {
            Ok(Some((coinbase_tx_prefix_stripped, coinbase_tx_suffix_stripped))) => {
                (coinbase_tx_prefix_stripped, coinbase_tx_suffix_stripped)
            }
            Ok(None) => (coinbase_tx_prefix.to_vec(), coinbase_tx_suffix.to_vec()),
            Err(e) => {
                error!("ERROR: {:?}", e);
                return None;
            }
        };

    let mut coinbase =
        Vec::with_capacity(coinbase_tx_prefix.len() + coinbase_tx_suffix.len() + extranonce.len());
    coinbase.extend_from_slice(&coinbase_tx_prefix);
    coinbase.extend_from_slice(extranonce);
    coinbase.extend_from_slice(&coinbase_tx_suffix);

    // sha256d hash of the raw coinbase transaction
    let coinbase_txid: [u8; 32] = *DHash::hash(&coinbase).as_ref();

    Some(merkle_root_from_path_(coinbase_txid, path).to_vec())
}

/// Computes the Merkle root from a validated coinbase transaction and a path of transaction
/// hashes.
///
/// If the `path` is empty, the coinbase transaction hash (`coinbase_id`) is returned as the root.
///
/// ## Components
/// * `coinbase_id`: Coinbase transaction hash.
/// * `path`: List of transaction hashes. Should be converted from [`binary_sv2::U256`].
pub fn merkle_root_from_path_<T: AsRef<[u8]>>(coinbase_id: [u8; 32], path: &[T]) -> [u8; 32] {
    match path.len() {
        0 => coinbase_id,
        _ => reduce_path(coinbase_id, path),
    }
}

// Computes the Merkle root by iteratively combining the coinbase transaction hash with each
// transaction hash in the `path`.
//
// Handles the core logic of combining hashes using the Bitcoin double-SHA256 hashing algorithm.
fn reduce_path<T: AsRef<[u8]>>(coinbase_id: [u8; 32], path: &[T]) -> [u8; 32] {
    let mut root = coinbase_id;
    for node in path {
        let to_hash = [&root[..], node.as_ref()].concat();
        let hash = DHash::hash(&to_hash);
        root = *hash.as_ref();
    }
    root
}
