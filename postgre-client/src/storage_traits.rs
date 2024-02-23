use crate::model::{AssetSortedIndex, AssetSorting, SearchAssetsFilter};
use async_trait::async_trait;
use entities::models::AssetIndex;
use mockall::automock;
use std::collections::HashSet;

#[automock]
#[async_trait]
pub trait AssetIndexStorage {
    async fn fetch_last_synced_id(&self) -> Result<Option<Vec<u8>>, String>;
    async fn update_asset_indexes_batch(
        &self,
        asset_indexes: &[AssetIndex],
        last_key: &[u8],
    ) -> Result<(), String>;
    async fn load_from_dump(
        &self,
        base_path: &std::path::Path,
        last_key: &[u8],
    ) -> Result<(), String>;
    async fn get_existing_metadata_keys(&self) -> Result<HashSet<Vec<u8>>, String>;
}

#[automock]
#[async_trait]
pub trait AssetPubkeyFilteredFetcher {
    async fn get_asset_pubkeys_filtered(
        &self,
        filter: &SearchAssetsFilter,
        order: &AssetSorting,
        limit: u64,
        page: Option<u64>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<Vec<AssetSortedIndex>, String>;
}

#[automock]
#[async_trait]
pub trait IntegrityVerificationKeysFetcher {
    async fn get_verification_required_owners_keys(&self) -> Result<Vec<String>, String>;
    async fn get_verification_required_creators_keys(&self) -> Result<Vec<String>, String>;
    async fn get_verification_required_authorities_keys(&self) -> Result<Vec<String>, String>;
    async fn get_verification_required_groups_keys(&self) -> Result<Vec<String>, String>;
    async fn get_verification_required_assets_keys(&self) -> Result<Vec<String>, String>;
    async fn get_verification_required_assets_proof_keys(&self) -> Result<Vec<String>, String>;
}
