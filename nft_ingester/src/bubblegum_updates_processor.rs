use crate::buffer::Buffer;
use crate::error::IngesterError;
use crate::flatbuffer_mapper::FlatbufferMapper;
use blockbuster::programs::bubblegum::{BubblegumInstruction, Payload};
use blockbuster::token_metadata::state::{TokenStandard, UseMethod, Uses};
use blockbuster::{
    instruction::{order_instructions, InstructionBundle, IxPair},
    program_handler::ProgramParser,
    programs::{bubblegum::BubblegumParser, ProgramParseResult},
};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use metrics_utils::IngesterMetricsConfig;
use mpl_bubblegum::state::leaf_schema::LeafSchema;
use mpl_bubblegum::InstructionName;
use num_traits::FromPrimitive;
use plerkle_serialization::{Pubkey as FBPubkey, TransactionInfo};
use rocks_db::asset::{
    AssetAuthority, AssetCollection, AssetDynamicDetails, AssetLeaf, AssetStaticDetails,
    ChainDataV1, Creator, RoyaltyTargetType, SpecificationAssetClass, SpecificationVersions,
};
use rocks_db::asset::{AssetOwner, OwnerType};
use serde_json::json;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::Instant;

pub const BUFFER_PROCESSING_COUNTER: i32 = 10;

#[derive(Clone)]
pub struct BubblegumTxProcessor {
    pub transaction_parser: Arc<FlatbufferMapper>,
    pub instruction_parser: Arc<BubblegumParser>,

    pub rocks_client: Arc<rocks_db::Storage>,

    pub buffer: Arc<Buffer>,
    pub metrics: Arc<IngesterMetricsConfig>,
}

impl BubblegumTxProcessor {
    pub fn new(
        rocks_client: Arc<rocks_db::Storage>,
        buffer: Arc<Buffer>,
        metrics: Arc<IngesterMetricsConfig>,
    ) -> Self {
        let instruction_parser = Arc::new(BubblegumParser {});

        let transaction_parser = Arc::new(FlatbufferMapper {});

        BubblegumTxProcessor {
            transaction_parser,
            instruction_parser,
            rocks_client,
            buffer,
            metrics,
        }
    }

    pub fn break_transaction<'i>(
        &self,
        tx: &'i TransactionInfo<'i>,
    ) -> VecDeque<(IxPair<'i>, Option<Vec<IxPair<'i>>>)> {
        let mut ref_set: HashSet<&[u8]> = HashSet::new();
        let k = self.instruction_parser.key();
        ref_set.insert(k.as_ref());
        order_instructions(ref_set, tx)
    }

    pub async fn run(&self, keep_running: Arc<AtomicBool>) {
        let cloned_buffer = self.buffer.clone();

        while keep_running.load(Ordering::SeqCst) {
            let mut buffer = cloned_buffer.transactions.lock().await;
            if let Some(data) = buffer.pop_front() {
                drop(buffer);

                let seen_at = Utc::now();

                let mut transaction_info_bytes = data.transaction.clone();

                if data.map_flatbuffer {
                    let tx_update =
                        utils::flatbuffer::transaction_info_generated::transaction_info::root_as_transaction_info(
                            &data.transaction,
                        ).unwrap();
                    transaction_info_bytes = self
                        .transaction_parser
                        .map_tx_fb_bytes(tx_update, seen_at)
                        .unwrap();
                }
                let transaction_info = plerkle_serialization::root_as_transaction_info(
                    transaction_info_bytes.as_slice(),
                )
                .unwrap();

                let parsed_data = self.handle_transaction(transaction_info).await;
                if let Err(e) = parsed_data {
                    if e != IngesterError::NotImplemented {
                        error!("Background saver could not process received data: {}", e);
                    }
                }
            } else {
                drop(buffer);
            }
        }
    }

    fn instruction_name_to_string(&self, ix: &InstructionName) -> &'static str {
        match ix {
            InstructionName::Unknown => "Unknown",
            InstructionName::MintV1 => "MintV1",
            InstructionName::MintToCollectionV1 => "MintToCollectionV1",
            InstructionName::Redeem => "Redeem",
            InstructionName::CancelRedeem => "CancelRedeem",
            InstructionName::Transfer => "Transfer",
            InstructionName::Delegate => "Delegate",
            InstructionName::DecompressV1 => "DecompressV1",
            InstructionName::Compress => "Compress",
            InstructionName::Burn => "Burn",
            InstructionName::CreateTree => "CreateTree",
            InstructionName::VerifyCreator => "VerifyCreator",
            InstructionName::UnverifyCreator => "UnverifyCreator",
            InstructionName::VerifyCollection => "VerifyCollection",
            InstructionName::UnverifyCollection => "UnverifyCollection",
            InstructionName::SetAndVerifyCollection => "SetAndVerifyCollection",
        }
    }

    // PDA lookup requires an 8-byte array.
    fn u32_to_u8_array(&self, value: u32) -> [u8; 8] {
        let bytes: [u8; 4] = value.to_le_bytes();
        let mut result: [u8; 8] = [0; 8];
        result[..4].copy_from_slice(&bytes);
        result
    }

    pub async fn handle_transaction<'a>(
        &self,
        tx: TransactionInfo<'a>,
    ) -> Result<(), IngesterError> {
        let sig: Option<&str> = tx.signature();

        let instructions = self.break_transaction(&tx);

        let accounts = tx.account_keys().unwrap_or_default();

        let slot = tx.slot();

        let txn_id = tx.signature().unwrap_or("");

        let mut keys: Vec<FBPubkey> = Vec::with_capacity(accounts.len());
        for k in accounts.into_iter() {
            keys.push(*k);
        }

        let mut not_impl = 0;
        let ixlen = instructions.len();

        let contains = instructions
            .iter()
            .filter(|(ib, _inner)| ib.0 .0.as_ref() == mpl_bubblegum::id().as_ref());

        for (outer_ix, inner_ix) in instructions {
            let (program, instruction) = outer_ix;
            let ix_accounts = instruction.accounts().unwrap().iter().collect::<Vec<_>>();
            let ix_account_len = ix_accounts.len();
            let max = ix_accounts.iter().max().copied().unwrap_or(0) as usize;

            if keys.len() < max {
                return Err(IngesterError::DeserializationError(
                    "Missing Accounts in Serialized Ixn/Txn".to_string(),
                ));
            }

            let ix_accounts =
                ix_accounts
                    .iter()
                    .fold(Vec::with_capacity(ix_account_len), |mut acc, a| {
                        if let Some(key) = keys.get(*a as usize) {
                            acc.push(*key);
                        }
                        acc
                    });

            let ix = InstructionBundle {
                txn_id: txn_id,
                program,
                instruction: Some(instruction),
                inner_ix,
                keys: ix_accounts.as_slice(),
                slot,
            };

            if ix.program.0 == mpl_bubblegum::id().to_bytes() {
                let result = self.instruction_parser.handle_instruction(&ix)?;

                let concrete = result.result_type();
                match concrete {
                    ProgramParseResult::Bubblegum(parsing_result) => {
                        self.metrics.inc_instructions(
                            self.instruction_name_to_string(&parsing_result.instruction),
                        );

                        self.handle_bubblegum_instruction(parsing_result, &ix)
                            .await
                            .map_err(|err| {
                                error!(
                                    "Failed to handle bubblegum instruction for txn {:?}: {:?}",
                                    sig, err
                                );

                                return err;
                            })?;
                    }
                    _ => {
                        not_impl += 1;
                    }
                };
            }
        }

        if not_impl == ixlen {
            return Err(IngesterError::NotImplemented);
        }
        Ok(())
    }

    pub async fn handle_bubblegum_instruction<'c>(
        &self,
        parsing_result: &'c BubblegumInstruction,
        bundle: &'c InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        let ix_type = &parsing_result.instruction;

        let begin_processing = Instant::now();

        let ix_str = self.instruction_name_to_string(ix_type);
        info!("BGUM instruction txn={:?}: {:?}", ix_str, bundle.txn_id);

        let mut processed = true;

        match ix_type {
            InstructionName::Transfer => {
                self.transfer(parsing_result, bundle).await?;
            }
            InstructionName::Burn => {
                self.burn(parsing_result, bundle).await?;
            }
            InstructionName::Delegate => {
                self.delegate(parsing_result, bundle).await?;
            }
            InstructionName::MintV1 | InstructionName::MintToCollectionV1 => {
                self.mint_v1(parsing_result, bundle).await?;
            }
            InstructionName::Redeem => {
                self.redeem(parsing_result, bundle).await?;
            }
            InstructionName::CancelRedeem => {
                self.cancel_redeem(parsing_result, bundle).await?;
            }
            InstructionName::DecompressV1 => {
                self.decompress(parsing_result, bundle).await?;
            }
            InstructionName::VerifyCreator => {
                self.creator_verification(parsing_result, bundle).await?;
            }
            InstructionName::UnverifyCreator => {
                self.creator_verification(parsing_result, bundle).await?;
            }
            InstructionName::VerifyCollection
            | InstructionName::UnverifyCollection
            | InstructionName::SetAndVerifyCollection => {
                self.collection_verification(parsing_result, bundle).await?;
            }
            _ => {
                debug!("Bubblegum: Not Implemented Instruction");
                processed = false;
            }
        }

        if processed {
            self.metrics.set_latency(
                "transactions_parser",
                begin_processing.elapsed().as_secs_f64(),
            );
        }
        Ok(())
    }

    pub async fn transfer<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            match le.schema {
                LeafSchema::V1 {
                    id,
                    owner,
                    delegate,
                    nonce,
                    ..
                } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let asset_owner = AssetOwner {
                        pubkey: id,
                        owner,
                        delegate: Some(delegate),
                        owner_type: OwnerType::Single,
                        owner_delegate_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_owner_data.merge(id, &asset_owner) {
                        error!("Error while saving owner for cNFT: {}", e);
                    };
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn burn<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(_le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            let (asset_id, _) = Pubkey::find_program_address(
                &[
                    "asset".as_bytes(),
                    cl.id.as_ref(),
                    self.u32_to_u8_array(cl.index).as_ref(),
                ],
                &mpl_bubblegum::ID,
            );

            self.rocks_client.asset_updated(bundle.slot, asset_id)?;

            let asset_data = self.rocks_client.asset_dynamic_data.get(asset_id).unwrap();
            if let Some(current_asset_data) = asset_data {
                let mut new_asset_data = current_asset_data.clone();
                new_asset_data.is_burnt = (bundle.slot, true);
                new_asset_data.supply = (bundle.slot, Some(0));
                new_asset_data.seq = (bundle.slot, Some(cl.seq));

                if let Some(current_seq) = current_asset_data.seq.1 {
                    if current_seq < cl.seq {
                        if let Err(e) = self
                            .rocks_client
                            .asset_dynamic_data
                            .put(asset_id, &new_asset_data)
                        {
                            error!("Error while saving asset data for cNFT: {}", e);
                        };
                    }
                } else {
                    if let Err(e) = self
                        .rocks_client
                        .asset_dynamic_data
                        .put(asset_id, &new_asset_data)
                    {
                        error!("Error while saving asset data for cNFT: {}", e);
                    };
                }
            } else {
                let new_asset_data = AssetDynamicDetails {
                    pubkey: asset_id,
                    is_compressible: (bundle.slot, false),
                    is_compressed: (bundle.slot, true),
                    is_frozen: (bundle.slot, false),
                    supply: (bundle.slot, Some(0)),
                    seq: (bundle.slot, Some(cl.seq)),
                    is_burnt: (bundle.slot, true),
                    was_decompressed: (bundle.slot, false),
                    onchain_data: (bundle.slot, None),
                    creators: (bundle.slot, vec![]),
                    royalty_amount: (bundle.slot, 0),
                };
                if let Err(e) = self
                    .rocks_client
                    .asset_dynamic_data
                    .put(asset_id, &new_asset_data)
                {
                    error!("Error while saving asset data for cNFT: {}", e);
                };
            }
        }

        Ok(())
    }

    pub async fn delegate<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            match le.schema {
                LeafSchema::V1 {
                    id,
                    owner,
                    delegate,
                    nonce,
                    ..
                } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let asset_owner = AssetOwner {
                        pubkey: id,
                        owner,
                        delegate: Some(delegate),
                        owner_type: OwnerType::Single,
                        owner_delegate_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_owner_data.merge(id, &asset_owner) {
                        error!("Error while saving owner for cNFT: {}", e);
                    };
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn mint_v1<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl), Some(Payload::MintV1 { args })) = (
            &parsing_result.leaf_update,
            &parsing_result.tree_update,
            &parsing_result.payload,
        ) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            let tree_id =
                Pubkey::new_from_array(bundle.keys.get(3).unwrap().0.to_vec().try_into().unwrap());
            let authority =
                Pubkey::new_from_array(bundle.keys.get(0).unwrap().0.to_vec().try_into().unwrap());

            match le.schema {
                LeafSchema::V1 {
                    id,
                    delegate,
                    owner,
                    nonce,
                    ..
                } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let mut chain_data = ChainDataV1 {
                        name: args.name.clone(),
                        symbol: args.symbol.clone(),
                        edition_nonce: args.edition_nonce,
                        primary_sale_happened: args.primary_sale_happened,
                        token_standard: Some(TokenStandard::NonFungible),
                        uses: args.uses.clone().map(|u| Uses {
                            use_method: UseMethod::from_u8(u.use_method as u8).unwrap(),
                            remaining: u.remaining,
                            total: u.total,
                        }),
                    };
                    chain_data.sanitize();

                    let chain_data = json!(chain_data);
                    let asset_static_details = AssetStaticDetails {
                        pubkey: id,
                        specification_asset_class: SpecificationAssetClass::Nft,
                        royalty_target_type: RoyaltyTargetType::Creators,
                        created_at: bundle.slot as i64,
                    };

                    if let Err(e) = self
                        .rocks_client
                        .asset_static_data
                        .put(id, &asset_static_details)
                    {
                        error!("Error while saving static data for cNFT: {}", e);
                    };

                    let creators = {
                        let mut creators = vec![];
                        for creator in args.creators.iter() {
                            creators.push(Creator {
                                creator: creator.address,
                                creator_verified: creator.verified,
                                creator_share: creator.share,
                            });
                        }
                        creators
                    };

                    let asset_dynamic_details = AssetDynamicDetails {
                        pubkey: id,
                        is_compressible: (bundle.slot, false),
                        is_compressed: (bundle.slot, true),
                        is_frozen: (bundle.slot, false),
                        supply: (bundle.slot, Some(1)),
                        seq: (bundle.slot, Some(cl.seq)),
                        is_burnt: (bundle.slot, false),
                        was_decompressed: (bundle.slot, false),
                        onchain_data: (bundle.slot, Some(chain_data.to_string())),
                        creators: (bundle.slot, creators),
                        royalty_amount: (bundle.slot, args.seller_fee_basis_points),
                    };

                    if let Err(e) = self
                        .rocks_client
                        .asset_dynamic_data
                        .merge(id, &asset_dynamic_details)
                    {
                        error!("Error while saving dynamic data for cNFT: {}", e);
                    };

                    let asset_authority = AssetAuthority {
                        pubkey: id,
                        authority: authority,
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self
                        .rocks_client
                        .asset_authority_data
                        .put(id, &asset_authority)
                    {
                        error!("Error while saving authority for cNFT: {}", e);
                    };

                    let asset_owner = AssetOwner {
                        pubkey: id,
                        owner,
                        delegate: Some(delegate),
                        owner_type: OwnerType::Single,
                        owner_delegate_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_owner_data.put(id, &asset_owner) {
                        error!("Error while saving owner for cNFT: {}", e);
                    };

                    let asset_leaf = AssetLeaf {
                        pubkey: id,
                        tree_id,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.put(id, &asset_leaf) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    if let Some(collection) = &args.collection {
                        let asset_collection = AssetCollection {
                            pubkey: id,
                            collection: collection.key,
                            is_collection_verified: collection.verified,
                            collection_seq: Some(cl.seq),
                            slot_updated: bundle.slot,
                        };

                        if let Err(e) = self
                            .rocks_client
                            .asset_collection_data
                            .merge(id, &asset_collection)
                        {
                            error!("Error while saving collection for cNFT: {}", e);
                        };
                    }
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn redeem<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            match le.schema {
                LeafSchema::V1 { id, nonce, .. } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: None,
                        nonce: Some(nonce),
                        data_hash: None,
                        creator_hash: None,
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn cancel_redeem<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            match le.schema {
                LeafSchema::V1 {
                    id,
                    owner,
                    delegate,
                    nonce,
                    ..
                } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let asset_owner = AssetOwner {
                        pubkey: id,
                        owner,
                        delegate: Some(delegate),
                        owner_type: OwnerType::Single,
                        owner_delegate_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_owner_data.merge(id, &asset_owner) {
                        error!("Error while saving owner for cNFT: {}", e);
                    };
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn decompress<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            match le.schema {
                LeafSchema::V1 { id, .. } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: None,
                        nonce: None,
                        data_hash: None,
                        creator_hash: None,
                        leaf_seq: None,
                        slot_updated: bundle.slot,
                    };

                    // if we got decompress instruction we shouldn't even merge data
                    if let Err(e) = self.rocks_client.asset_leaf_data.put(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let asset_data = self.rocks_client.asset_dynamic_data.get(id).unwrap();
                    if let Some(current_asset_data) = asset_data {
                        let mut new_asset_data = current_asset_data.clone();
                        new_asset_data.seq = (bundle.slot, None);
                        new_asset_data.was_decompressed = (bundle.slot, true);

                        if let Err(e) = self
                            .rocks_client
                            .asset_dynamic_data
                            .put(id, &new_asset_data)
                        {
                            error!("Error while saving asset data for cNFT: {}", e);
                        };
                    } else {
                        let new_asset_data = AssetDynamicDetails {
                            pubkey: id,
                            is_compressible: (bundle.slot, true),
                            is_compressed: (bundle.slot, false),
                            is_frozen: (bundle.slot, false),
                            supply: (bundle.slot, Some(1)),
                            seq: (bundle.slot, None),
                            is_burnt: (bundle.slot, false),
                            was_decompressed: (bundle.slot, true),
                            onchain_data: (bundle.slot, None),
                            creators: (bundle.slot, vec![]),
                            royalty_amount: (bundle.slot, 0),
                        };
                        if let Err(e) = self
                            .rocks_client
                            .asset_dynamic_data
                            .put(id, &new_asset_data)
                        {
                            error!("Error while saving asset data for cNFT: {}", e);
                        };
                    }
                }
            }

            return Ok(());
        }

        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn creator_verification<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl), Some(payload)) = (
            &parsing_result.leaf_update,
            &parsing_result.tree_update,
            &parsing_result.payload,
        ) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            let (creator, verify) = match payload {
                Payload::CreatorVerification {
                    creator, verify, ..
                } => (creator, verify),
                _ => {
                    return Err(IngesterError::DatabaseError(
                        "Ix not parsed correctly".to_string(),
                    ));
                }
            };

            match le.schema {
                LeafSchema::V1 { id, nonce, .. } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let asset_data = self.rocks_client.asset_dynamic_data.get(id).unwrap();
                    if let Some(current_asset_data) = asset_data {
                        let mut new_asset_data = current_asset_data.clone();
                        new_asset_data.seq = (bundle.slot, Some(cl.seq));

                        for crt in new_asset_data.creators.1.iter_mut() {
                            if crt.creator == *creator {
                                crt.creator_verified = *verify;
                            }
                        }

                        if let Err(e) = self
                            .rocks_client
                            .asset_dynamic_data
                            .put(id, &new_asset_data)
                        {
                            error!("Error while saving asset data for cNFT: {}", e);
                        };
                    } else {
                        let creator = Creator {
                            creator: *creator,
                            creator_verified: *verify,
                            creator_share: 0,
                        };

                        let new_asset_data = AssetDynamicDetails {
                            pubkey: id,
                            is_compressible: (bundle.slot, false),
                            is_compressed: (bundle.slot, true),
                            is_frozen: (bundle.slot, false),
                            supply: (bundle.slot, Some(1)),
                            seq: (bundle.slot, Some(cl.seq)),
                            is_burnt: (bundle.slot, false),
                            was_decompressed: (bundle.slot, false),
                            onchain_data: (bundle.slot, None),
                            creators: (bundle.slot, vec![creator]),
                            royalty_amount: (bundle.slot, 0),
                        };
                        if let Err(e) = self
                            .rocks_client
                            .asset_dynamic_data
                            .put(id, &new_asset_data)
                        {
                            error!("Error while saving asset data for cNFT: {}", e);
                        };
                    }
                }
            }

            return Ok(());
        }
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }

    pub async fn collection_verification<'c>(
        &self,
        parsing_result: &BubblegumInstruction,
        bundle: &InstructionBundle<'c>,
    ) -> Result<(), IngesterError> {
        if let (Some(le), Some(cl), Some(payload)) = (
            &parsing_result.leaf_update,
            &parsing_result.tree_update,
            &parsing_result.payload,
        ) {
            self.rocks_client.save_changelog(cl, bundle.slot).await;

            let (collection, verify) = match payload {
                Payload::CollectionVerification {
                    collection, verify, ..
                } => (*collection, *verify),
                _ => {
                    return Err(IngesterError::DatabaseError(
                        "Ix not parsed correctly".to_string(),
                    ));
                }
            };

            match le.schema {
                LeafSchema::V1 { id, nonce, .. } => {
                    self.rocks_client.asset_updated(bundle.slot, id)?;

                    let tree = cl.id;

                    let leaf_info = AssetLeaf {
                        pubkey: id,
                        tree_id: tree,
                        leaf: Some(le.leaf_hash.to_vec()),
                        nonce: Some(nonce),
                        data_hash: Some(Hash::from(le.schema.data_hash())),
                        creator_hash: Some(Hash::from(le.schema.creator_hash())),
                        leaf_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self.rocks_client.asset_leaf_data.merge(id, &leaf_info) {
                        error!("Error while saving leaf for cNFT: {}", e);
                    };

                    let collection = AssetCollection {
                        pubkey: id,
                        collection: collection,
                        is_collection_verified: verify,
                        collection_seq: Some(cl.seq),
                        slot_updated: bundle.slot,
                    };

                    if let Err(e) = self
                        .rocks_client
                        .asset_collection_data
                        .merge(id, &collection)
                    {
                        error!("Error while saving collection for cNFT: {}", e);
                    };
                }
            }

            return Ok(());
        };
        Err(IngesterError::ParsingError(
            "Ix not parsed correctly".to_string(),
        ))
    }
}
