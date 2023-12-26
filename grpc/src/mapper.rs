use crate::gapfiller::{
    AssetCollection, AssetDetails, AssetLeaf, ChainDataV1, Creator, DynamicBoolField,
    DynamicBytesField, DynamicCreatorsField, DynamicEnumField, DynamicUint32Field,
    DynamicUint64Field, OwnerType, RoyaltyTargetType, SpecificationAssetClass,
    SpecificationVersions, TokenStandard, UseMethod, Uses,
};
use entities::models::{CompleteAssetDetails, Updated};
use solana_sdk::pubkey::Pubkey;
impl From<CompleteAssetDetails> for AssetDetails {
    fn from(value: CompleteAssetDetails) -> Self {
        Self {
            pubkey: value.pubkey.to_bytes().to_vec(),
            specification_asset_class: SpecificationAssetClass::from(
                value.specification_asset_class,
            )
            .into(),
            royalty_target_type: RoyaltyTargetType::from(value.royalty_target_type).into(),
            slot_created: value.slot_created,
            is_compressible: Some(value.is_compressible.into()),
            is_compressed: Some(value.is_compressed.into()),
            is_frozen: Some(value.is_frozen.into()),
            supply: value.supply.map(|v| v.into()),
            seq: value.seq.map(|v| v.into()),
            is_burnt: Some(value.is_burnt.into()),
            was_decompressed: Some(value.was_decompressed.into()),
            creators: Some(value.creators.into()),
            royalty_amount: Some(value.royalty_amount.into()),
            authority: Some(value.authority.into()),
            owner: Some(value.owner.into()),
            delegate: value.delegate.map(|v| v.into()),
            owner_type: Some(value.owner_type.into()),
            owner_delegate_seq: value.owner_delegate_seq.map(|v| v.into()),
            leaves: value.leaves.iter().map(AssetLeaf::from).collect(),
            collection: value.collection.map(|v| v.into()),
            chain_data: value.onchain_data.map(|v| v.into()),
        }
    }
}

impl From<Updated<bool>> for DynamicBoolField {
    fn from(value: Updated<bool>) -> Self {
        Self {
            value: value.value,
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<u64>> for DynamicUint64Field {
    fn from(value: Updated<u64>) -> Self {
        Self {
            value: value.value,
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<u16>> for DynamicUint32Field {
    fn from(value: Updated<u16>) -> Self {
        Self {
            value: value.value as u32,
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<Pubkey>> for DynamicBytesField {
    fn from(value: Updated<Pubkey>) -> Self {
        Self {
            value: value.value.to_bytes().to_vec(),
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<entities::enums::OwnerType>> for DynamicEnumField {
    fn from(value: Updated<entities::enums::OwnerType>) -> Self {
        Self {
            value: OwnerType::from(value.value).into(),
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<&entities::models::Creator> for Creator {
    fn from(value: &entities::models::Creator) -> Self {
        Self {
            creator: value.creator.to_bytes().to_vec(),
            creator_verified: value.creator_verified,
            creator_share: value.creator_share as u32,
        }
    }
}
impl From<Updated<Vec<entities::models::Creator>>> for DynamicCreatorsField {
    fn from(value: Updated<Vec<entities::models::Creator>>) -> Self {
        Self {
            creators: value.value.iter().map(|v| v.into()).collect(),
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<&Updated<entities::models::AssetLeaf>> for AssetLeaf {
    fn from(value: &Updated<entities::models::AssetLeaf>) -> Self {
        Self {
            tree_id: value.value.tree_id.to_bytes().to_vec(),
            leaf: value.value.leaf.clone(),
            nonce: value.value.nonce,
            data_hash: value.value.data_hash.map(|h| h.to_bytes().to_vec()),
            creator_hash: value.value.creator_hash.map(|h| h.to_bytes().to_vec()),
            leaf_seq: value.value.leaf_seq,
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<entities::models::AssetCollection>> for AssetCollection {
    fn from(value: Updated<entities::models::AssetCollection>) -> Self {
        Self {
            collection: value.value.collection.to_bytes().to_vec(),
            is_collection_verified: value.value.is_collection_verified,
            collection_seq: value.value.collection_seq,
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<Updated<entities::models::ChainDataV1>> for ChainDataV1 {
    fn from(value: Updated<entities::models::ChainDataV1>) -> Self {
        Self {
            name: value.value.name.clone(),
            symbol: value.value.symbol.clone(),
            edition_nonce: value.value.edition_nonce.map(|v| v as u32),
            primary_sale_happened: value.value.primary_sale_happened,
            token_standard: value
                .value
                .token_standard
                .map(|v| TokenStandard::from(v).into())
                .unwrap_or_default(),
            uses: value.value.uses.map(|v| v.into()),
            slot_updated: value.slot_updated,
            seq_updated: value.seq,
        }
    }
}

impl From<entities::models::Uses> for Uses {
    fn from(value: entities::models::Uses) -> Self {
        Self {
            use_method: UseMethod::from(value.use_method).into(),
            remaining: value.remaining,
            total: value.total,
        }
    }
}
macro_rules! impl_from_enum {
    ($src:ty, $dst:ident, $($variant:ident),*) => {
        impl From<$src> for $dst {
            fn from(value: $src) -> Self {
                match value {
                    $(
                        <$src>::$variant => $dst::$variant,
                    )*
                }
            }
        }
    };
}

impl_from_enum!(
    entities::enums::SpecificationVersions,
    SpecificationVersions,
    Unknown,
    V0,
    V1,
    V2
);
impl_from_enum!(
    entities::enums::SpecificationAssetClass,
    SpecificationAssetClass,
    Unknown,
    FungibleToken,
    FungibleAsset,
    Nft,
    PrintableNft,
    ProgrammableNft,
    Print,
    TransferRestrictedNft,
    NonTransferableNft,
    IdentityNft
);
impl_from_enum!(
    entities::enums::RoyaltyTargetType,
    RoyaltyTargetType,
    Unknown,
    Creators,
    Fanout,
    Single
);
impl_from_enum!(
    entities::enums::OwnerType,
    OwnerType,
    Unknown,
    Token,
    Single
);
impl_from_enum!(
    entities::enums::TokenStandard,
    TokenStandard,
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    ProgrammableNonFungibleEdition
);
impl_from_enum!(
    entities::enums::UseMethod,
    UseMethod,
    Burn,
    Multiple,
    Single
);
