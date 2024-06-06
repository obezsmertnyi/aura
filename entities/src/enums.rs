use num_derive::FromPrimitive;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Default)]
pub enum RoyaltyTargetType {
    #[default]
    Unknown,
    Creators,
    Fanout,
    Single,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Default)]
pub enum SpecificationVersions {
    #[default]
    Unknown,
    V0,
    V1,
    V2,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Default)]
pub enum SpecificationAssetClass {
    #[default]
    Unknown,
    FungibleToken,
    FungibleAsset,
    Nft,
    PrintableNft,
    ProgrammableNft,
    Print,
    TransferRestrictedNft,
    NonTransferableNft,
    IdentityNft,
    MplCoreAsset,
    MplCoreCollection,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Default)]
pub enum OwnerType {
    #[default]
    Unknown,
    Token,
    Single,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum TokenStandard {
    NonFungible,                    // This is a master edition
    FungibleAsset,                  // A token with metadata that can also have attributes
    Fungible,                       // A token with simple metadata
    NonFungibleEdition,             // This is a limited edition
    ProgrammableNonFungible,        // NonFungible with programmable configuration
    ProgrammableNonFungibleEdition, // NonFungible with programmable configuration
}

impl From<blockbuster::token_metadata::types::TokenStandard> for TokenStandard {
    fn from(value: blockbuster::token_metadata::types::TokenStandard) -> Self {
        match value {
            blockbuster::token_metadata::types::TokenStandard::NonFungible => {
                TokenStandard::NonFungible
            }
            blockbuster::token_metadata::types::TokenStandard::FungibleAsset => {
                TokenStandard::FungibleAsset
            }
            blockbuster::token_metadata::types::TokenStandard::Fungible => TokenStandard::Fungible,
            blockbuster::token_metadata::types::TokenStandard::NonFungibleEdition => {
                TokenStandard::NonFungibleEdition
            }
            blockbuster::token_metadata::types::TokenStandard::ProgrammableNonFungible => {
                TokenStandard::ProgrammableNonFungible
            }
            blockbuster::token_metadata::types::TokenStandard::ProgrammableNonFungibleEdition => {
                TokenStandard::ProgrammableNonFungibleEdition
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromPrimitive)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

impl From<blockbuster::token_metadata::types::UseMethod> for UseMethod {
    fn from(value: blockbuster::token_metadata::types::UseMethod) -> Self {
        match value {
            blockbuster::token_metadata::types::UseMethod::Burn => UseMethod::Burn,
            blockbuster::token_metadata::types::UseMethod::Multiple => UseMethod::Multiple,
            blockbuster::token_metadata::types::UseMethod::Single => UseMethod::Single,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub enum ChainMutability {
    // Original implementation also contain "Unknown"
    // enum variant, which is default. But we do not saved any
    // previous versions of ChainMutability, so if we will want to
    // use unwrap_or_default() on Option<ChainMutability>, it is
    // convenient to have Immutable variant as default, because
    // previous we marked all ChainData as Immutable
    #[default]
    Immutable,
    Mutable,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum Interface {
    #[serde(rename = "V1_NFT")]
    V1NFT,
    #[serde(rename = "V1_PRINT")]
    V1PRINT,
    #[serde(rename = "LEGACY_NFT")]
    LegacyNft,
    #[serde(rename = "V2_NFT")]
    Nft,
    #[serde(rename = "FungibleAsset")]
    FungibleAsset,
    #[serde(rename = "Custom")]
    Custom,
    #[serde(rename = "Identity")]
    Identity,
    #[serde(rename = "Executable")]
    Executable,
    #[serde(rename = "ProgrammableNFT")]
    ProgrammableNFT,
    #[serde(rename = "MplCoreAsset")]
    MplCoreAsset,
    #[serde(rename = "MplCoreCollection")]
    MplCoreCollection,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum OwnershipModel {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "token")]
    Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum RoyaltyModel {
    #[serde(rename = "creators")]
    Creators,
    #[serde(rename = "fanout")]
    Fanout,
    #[serde(rename = "single")]
    Single,
}

#[derive(
    serde_derive::Deserialize,
    serde_derive::Serialize,
    PartialEq,
    Debug,
    Eq,
    Hash,
    sqlx::Type,
    Copy,
    Clone,
    Default,
)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromPrimitive)]
pub enum RollupState {
    Uploaded,
    ValidationFail,
    ValidationComplete,
    UploadedToArweave,
    FailUploadToArweave,
    FailSendingTransaction,
    Complete,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromPrimitive)]
pub enum PersistingRollupState {
    ReceivedTransaction,
    FailedToPersist,
    SuccessfullyDownload,
    SuccessfullyValidate,
    Complete,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromPrimitive)]
pub enum FailedRollupState {
    DownloadFailed,
    ChecksumVerifyFailed,
    RollupVerifyFailed,
    FileSerialization,
}

impl From<FailedRollupState> for u8 {
    fn from(value: FailedRollupState) -> Self {
        match value {
            FailedRollupState::DownloadFailed => 0,
            FailedRollupState::ChecksumVerifyFailed => 1,
            FailedRollupState::RollupVerifyFailed => 2,
            FailedRollupState::FileSerialization => 3,
        }
    }
}

impl TryFrom<u8> for FailedRollupState {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FailedRollupState::DownloadFailed),
            1 => Ok(FailedRollupState::ChecksumVerifyFailed),
            2 => Ok(FailedRollupState::RollupVerifyFailed),
            3 => Ok(FailedRollupState::FileSerialization),
            _ => Err("Wrong enum value".to_string()),
        }
    }
}
