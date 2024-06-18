use entities::{
    enums::FailedRollupState,
    models::{AssetSignatureKey, FailedRollupKey},
};
use solana_sdk::pubkey::Pubkey;

use crate::{storage_traits::AssetUpdatedKey, Result};

pub fn encode_u64x2_pubkey(seq: u64, slot: u64, pubkey: Pubkey) -> Vec<u8> {
    // create a key that is a concatenation of the seq, slot and the pubkey allocating memory immediately
    let slot_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let mut key = Vec::with_capacity(slot_size * 2 + pubkey_size);
    key.extend_from_slice(&seq.to_be_bytes());
    key.extend_from_slice(&slot.to_be_bytes());
    key.extend_from_slice(&pubkey.to_bytes());
    key
}

pub fn decode_u64x2_pubkey(bytes: Vec<u8>) -> Result<AssetUpdatedKey> {
    let slot_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    if bytes.len() != slot_size * 2 + pubkey_size {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let seq = u64::from_be_bytes(bytes[..slot_size].try_into()?);
    let slot = u64::from_be_bytes(bytes[slot_size..slot_size * 2].try_into()?);
    let pubkey = Pubkey::try_from(&bytes[slot_size * 2..])?;
    Ok(AssetUpdatedKey::new(seq, slot, pubkey))
}

pub fn encode_u64_pubkey(slot: u64, pubkey: Pubkey) -> Vec<u8> {
    // create a key that is a concatenation of the slot and the pubkey allocating memory immediately
    let slot_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let mut key = Vec::with_capacity(slot_size + pubkey_size);
    key.extend_from_slice(&slot.to_be_bytes());
    key.extend_from_slice(&pubkey.to_bytes());
    key
}

pub fn decode_u64_pubkey(bytes: Vec<u8>) -> Result<(u64, Pubkey)> {
    let slot_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    if bytes.len() != slot_size + pubkey_size {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let slot = u64::from_be_bytes(bytes[..slot_size].try_into()?);
    let pubkey = Pubkey::try_from(&bytes[slot_size..])?;
    Ok((slot, pubkey))
}

pub fn encode_pubkey_u64(pubkey: Pubkey, slot: u64) -> Vec<u8> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let slot_size = std::mem::size_of::<u64>();
    let mut key = Vec::with_capacity(pubkey_size + slot_size);
    key.extend_from_slice(&pubkey.to_bytes());
    key.extend_from_slice(&slot.to_be_bytes());
    key
}

pub fn decode_pubkey_u64(bytes: Vec<u8>) -> Result<(Pubkey, u64)> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let slot_size = std::mem::size_of::<u64>();
    if bytes.len() != slot_size + pubkey_size {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let pubkey = Pubkey::try_from(&bytes[..pubkey_size])?;
    let slot = u64::from_be_bytes(bytes[pubkey_size..].try_into()?);
    Ok((pubkey, slot))
}

pub fn encode_string(key: String) -> Vec<u8> {
    key.into_bytes()
}

pub fn decode_string(bytes: Vec<u8>) -> Result<String> {
    let key = String::from_utf8(bytes).unwrap_or_default();
    Ok(key)
}

pub fn encode_pubkey(pubkey: Pubkey) -> Vec<u8> {
    pubkey.to_bytes().to_vec()
}

pub fn decode_pubkey(bytes: Vec<u8>) -> Result<Pubkey> {
    let key = Pubkey::try_from(&bytes[..])?;
    Ok(key)
}

pub fn encode_u64(slot: u64) -> Vec<u8> {
    slot.to_be_bytes().to_vec()
}

pub fn decode_u64(bytes: Vec<u8>) -> Result<u64> {
    let slot = u64::from_be_bytes(bytes[..].try_into()?);
    Ok(slot)
}

pub fn decode_asset_signature_key(bytes: Vec<u8>) -> Result<AssetSignatureKey> {
    let u64_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    if bytes.len() != u64_size * 2 + pubkey_size {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let tree = Pubkey::try_from(&bytes[..pubkey_size])?;
    let leaf = u64::from_be_bytes(bytes[pubkey_size..pubkey_size + u64_size].try_into()?);
    let seq = u64::from_be_bytes(bytes[pubkey_size + u64_size..].try_into()?);
    Ok(AssetSignatureKey {
        tree,
        leaf_idx: leaf,
        seq,
    })
}

pub fn encode_asset_signature_key(ask: AssetSignatureKey) -> Vec<u8> {
    let u64_size = std::mem::size_of::<u64>();
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let mut key = Vec::with_capacity(u64_size * 2 + pubkey_size);
    key.extend_from_slice(&ask.tree.to_bytes());
    key.extend_from_slice(&ask.leaf_idx.to_be_bytes());
    key.extend_from_slice(&ask.seq.to_be_bytes());
    key
}

pub fn decode_pubkeyx2(bytes: Vec<u8>) -> Result<(Pubkey, Pubkey)> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    if bytes.len() != pubkey_size * 2 {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let pk1 = Pubkey::try_from(&bytes[..pubkey_size])?;
    let pk2 = Pubkey::try_from(&bytes[pubkey_size..])?;
    Ok((pk1, pk2))
}

pub fn encode_pubkeyx2(ask: (Pubkey, Pubkey)) -> Vec<u8> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let mut key = Vec::with_capacity(pubkey_size * 2);
    key.extend_from_slice(&ask.0.to_bytes());
    key.extend_from_slice(&ask.1.to_bytes());
    key
}

pub fn decode_pubkeyx3(bytes: Vec<u8>) -> Result<(Pubkey, Pubkey, Pubkey)> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    if bytes.len() != pubkey_size * 3 {
        return Err(crate::StorageError::InvalidKeyLength);
    }
    let pk1 = Pubkey::try_from(&bytes[..pubkey_size])?;
    let pk2 = Pubkey::try_from(&bytes[pubkey_size..pubkey_size * 2])?;
    let pk3 = Pubkey::try_from(&bytes[pubkey_size * 2..])?;
    Ok((pk1, pk2, pk3))
}

pub fn encode_pubkeyx3(ask: (Pubkey, Pubkey, Pubkey)) -> Vec<u8> {
    let pubkey_size = std::mem::size_of::<Pubkey>();
    let mut key = Vec::with_capacity(pubkey_size * 3);
    key.extend_from_slice(&ask.0.to_bytes());
    key.extend_from_slice(&ask.1.to_bytes());
    key.extend_from_slice(&ask.2.to_bytes());
    key
}

pub fn encode_failed_rollup_key(key: FailedRollupKey) -> Vec<u8> {
    let state = key.status as u8;
    let hash = key.hash.into_bytes();
    [vec![state], hash].concat()
}

pub fn decode_failed_rollup_key(key: Vec<u8>) -> Result<FailedRollupKey> {
    Ok(FailedRollupKey {
        status: FailedRollupState::try_from(
            *key.first().ok_or(crate::StorageError::InvalidKeyLength)?,
        )
        .map_err(crate::StorageError::Common)?,
        hash: String::from_utf8(key[1..].to_vec()).unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    // Import functions from the parent module
    use super::*;

    #[test]
    fn test_encode_decode_u64_pubkey() {
        let seq = 4321u64;
        let slot = 12345u64;
        let pubkey = Pubkey::new_unique(); // or some other way to create a Pubkey

        let encoded = encode_u64x2_pubkey(seq, slot, pubkey);
        let decoded = decode_u64x2_pubkey(encoded).unwrap();

        assert_eq!(decoded.seq, seq);
        assert_eq!(decoded.slot, slot);
        assert_eq!(decoded.pubkey, pubkey);
    }

    #[test]
    fn test_encode_decode_pubkey() {
        let pubkey = Pubkey::new_unique(); // Create a test Pubkey

        let encoded = encode_pubkey(pubkey);
        let decoded = decode_pubkey(encoded).unwrap();

        assert_eq!(decoded, pubkey);
    }

    #[test]
    fn test_decode_u64_pubkey_invalid_data() {
        let invalid_data = vec![1, 2, 3]; // An intentionally invalid byte sequence
        assert!(decode_u64x2_pubkey(invalid_data).is_err());
    }

    #[test]
    fn test_decode_pubkey_invalid_data() {
        let invalid_data = vec![]; // An intentionally invalid byte sequence
        assert!(decode_pubkey(invalid_data).is_err());
    }

    #[test]
    fn test_encode_decode_pubkey_u64() {
        let slot = 12345u64;
        let pubkey = Pubkey::new_unique(); // or some other way to create a Pubkey

        let encoded = encode_pubkey_u64(pubkey, slot);
        let decoded = decode_pubkey_u64(encoded).unwrap();

        assert_eq!(decoded.0, pubkey);
        assert_eq!(decoded.1, slot);
    }

    #[test]
    fn test_pubkey2() {
        let pk1 = Pubkey::new_unique(); // or some other way to create a Pubkey
        let pk2 = Pubkey::new_unique(); // or some other way to create a Pubkey

        let encoded = encode_pubkeyx2((pk1, pk2));
        let decoded = decode_pubkeyx2(encoded).unwrap();

        assert_eq!(decoded.0, pk1);
        assert_eq!(decoded.1, pk2);
    }

    #[test]
    fn test_pubkeyx3() {
        let pk1 = Pubkey::new_unique(); // or some other way to create a Pubkey
        let pk2 = Pubkey::new_unique(); // or some other way to create a Pubkey
        let pk3 = Pubkey::new_unique(); // or some other way to create a Pubkey

        let encoded = encode_pubkeyx3((pk1, pk2, pk3));
        let decoded = decode_pubkeyx3(encoded).unwrap();

        assert_eq!(decoded.0, pk1);
        assert_eq!(decoded.1, pk2);
        assert_eq!(decoded.2, pk3);
    }

    #[test]
    fn test_encode_decode_asset_signature_key() {
        let seq = 4321u64;
        let leaf = 12345u64;
        let tree = Pubkey::new_unique(); // or some other way to create a Pubkey

        let encoded = encode_asset_signature_key(AssetSignatureKey {
            tree,
            leaf_idx: leaf,
            seq,
        });
        let decoded = decode_asset_signature_key(encoded).unwrap();

        assert_eq!(decoded.tree, tree);
        assert_eq!(decoded.leaf_idx, leaf);
        assert_eq!(decoded.seq, seq);
    }

    #[test]
    fn test_encode_decode_failed_rollup_key() {
        let key = FailedRollupKey {
            status: FailedRollupState::DownloadFailed,
            hash: "".to_string(),
        };

        let encoded_key = encode_failed_rollup_key(key.clone());
        let decoded_key = decode_failed_rollup_key(encoded_key).unwrap();

        assert_eq!(decoded_key.status, key.status);
        assert_eq!(decoded_key.hash, key.hash);

        let key2 = FailedRollupKey {
            status: FailedRollupState::RollupVerifyFailed,
            hash: "asdfasdf".to_string(),
        };

        let encoded_key = encode_failed_rollup_key(key2.clone());
        let decoded_key = decode_failed_rollup_key(encoded_key).unwrap();

        assert_eq!(decoded_key.status, key2.status);
        assert_eq!(decoded_key.hash, key2.hash);
    }
    // Add more tests as needed...
}
