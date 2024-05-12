use entities::models::{CompleteAssetDetails, Updated};
use futures::stream;
use interface::asset_streaming_and_discovery::{AsyncError, MockAssetDetailsConsumer};
use metrics_utils::red::RequestErrorDurationMetrics;
use nft_ingester::gapfiller::process_asset_details_stream;
use rocks_db::column_migrator::MigrationState;
use solana_sdk::pubkey::Pubkey;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::{sync::Mutex, task::JoinSet};

use rocks_db::Storage;

fn create_test_complete_asset_details(pubkey: Pubkey) -> CompleteAssetDetails {
    CompleteAssetDetails {
        pubkey,
        supply: Some(Updated::new(1, None, 10)),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_process_asset_details_stream() {
    let temp_dir = TempDir::new().expect("Failed to create a temporary directory");
    let red_metrics = Arc::new(RequestErrorDurationMetrics::new());
    let storage = Arc::new(
        Storage::open(
            temp_dir.path().to_str().unwrap(),
            Arc::new(Mutex::new(JoinSet::new())),
            red_metrics.clone(),
            MigrationState::Last,
        )
        .expect("Failed to create a database"),
    );

    let first_key = Pubkey::new_unique();
    let second_key = Pubkey::new_unique();

    let details1 = create_test_complete_asset_details(first_key.clone());
    let details2 = create_test_complete_asset_details(second_key.clone());

    let mut mock = MockAssetDetailsConsumer::new();
    mock.expect_get_consumable_stream_in_range()
        .returning(move |_, _| {
            Ok(Box::pin(stream::iter(vec![
                Ok(details1.clone()),
                Ok(details2.clone()),
                Err(AsyncError::from("test error")),
            ])))
        });

    process_asset_details_stream(
        Arc::new(AtomicBool::new(true)),
        storage.clone(),
        100,
        200,
        mock,
    )
    .await;

    let selected_data = storage
        .asset_dynamic_data
        .get(first_key.clone())
        .unwrap()
        .unwrap();
    assert_eq!(selected_data.supply, Some(Updated::new(1, None, 10)));

    let selected_data = storage
        .asset_dynamic_data
        .get(second_key.clone())
        .unwrap()
        .unwrap();
    assert_eq!(selected_data.supply, Some(Updated::new(1, None, 10)));
}
