use std::{str::FromStr, sync::Arc};

use blockbuster::error::BlockbusterError;
use blockbuster::programs::token_account::TokenProgramAccount;
use blockbuster::{
    program_handler::ProgramParser,
    programs::{
        token_account::TokenAccountParser,
        token_metadata::{TokenMetadataAccountData, TokenMetadataParser},
        ProgramParseResult,
    },
};
use chrono::Utc;
use entities::models::BufferedTransaction;
use flatbuffers::FlatBufferBuilder;
use log::{debug, error, warn};
use plerkle_serialization::AccountInfo;
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use utils::flatbuffer::account_data_generated::account_data::root_as_account_data;

use rocks_db::columns::{Mint, TokenAccount};

use crate::buffer::Buffer;
use crate::error::IngesterError;
use crate::error::IngesterError::MissingFlatbuffersFieldError;
use crate::mplx_updates_processor::MetadataInfo;

const BYTE_PREFIX_TX_SIMPLE_FINALIZED: u8 = 22;
const BYTE_PREFIX_TX_FINALIZED: u8 = 12;
const BYTE_PREFIX_TX_PROCESSED: u8 = 2;
const BYTE_PREFIX_ACCOUNT_FINALIZED: u8 = 10;
const BYTE_PREFIX_ACCOUNT_PROCESSED: u8 = 0;

pub struct MessageHandler {
    buffer: Arc<Buffer>,
    token_acc_parser: Arc<TokenAccountParser>,
    mplx_acc_parser: Arc<TokenMetadataParser>,
}

impl MessageHandler {
    pub fn new(buffer: Arc<Buffer>) -> Self {
        let token_acc_parser = Arc::new(TokenAccountParser {});
        let mplx_acc_parser = Arc::new(TokenMetadataParser {});

        Self {
            buffer,
            token_acc_parser,
            mplx_acc_parser,
        }
    }

    pub async fn call(&self, msg: Vec<u8>) {
        let prefix = msg[0];
        let data = msg[1..].to_vec();

        match prefix {
            BYTE_PREFIX_ACCOUNT_FINALIZED | BYTE_PREFIX_ACCOUNT_PROCESSED => {
                if let Err(err) = self.handle_account(data).await {
                    error!("handle_account: {:?}", err)
                }
            }
            BYTE_PREFIX_TX_SIMPLE_FINALIZED
            | BYTE_PREFIX_TX_FINALIZED
            | BYTE_PREFIX_TX_PROCESSED => {
                let mut res = self.buffer.transactions.lock().await;
                res.push_back(BufferedTransaction {
                    transaction: data,
                    map_flatbuffer: true,
                });
            }
            _ => {}
        }
    }

    async fn handle_account(&self, data: Vec<u8>) -> Result<(), IngesterError> {
        let account_update =
            utils::flatbuffer::account_info_generated::account_info::root_as_account_info(&data)?;

        let account_info_bytes = map_account_info_fb_bytes(account_update)?;

        let account_info = plerkle_serialization::root_as_account_info(account_info_bytes.as_ref())
            .map_err(|e| IngesterError::AccountParsingError(e.to_string()))
            .unwrap();

        let account_owner = account_info
            .owner()
            .ok_or(IngesterError::DeserializationError(
                "Missed account owner field for account".to_string(),
            ))?;

        if account_owner.0 == spl_token::id().to_bytes() {
            self.handle_spl_token_account(&account_info).await?;
        } else if account_owner.0
            == blockbuster::programs::token_metadata::token_metadata_id().to_bytes()
        {
            self.handle_token_metadata_account(&account_info).await?;
        }

        Ok(())
    }

    async fn handle_spl_token_account<'a>(
        &self,
        account_info: &'a AccountInfo<'a>,
    ) -> Result<(), IngesterError> {
        // skip processing of empty mints and multi-sig accounts
        if let Some(account_info) = account_info.data() {
            let account_data = account_info.iter().collect::<Vec<_>>();
            if account_data.as_slice() == [0; spl_token::state::Mint::LEN] // empty mint
                || account_data.as_slice() == [0; spl_token::state::Account::LEN] // empty token account
                || account_data.len() == spl_token::state::Multisig::LEN
            {
                return Ok(());
            }
        }
        let acc_parse_result = self.token_acc_parser.handle_account(account_info);

        match acc_parse_result {
            Ok(acc_parsed) => {
                let concrete = acc_parsed.result_type();
                match concrete {
                    ProgramParseResult::TokenProgramAccount(parsing_result) => {
                        self.write_spl_accounts_models_to_buffer(account_info, parsing_result)
                            .await
                    }
                    _ => debug!("\nUnexpected message\n"),
                };
            }
            Err(e) => {
                account_parsing_error(e, account_info);
            }
        }

        Ok(())
    }

    async fn write_spl_accounts_models_to_buffer<'a, 'b>(
        &self,
        account_update: &'a AccountInfo<'a>,
        parsing_result: &'b TokenProgramAccount,
    ) {
        let key = *account_update.pubkey().unwrap();
        let key_bytes = key.0.to_vec();
        match &parsing_result {
            TokenProgramAccount::TokenAccount(ta) => {
                let frozen = matches!(ta.state, spl_token::state::AccountState::Frozen);

                let token_acc_model = TokenAccount {
                    pubkey: Pubkey::try_from(key_bytes.clone()).unwrap(),
                    mint: ta.mint,
                    delegate: ta.delegate.into(),
                    owner: ta.owner,
                    frozen,
                    delegated_amount: ta.delegated_amount as i64,
                    slot_updated: account_update.slot() as i64,
                    amount: ta.amount as i64,
                };

                let mut token_accounts = self.buffer.token_accs.lock().await;

                if let Some(existing_model) = token_accounts.get(key_bytes.as_slice()) {
                    if token_acc_model.slot_updated > existing_model.slot_updated {
                        token_accounts.insert(key_bytes, token_acc_model);
                    }
                } else {
                    token_accounts.insert(key_bytes, token_acc_model);
                }
            }
            TokenProgramAccount::Mint(m) => {
                let mint_acc_model = Mint {
                    pubkey: Pubkey::try_from(key_bytes.clone()).unwrap_or_default(),
                    slot_updated: account_update.slot() as i64,
                    supply: m.supply as i64,
                    decimals: m.decimals as i32,
                    mint_authority: m.mint_authority.into(),
                    freeze_authority: m.freeze_authority.into(),
                };

                let mut mints = self.buffer.mints.lock().await;

                if let Some(existing_model) = mints.get(key_bytes.as_slice()) {
                    if mint_acc_model.slot_updated > existing_model.slot_updated {
                        mints.insert(key_bytes, mint_acc_model);
                    }
                } else {
                    mints.insert(key_bytes, mint_acc_model);
                }
            }
            _ => debug!("Not implemented"),
        };
    }

    async fn handle_token_metadata_account<'a>(
        &self,
        account_info: &'a AccountInfo<'a>,
    ) -> Result<(), IngesterError> {
        let acc_parse_result = self.mplx_acc_parser.handle_account(account_info);

        match acc_parse_result {
            Ok(acc_parsed) => {
                let concrete = acc_parsed.result_type();

                match concrete {
                    ProgramParseResult::TokenMetadata(parsing_result) => {
                        let key = account_info.pubkey().unwrap();

                        match &parsing_result.data {
                            TokenMetadataAccountData::MetadataV1(m) => {
                                let mut metadata = self.buffer.mplx_metadata_info.lock().await;

                                if let Some(existed_models) =
                                    metadata.get(m.mint.to_bytes().as_ref())
                                {
                                    if existed_models.slot < account_info.slot() {
                                        metadata.insert(
                                            key.0.to_vec(),
                                            MetadataInfo {
                                                metadata: m.clone(),
                                                slot: account_info.slot(),
                                            },
                                        );
                                    }
                                } else {
                                    metadata.insert(
                                        key.0.to_vec(),
                                        MetadataInfo {
                                            metadata: m.clone(),
                                            slot: account_info.slot(),
                                        },
                                    );
                                }
                            }
                            _ => debug!("Not implemented"),
                        };
                    }
                    _ => debug!("\nUnexpected message\n"),
                };
            }
            Err(e) => match e {
                BlockbusterError::AccountTypeNotImplemented => {}
                _ => account_parsing_error(e, account_info),
            },
        }

        Ok(())
    }
}

fn map_account_info_fb_bytes(
    account_update: utils::flatbuffer::account_info_generated::account_info::AccountInfo,
) -> Result<Vec<u8>, IngesterError> {
    let account_data = root_as_account_data(
        account_update
            .account_data()
            .ok_or(MissingFlatbuffersFieldError("account data".to_string()))?
            .bytes(),
    )?;

    let pubkey = plerkle_serialization::Pubkey::new(
        &Pubkey::from_str(
            account_update
                .pubkey()
                .ok_or(MissingFlatbuffersFieldError("pubkey".to_string()))?,
        )?
        .to_bytes(),
    );
    let owner = plerkle_serialization::Pubkey::new(
        &Pubkey::from_str(
            account_update
                .owner()
                .ok_or(MissingFlatbuffersFieldError("owner".to_string()))?,
        )?
        .to_bytes(),
    );

    let mut builder = FlatBufferBuilder::new();

    let data = builder.create_vector(
        account_data
            .data()
            .ok_or(MissingFlatbuffersFieldError("data".to_string()))?
            .bytes(),
    );

    let args = plerkle_serialization::AccountInfoArgs {
        pubkey: Some(&pubkey),
        owner: Some(&owner),
        data: Some(data),
        slot: account_update.slot(),
        is_startup: false,
        seen_at: Utc::now().timestamp_millis(),
        lamports: account_data.lamports(),
        executable: account_data.executable(),
        rent_epoch: account_data.rent_epoch(),
        write_version: account_data.version(),
    };

    let account_info_wip = plerkle_serialization::AccountInfo::create(&mut builder, &args);
    builder.finish(account_info_wip, None);

    Ok(builder.finished_data().to_owned())
}

fn account_parsing_error(err: BlockbusterError, account_info: &AccountInfo) {
    warn!(
        "Error while parsing account: {:?} {}",
        err,
        bs58::encode(
            account_info
                .pubkey()
                .unwrap_or(&plerkle_serialization::Pubkey::new(&[0u8; 32]))
                .key()
                .iter()
                .collect::<Vec<_>>()
                .as_slice()
        )
        .into_string()
    );
}

#[test]
fn test_mint_uninitialized() {
    let unpack_res = spl_token::state::Mint::unpack(&[0; 82]);
    assert_eq!(
        Err(solana_program::program_error::ProgramError::UninitializedAccount),
        unpack_res
    )
}

#[test]
fn test_token_account_uninitialized() {
    let unpack_res = spl_token::state::Account::unpack(&[0; 165]);
    assert_eq!(
        Err(solana_program::program_error::ProgramError::UninitializedAccount),
        unpack_res
    )
}
