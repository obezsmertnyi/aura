use bincode::serialize;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::Ordering;

use crate::asset::{AssetSelectedMaps, AssetsUpdateIdx, SlotAssetIdx};
use crate::column::Column;
use crate::editions::TokenMetadataEdition;
use crate::errors::StorageError;
use crate::key_encoders::encode_u64x2_pubkey;
use crate::{Result, Storage};
use entities::models::EditionData;
use std::collections::HashMap;

impl Storage {
    fn get_next_asset_update_seq(&self) -> Result<u64> {
        if self.assets_update_last_seq.load(Ordering::SeqCst) == 0 {
            // If assets_update_next_seq is zero, fetch the last key from assets_update_idx
            let mut iter = self.assets_update_idx.iter_end(); // Assuming iter_end method fetches the last item

            if let Some(pair) = iter.next() {
                let (last_key, _) = pair?;
                // Assuming the key is structured as (u64, ...)

                let seq = u64::from_be_bytes(last_key[..std::mem::size_of::<u64>()].try_into()?);
                self.assets_update_last_seq.store(seq, Ordering::SeqCst);
            }
        }
        // Increment and return the sequence number
        let seq = self.assets_update_last_seq.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(seq)
    }

    // TODO: Add a backfiller to fill the slot_asset_idx based on the assets_update_idx

    pub fn asset_updated(&self, slot: u64, pubkey: Pubkey) -> Result<()> {
        let mut batch = rocksdb::WriteBatchWithTransaction::<false>::default();
        self.asset_updated_with_batch(&mut batch, slot, pubkey)?;
        self.db.write(batch)?;
        Ok(())
    }

    pub(crate) fn asset_updated_with_batch(
        &self,
        batch: &mut rocksdb::WriteBatchWithTransaction<false>,
        slot: u64,
        pubkey: Pubkey,
    ) -> Result<()> {
        let seq = self.get_next_asset_update_seq()?;
        let value = encode_u64x2_pubkey(seq, slot, pubkey);
        let serialized_value = serialize(&AssetsUpdateIdx {})?;
        batch.put_cf(
            &self.assets_update_idx.handle(),
            Column::<AssetsUpdateIdx>::encode_key(value),
            serialized_value,
        );
        let serialized_value = serialize(&SlotAssetIdx {})?;
        batch.put_cf(
            &self.slot_asset_idx.handle(),
            Column::<SlotAssetIdx>::encode_key((slot, pubkey)),
            serialized_value,
        );
        Ok(())
    }
}

#[macro_export]
macro_rules! to_map {
    ($res:expr) => {{
        $res.map_err(|e| StorageError::Common(e.to_string()))?
            .into_iter()
            .filter_map(|asset| asset.map(|a| (a.pubkey, a)))
            .collect::<HashMap<_, _>>()
    }};
}

impl Storage {
    pub async fn get_asset_selected_maps_async(
        &self,
        asset_ids: Vec<Pubkey>,
    ) -> Result<AssetSelectedMaps> {
        let assets_dynamic_fut = self.asset_dynamic_data.batch_get(asset_ids.clone());
        let assets_static_fut = self.asset_static_data.batch_get(asset_ids.clone());
        let assets_authority_fut = self.asset_authority_data.batch_get(asset_ids.clone());
        let assets_collection_fut = self.asset_collection_data.batch_get(asset_ids.clone());
        let assets_owner_fut = self.asset_owner_data.batch_get(asset_ids.clone());
        let assets_leaf_fut = self.asset_leaf_data.batch_get(asset_ids.clone());

        let assets_dynamic = to_map!(assets_dynamic_fut.await);
        let urls: HashMap<_, _> = assets_dynamic
            .iter()
            .map(|(key, asset)| (key.to_string(), asset.url.value.clone()))
            .collect();
        let offchain_data_fut = self
            .asset_offchain_data
            .batch_get(urls.clone().into_values().collect::<Vec<_>>());

        let (
            assets_static,
            assets_authority,
            assets_collection,
            assets_owner,
            assets_leaf,
            offchain_data,
        ) = tokio::join!(
            assets_static_fut,
            assets_authority_fut,
            assets_collection_fut,
            assets_owner_fut,
            assets_leaf_fut,
            offchain_data_fut
        );
        let offchain_data = offchain_data
            .map_err(|e| StorageError::Common(e.to_string()))?
            .into_iter()
            .filter_map(|asset| asset.map(|a| (a.url.clone(), a)))
            .collect::<HashMap<_, _>>();
        let assets_static = to_map!(assets_static);
        Ok(AssetSelectedMaps {
            editions: self
                .get_editions(
                    assets_static
                        .values()
                        .filter_map(|s| s.edition_address)
                        .collect::<Vec<_>>(),
                )
                .await?,
            assets_static,
            assets_dynamic,
            assets_authority: to_map!(assets_authority),
            assets_collection: to_map!(assets_collection),
            assets_owner: to_map!(assets_owner),
            assets_leaf: to_map!(assets_leaf),
            offchain_data,
            urls,
        })
    }

    async fn get_editions(
        &self,
        edition_keys: Vec<Pubkey>,
    ) -> Result<HashMap<Pubkey, EditionData>> {
        let first_batch = self
            .token_metadata_edition_cbor
            .batch_get_cbor(edition_keys)
            .await?;
        let mut edition_data_list = Vec::new();
        let mut parent_keys = Vec::new();

        for token_metadata_edition in &first_batch {
            match token_metadata_edition {
                Some(TokenMetadataEdition::EditionV1(edition)) => {
                    parent_keys.push(edition.parent);
                }
                Some(TokenMetadataEdition::MasterEdition(master)) => {
                    edition_data_list.push(EditionData {
                        key: master.key,
                        supply: master.supply,
                        max_supply: master.max_supply,
                        edition_number: None,
                    });
                }
                None => {}
            }
        }

        if !parent_keys.is_empty() {
            let master_edition_map = self
                .token_metadata_edition_cbor
                .batch_get_cbor(parent_keys)
                .await?
                .into_iter()
                .filter_map(|e| {
                    if let Some(TokenMetadataEdition::MasterEdition(master)) = e {
                        Some((master.key, master))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            for token_metadata_edition in first_batch.iter().flatten() {
                if let TokenMetadataEdition::EditionV1(edition) = token_metadata_edition {
                    if let Some(master) = master_edition_map.get(&edition.parent) {
                        edition_data_list.push(EditionData {
                            key: edition.key,
                            supply: master.supply,
                            max_supply: master.max_supply,
                            edition_number: Some(edition.edition),
                        });
                    }
                }
            }
        }

        Ok(edition_data_list
            .into_iter()
            .map(|edition| (edition.key, edition))
            .collect::<HashMap<_, _>>())
    }
}
