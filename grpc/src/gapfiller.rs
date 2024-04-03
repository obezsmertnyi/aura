#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateVersionValue {
    #[prost(enumeration = "UpdateVersion", tag = "1")]
    pub r#type: i32,
    #[prost(uint64, tag = "2")]
    pub value: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uses {
    #[prost(enumeration = "UseMethod", tag = "1")]
    pub use_method: i32,
    #[prost(uint64, tag = "2")]
    pub remaining: u64,
    #[prost(uint64, tag = "3")]
    pub total: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainDataV1 {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub symbol: ::prost::alloc::string::String,
    /// Changed from u8 to uint32 as Protobuf does not have a u8 type
    #[prost(message, optional, tag = "3")]
    pub edition_nonce: ::core::option::Option<u32>,
    #[prost(bool, tag = "4")]
    pub primary_sale_happened: bool,
    #[prost(enumeration = "TokenStandard", tag = "5")]
    pub token_standard: i32,
    #[prost(message, optional, tag = "6")]
    pub uses: ::core::option::Option<Uses>,
    #[prost(message, optional, tag = "7")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "8")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetLeaf {
    #[prost(bytes = "vec", tag = "1")]
    pub tree_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub leaf: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub nonce: ::core::option::Option<u64>,
    #[prost(message, optional, tag = "4")]
    pub data_hash: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "5")]
    pub creator_hash: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "6")]
    pub leaf_seq: ::core::option::Option<u64>,
    #[prost(message, optional, tag = "7")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "8")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetCollection {
    #[prost(bytes = "vec", tag = "1")]
    pub collection: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "2")]
    pub is_collection_verified: bool,
    #[prost(message, optional, tag = "3")]
    pub collection_seq: ::core::option::Option<u64>,
    #[prost(message, optional, tag = "4")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "5")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Creator {
    #[prost(bytes = "vec", tag = "1")]
    pub creator: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "2")]
    pub creator_verified: bool,
    /// Percentage
    #[prost(uint32, tag = "3")]
    pub creator_share: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetDetails {
    /// From AssetStaticDetails
    #[prost(bytes = "vec", tag = "1")]
    pub pubkey: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "SpecificationAssetClass", tag = "2")]
    pub specification_asset_class: i32,
    #[prost(enumeration = "RoyaltyTargetType", tag = "3")]
    pub royalty_target_type: i32,
    #[prost(uint64, tag = "4")]
    pub slot_created: u64,
    #[prost(message, optional, tag = "5")]
    pub edition_address: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    /// From AssetDynamicDetails as Tuples
    #[prost(message, optional, tag = "6")]
    pub is_compressible: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "7")]
    pub is_compressed: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "8")]
    pub is_frozen: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "9")]
    pub supply: ::core::option::Option<DynamicUint64Field>,
    #[prost(message, optional, tag = "10")]
    pub seq: ::core::option::Option<DynamicUint64Field>,
    #[prost(message, optional, tag = "11")]
    pub is_burnt: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "12")]
    pub was_decompressed: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "13")]
    pub creators: ::core::option::Option<DynamicCreatorsField>,
    #[prost(message, optional, tag = "14")]
    pub royalty_amount: ::core::option::Option<DynamicUint32Field>,
    #[prost(message, optional, tag = "15")]
    pub authority: ::core::option::Option<DynamicBytesField>,
    #[prost(message, optional, tag = "16")]
    pub owner: ::core::option::Option<DynamicBytesField>,
    #[prost(message, optional, tag = "17")]
    pub delegate: ::core::option::Option<DynamicBytesField>,
    #[prost(message, optional, tag = "18")]
    pub owner_type: ::core::option::Option<DynamicEnumField>,
    #[prost(message, optional, tag = "19")]
    pub owner_delegate_seq: ::core::option::Option<DynamicUint64Field>,
    #[prost(message, optional, tag = "20")]
    pub chain_mutability: ::core::option::Option<DynamicChainMutability>,
    #[prost(message, optional, tag = "21")]
    pub lamports: ::core::option::Option<DynamicUint64Field>,
    #[prost(message, optional, tag = "22")]
    pub executable: ::core::option::Option<DynamicBoolField>,
    #[prost(message, optional, tag = "23")]
    pub metadata_owner: ::core::option::Option<DynamicStringField>,
    #[prost(message, optional, tag = "24")]
    pub url: ::core::option::Option<DynamicStringField>,
    #[prost(message, optional, tag = "25")]
    pub asset_leaf: ::core::option::Option<AssetLeaf>,
    #[prost(message, optional, tag = "26")]
    pub collection: ::core::option::Option<AssetCollection>,
    #[prost(message, optional, tag = "27")]
    pub chain_data: ::core::option::Option<ChainDataV1>,
    #[prost(message, optional, tag = "28")]
    pub cl_leaf: ::core::option::Option<ClLeaf>,
    #[prost(message, repeated, tag = "29")]
    pub cl_items: ::prost::alloc::vec::Vec<ClItem>,
    /// From TokenMetadataEdition
    #[prost(message, optional, tag = "30")]
    pub edition: ::core::option::Option<EditionV1>,
    #[prost(message, optional, tag = "31")]
    pub master_edition: ::core::option::Option<MasterEdition>,
}
/// Dynamic field messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicBoolField {
    #[prost(bool, tag = "1")]
    pub value: bool,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicUint64Field {
    #[prost(uint64, tag = "1")]
    pub value: u64,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicUint32Field {
    #[prost(uint32, tag = "1")]
    pub value: u32,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicBytesField {
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicStringField {
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicChainMutability {
    #[prost(enumeration = "ChainMutability", tag = "1")]
    pub value: i32,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicEnumField {
    #[prost(enumeration = "OwnerType", tag = "1")]
    pub value: i32,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DynamicCreatorsField {
    #[prost(message, repeated, tag = "1")]
    pub creators: ::prost::alloc::vec::Vec<Creator>,
    #[prost(message, optional, tag = "2")]
    pub update_version: ::core::option::Option<UpdateVersionValue>,
    #[prost(uint64, tag = "3")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClLeaf {
    #[prost(uint64, tag = "1")]
    pub cli_leaf_idx: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub cli_tree_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub cli_node_idx: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClItem {
    #[prost(uint64, tag = "1")]
    pub cli_node_idx: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub cli_tree_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub cli_leaf_idx: ::core::option::Option<u64>,
    #[prost(uint64, tag = "4")]
    pub cli_seq: u64,
    #[prost(uint64, tag = "5")]
    pub cli_level: u64,
    #[prost(bytes = "vec", tag = "6")]
    pub cli_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "7")]
    pub slot_updated: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EditionV1 {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub parent: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub edition: u64,
    #[prost(uint64, tag = "4")]
    pub write_version: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MasterEdition {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub supply: u64,
    #[prost(message, optional, tag = "3")]
    pub max_supply: ::core::option::Option<u64>,
    #[prost(uint64, tag = "4")]
    pub write_version: u64,
}
/// RangeRequest and AssetDetailsResponse for data synchronization
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RangeRequest {
    #[prost(uint64, tag = "1")]
    pub start_slot: u64,
    #[prost(uint64, tag = "2")]
    pub end_slot: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RoyaltyTargetType {
    Unknown = 0,
    Creators = 1,
    Fanout = 2,
    Single = 3,
}
impl RoyaltyTargetType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            RoyaltyTargetType::Unknown => "ROYALTY_TARGET_TYPE_UNKNOWN",
            RoyaltyTargetType::Creators => "ROYALTY_TARGET_TYPE_CREATORS",
            RoyaltyTargetType::Fanout => "ROYALTY_TARGET_TYPE_FANOUT",
            RoyaltyTargetType::Single => "ROYALTY_TARGET_TYPE_SINGLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ROYALTY_TARGET_TYPE_UNKNOWN" => Some(Self::Unknown),
            "ROYALTY_TARGET_TYPE_CREATORS" => Some(Self::Creators),
            "ROYALTY_TARGET_TYPE_FANOUT" => Some(Self::Fanout),
            "ROYALTY_TARGET_TYPE_SINGLE" => Some(Self::Single),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpecificationVersions {
    Unknown = 0,
    V0 = 1,
    V1 = 2,
    V2 = 3,
}
impl SpecificationVersions {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SpecificationVersions::Unknown => "SPECIFICATION_VERSIONS_UNKNOWN",
            SpecificationVersions::V0 => "SPECIFICATION_VERSIONS_V0",
            SpecificationVersions::V1 => "SPECIFICATION_VERSIONS_V1",
            SpecificationVersions::V2 => "SPECIFICATION_VERSIONS_V2",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SPECIFICATION_VERSIONS_UNKNOWN" => Some(Self::Unknown),
            "SPECIFICATION_VERSIONS_V0" => Some(Self::V0),
            "SPECIFICATION_VERSIONS_V1" => Some(Self::V1),
            "SPECIFICATION_VERSIONS_V2" => Some(Self::V2),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpecificationAssetClass {
    Unknown = 0,
    FungibleToken = 1,
    FungibleAsset = 2,
    Nft = 3,
    PrintableNft = 4,
    ProgrammableNft = 5,
    Print = 6,
    TransferRestrictedNft = 7,
    NonTransferableNft = 8,
    IdentityNft = 9,
    MplCoreAsset = 10,
    MplCoreCollection = 11,
}
impl SpecificationAssetClass {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SpecificationAssetClass::Unknown => "SPECIFICATION_ASSET_CLASS_UNKNOWN",
            SpecificationAssetClass::FungibleToken => "SPECIFICATION_ASSET_CLASS_FUNGIBLE_TOKEN",
            SpecificationAssetClass::FungibleAsset => "SPECIFICATION_ASSET_CLASS_FUNGIBLE_ASSET",
            SpecificationAssetClass::Nft => "SPECIFICATION_ASSET_CLASS_NFT",
            SpecificationAssetClass::PrintableNft => "SPECIFICATION_ASSET_CLASS_PRINTABLE_NFT",
            SpecificationAssetClass::ProgrammableNft => {
                "SPECIFICATION_ASSET_CLASS_PROGRAMMABLE_NFT"
            }
            SpecificationAssetClass::Print => "SPECIFICATION_ASSET_CLASS_PRINT",
            SpecificationAssetClass::TransferRestrictedNft => {
                "SPECIFICATION_ASSET_CLASS_TRANSFER_RESTRICTED_NFT"
            }
            SpecificationAssetClass::NonTransferableNft => {
                "SPECIFICATION_ASSET_CLASS_NON_TRANSFERABLE_NFT"
            }
            SpecificationAssetClass::IdentityNft => "SPECIFICATION_ASSET_CLASS_IDENTITY_NFT",
            SpecificationAssetClass::MplCoreAsset => "SPECIFICATION_ASSET_CLASS_MPL_CORE_ASSET",
            SpecificationAssetClass::MplCoreCollection => {
                "SPECIFICATION_ASSET_CLASS_MPL_CORE_COLLECTION"
            }
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SPECIFICATION_ASSET_CLASS_UNKNOWN" => Some(Self::Unknown),
            "SPECIFICATION_ASSET_CLASS_FUNGIBLE_TOKEN" => Some(Self::FungibleToken),
            "SPECIFICATION_ASSET_CLASS_FUNGIBLE_ASSET" => Some(Self::FungibleAsset),
            "SPECIFICATION_ASSET_CLASS_NFT" => Some(Self::Nft),
            "SPECIFICATION_ASSET_CLASS_PRINTABLE_NFT" => Some(Self::PrintableNft),
            "SPECIFICATION_ASSET_CLASS_PROGRAMMABLE_NFT" => Some(Self::ProgrammableNft),
            "SPECIFICATION_ASSET_CLASS_PRINT" => Some(Self::Print),
            "SPECIFICATION_ASSET_CLASS_TRANSFER_RESTRICTED_NFT" => {
                Some(Self::TransferRestrictedNft)
            }
            "SPECIFICATION_ASSET_CLASS_NON_TRANSFERABLE_NFT" => Some(Self::NonTransferableNft),
            "SPECIFICATION_ASSET_CLASS_IDENTITY_NFT" => Some(Self::IdentityNft),
            "SPECIFICATION_ASSET_CLASS_MPL_CORE_ASSET" => Some(Self::MplCoreAsset),
            "SPECIFICATION_ASSET_CLASS_MPL_CORE_COLLECTION" => Some(Self::MplCoreCollection),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OwnerType {
    Unknown = 0,
    Token = 1,
    Single = 2,
}
impl OwnerType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OwnerType::Unknown => "OWNER_TYPE_UNKNOWN",
            OwnerType::Token => "OWNER_TYPE_TOKEN",
            OwnerType::Single => "OWNER_TYPE_SINGLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OWNER_TYPE_UNKNOWN" => Some(Self::Unknown),
            "OWNER_TYPE_TOKEN" => Some(Self::Token),
            "OWNER_TYPE_SINGLE" => Some(Self::Single),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ChainMutability {
    Immutable = 0,
    Mutable = 1,
}
impl ChainMutability {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ChainMutability::Immutable => "CHAIN_MUTABILITY_IMMUTABLE",
            ChainMutability::Mutable => "CHAIN_MUTABILITY_MUTABLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CHAIN_MUTABILITY_IMMUTABLE" => Some(Self::Immutable),
            "CHAIN_MUTABILITY_MUTABLE" => Some(Self::Mutable),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TokenStandard {
    NonFungible = 0,
    FungibleAsset = 1,
    Fungible = 2,
    NonFungibleEdition = 3,
    ProgrammableNonFungible = 4,
    ProgrammableNonFungibleEdition = 5,
}
impl TokenStandard {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TokenStandard::NonFungible => "NON_FUNGIBLE",
            TokenStandard::FungibleAsset => "FUNGIBLE_ASSET",
            TokenStandard::Fungible => "FUNGIBLE",
            TokenStandard::NonFungibleEdition => "NON_FUNGIBLE_EDITION",
            TokenStandard::ProgrammableNonFungible => "PROGRAMMABLE_NON_FUNGIBLE",
            TokenStandard::ProgrammableNonFungibleEdition => "PROGRAMMABLE_NON_FUNGIBLE_EDITION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NON_FUNGIBLE" => Some(Self::NonFungible),
            "FUNGIBLE_ASSET" => Some(Self::FungibleAsset),
            "FUNGIBLE" => Some(Self::Fungible),
            "NON_FUNGIBLE_EDITION" => Some(Self::NonFungibleEdition),
            "PROGRAMMABLE_NON_FUNGIBLE" => Some(Self::ProgrammableNonFungible),
            "PROGRAMMABLE_NON_FUNGIBLE_EDITION" => Some(Self::ProgrammableNonFungibleEdition),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum UseMethod {
    Burn = 0,
    Multiple = 1,
    Single = 2,
}
impl UseMethod {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            UseMethod::Burn => "BURN",
            UseMethod::Multiple => "MULTIPLE",
            UseMethod::Single => "SINGLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BURN" => Some(Self::Burn),
            "MULTIPLE" => Some(Self::Multiple),
            "SINGLE" => Some(Self::Single),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum UpdateVersion {
    Sequence = 0,
    WriteVersion = 1,
}
impl UpdateVersion {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            UpdateVersion::Sequence => "SEQUENCE",
            UpdateVersion::WriteVersion => "WRITE_VERSION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SEQUENCE" => Some(Self::Sequence),
            "WRITE_VERSION" => Some(Self::WriteVersion),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod gap_filler_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// Define the gRPC service
    #[derive(Debug, Clone)]
    pub struct GapFillerServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GapFillerServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GapFillerServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GapFillerServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            GapFillerServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn get_assets_updated_within(
            &mut self,
            request: impl tonic::IntoRequest<super::RangeRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::AssetDetails>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/gapfiller.GapFillerService/GetAssetsUpdatedWithin",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "gapfiller.GapFillerService",
                "GetAssetsUpdatedWithin",
            ));
            self.inner.server_streaming(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod gap_filler_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with GapFillerServiceServer.
    #[async_trait]
    pub trait GapFillerService: Send + Sync + 'static {
        /// Server streaming response type for the GetAssetsUpdatedWithin method.
        type GetAssetsUpdatedWithinStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::AssetDetails, tonic::Status>,
            > + Send
            + 'static;
        async fn get_assets_updated_within(
            &self,
            request: tonic::Request<super::RangeRequest>,
        ) -> std::result::Result<tonic::Response<Self::GetAssetsUpdatedWithinStream>, tonic::Status>;
    }
    /// Define the gRPC service
    #[derive(Debug)]
    pub struct GapFillerServiceServer<T: GapFillerService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: GapFillerService> GapFillerServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GapFillerServiceServer<T>
    where
        T: GapFillerService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/gapfiller.GapFillerService/GetAssetsUpdatedWithin" => {
                    #[allow(non_camel_case_types)]
                    struct GetAssetsUpdatedWithinSvc<T: GapFillerService>(pub Arc<T>);
                    impl<T: GapFillerService>
                        tonic::server::ServerStreamingService<super::RangeRequest>
                        for GetAssetsUpdatedWithinSvc<T>
                    {
                        type Response = super::AssetDetails;
                        type ResponseStream = T::GetAssetsUpdatedWithinStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RangeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GapFillerService>::get_assets_updated_within(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAssetsUpdatedWithinSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: GapFillerService> Clone for GapFillerServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: GapFillerService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: GapFillerService> tonic::server::NamedService for GapFillerServiceServer<T> {
        const NAME: &'static str = "gapfiller.GapFillerService";
    }
}
