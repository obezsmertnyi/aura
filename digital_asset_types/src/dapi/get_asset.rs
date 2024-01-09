use std::sync::Arc;

use sea_orm::DbErr;
use solana_sdk::pubkey::Pubkey;

use rocks_db::Storage;

use crate::{dao::scopes, rpc::Asset};

use super::common::asset_to_rpc;

pub async fn get_asset(rocks_db: Arc<Storage>, id: Pubkey) -> Result<Option<Asset>, DbErr> {
    let assets = scopes::asset::get_by_ids(rocks_db, vec![id]).await?;

    match &assets[0] {
        Some(asset) => asset_to_rpc(asset.clone()),
        None => Ok(None),
    }
}
