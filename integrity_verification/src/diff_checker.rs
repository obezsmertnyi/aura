use crate::api::IntegrityVerificationApi;
use crate::error::IntegrityVerificationError;
use crate::params::{
    generate_get_asset_params, generate_get_asset_proof_params,
    generate_get_assets_by_authority_params, generate_get_assets_by_creator_params,
    generate_get_assets_by_group_params, generate_get_assets_by_owner_params,
};
use crate::requests::Body;
use crate::slots_dumper::FileSlotsDumper;
use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use metrics_utils::{BackfillerMetricsConfig, IntegrityVerificationMetricsConfig};
use postgre_client::storage_traits::IntegrityVerificationKeysFetcher;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use usecase::bigtable::BigTableClient;
use usecase::slots_collector::SlotsCollector;

pub const GET_ASSET_METHOD: &str = "getAsset";
pub const GET_ASSET_PROOF_METHOD: &str = "getAssetProof";
pub const GET_ASSET_BY_OWNER_METHOD: &str = "getAssetsByOwner";
pub const GET_ASSET_BY_AUTHORITY_METHOD: &str = "getAssetsByAuthority";
pub const GET_ASSET_BY_GROUP_METHOD: &str = "getAssetsByGroup";
pub const GET_ASSET_BY_CREATOR_METHOD: &str = "getAssetsByCreator";

const REQUESTS_INTERVAL_MILLIS: u64 = 1500;
const BIGTABLE_TIMEOUT: u32 = 1000;
const GET_SLOT_METHOD: &str = "getSlot";
const TEST_RETRIES: usize = 10;

#[derive(Default)]
struct DiffWithReferenceResponse {
    diff: Option<String>,
    reference_response: Value,
}

struct CollectSlotsTools {
    bigtable_client: Arc<BigTableClient>,
    slots_collect_path: String,
    metrics: Arc<BackfillerMetricsConfig>,
}

pub struct DiffChecker<T>
where
    T: IntegrityVerificationKeysFetcher + Send + Sync,
{
    reference_host: String,
    testing_host: String,
    api: IntegrityVerificationApi,
    keys_fetcher: T,
    metrics: Arc<IntegrityVerificationMetricsConfig>,
    collect_slots_tools: Option<CollectSlotsTools>,
    regexes: Vec<Regex>,
}

impl<T> DiffChecker<T>
where
    T: IntegrityVerificationKeysFetcher + Send + Sync,
{
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        reference_host: String,
        testing_host: String,
        keys_fetcher: T,
        test_metrics: Arc<IntegrityVerificationMetricsConfig>,
        slot_collect_metrics: Arc<BackfillerMetricsConfig>,
        bigtable_creds: Option<String>,
        slots_collect_path: Option<String>,
        collect_slots_for_proofs: bool,
    ) -> Self {
        // Regular expressions, that purposed to filter out some difference between
        // testing and reference hosts that we already know about
        // Using unwraps is safe, because we pass correct patterns into Regex::new
        let regexes = vec![
            // token_standard field presented in new DAS-API spec, but we do not updated our implementation for now
            Regex::new(r#"json atom at path \".*?\.token_standard\" is missing from rhs\n*"#)
                .unwrap(),
            // cdn_uri field added by Helius, that do not presented in our impl
            Regex::new(r#"json atom at path \".*?\.cdn_uri\" is missing from rhs\n*"#).unwrap(),

            // Below placed regexes for ignoring errors that we must to fix, but already know about
            // TODO: remove after all fixes
            Regex::new(r#"json atoms at path \"(.*?\.compression\.seq)\" are not equal:\n\s*lhs:\n\s*\d+\n\s*rhs:\n\s*\d+\n*"#).unwrap(),
            Regex::new(r#"json atoms at path \"(.*?\.ownership\.delegate)\" are not equal:\n\s*lhs:\n\s*(null|\".*?\"|\d+)\n\s*rhs:\n\s*(null|\".*?\"|\d+)\n*"#).unwrap(),
            Regex::new(r#"json atoms at path \"(.*?\.ownership\.delegated)\" are not equal:\n\s*lhs:\n\s*(true|false|null|\".*?\"|\d+)\n\s*rhs:\n\s*(true|false|null|\".*?\"|\d+)\n*"#).unwrap(),
        ];

        if collect_slots_for_proofs && (bigtable_creds.is_none() || slots_collect_path.is_none()) {
            panic!("Invalid config: trying collect slots for proofs, but do not pass bigtable creds ({:?}) or slots collect path ({:?})", &bigtable_creds, &slots_collect_path);
        }
        let mut collect_slots_tools = None;
        if collect_slots_for_proofs {
            // Unwraps, is safe, because we check it above
            collect_slots_tools = Some(CollectSlotsTools {
                bigtable_client: Arc::new(
                    BigTableClient::connect_new_with(bigtable_creds.unwrap(), BIGTABLE_TIMEOUT)
                        .await
                        .unwrap(),
                ),
                slots_collect_path: slots_collect_path.unwrap(),
                metrics: slot_collect_metrics,
            })
        }

        Self {
            reference_host,
            testing_host,
            api: IntegrityVerificationApi::new(),
            keys_fetcher,
            metrics: test_metrics,
            collect_slots_tools,
            regexes,
        }
    }
}

impl<T> DiffChecker<T>
where
    T: IntegrityVerificationKeysFetcher + Send + Sync,
{
    pub fn compare_responses(
        &self,
        reference_response: &Value,
        testing_response: &Value,
    ) -> Option<String> {
        if let Err(diff) = assert_json_matches_no_panic(
            &reference_response,
            &testing_response,
            Config::new(CompareMode::Strict),
        ) {
            let diff = self
                .regexes
                .iter()
                .fold(diff, |acc, re| re.replace_all(&acc, "").to_string());
            if diff.is_empty() {
                return None;
            }

            return Some(diff);
        }

        None
    }

    async fn check_request(&self, req: &Body) -> DiffWithReferenceResponse {
        let request = json!(req).to_string();
        let reference_response_fut = self.api.make_request(&self.reference_host, &request);
        let testing_response_fut = self.api.make_request(&self.testing_host, &request);
        let (reference_response, testing_response) =
            tokio::join!(reference_response_fut, testing_response_fut);

        let reference_response = match reference_response {
            Ok(reference_response) => reference_response,
            Err(e) => {
                self.metrics.inc_network_errors_reference_host();
                error!("Reference host network error: {}", e);
                return DiffWithReferenceResponse::default();
            }
        };
        let testing_response = match testing_response {
            Ok(testing_response) => testing_response,
            Err(e) => {
                self.metrics.inc_network_errors_testing_host();
                error!("Testing host network error: {}", e);
                return DiffWithReferenceResponse::default();
            }
        };

        DiffWithReferenceResponse {
            diff: self.compare_responses(&reference_response, &testing_response),
            reference_response,
        }
    }

    async fn check_requests<F, G>(
        &self,
        requests: Vec<Body>,
        metrics_inc_total_fn: F,
        metrics_inc_failed_fn: G,
    ) where
        F: Fn() -> u64,
        G: Fn() -> u64,
    {
        for req in requests.iter() {
            metrics_inc_total_fn();
            let mut diff_with_reference_response = DiffWithReferenceResponse::default();
            for _ in 0..TEST_RETRIES {
                diff_with_reference_response = self.check_request(req).await;
                if diff_with_reference_response.diff.is_none() {
                    break;
                }
                // Prevent rate-limit errors
                tokio::time::sleep(Duration::from_millis(REQUESTS_INTERVAL_MILLIS)).await;
            }

            if let Some(diff) = diff_with_reference_response.diff {
                metrics_inc_failed_fn();
                error!(
                    "{}: mismatch responses: req: {:#?}, diff: {}",
                    req.method, req, diff
                );

                self.try_collect_slots(req, &diff_with_reference_response.reference_response)
                    .await;
            }
            // Prevent rate-limit errors
            tokio::time::sleep(Duration::from_millis(REQUESTS_INTERVAL_MILLIS)).await;
        }
    }

    pub async fn check_get_asset(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_assets_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| Body::new(GET_ASSET_METHOD, json!(generate_get_asset_params(key))))
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_asset_tested(),
            || self.metrics.inc_failed_get_asset_tested(),
        )
        .await;

        Ok(())
    }

    pub async fn check_get_asset_proof(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_assets_proof_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| {
                Body::new(
                    GET_ASSET_PROOF_METHOD,
                    json!(generate_get_asset_proof_params(key)),
                )
            })
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_asset_proof_tested(),
            || self.metrics.inc_failed_get_asset_proof_tested(),
        )
        .await;

        Ok(())
    }

    pub async fn check_get_asset_by_authority(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_authorities_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| {
                Body::new(
                    GET_ASSET_BY_AUTHORITY_METHOD,
                    json!(generate_get_assets_by_authority_params(key, None, None)),
                )
            })
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_assets_by_authority_tested(),
            || self.metrics.inc_failed_get_assets_by_authority_tested(),
        )
        .await;

        Ok(())
    }

    pub async fn check_get_asset_by_owner(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_owners_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| {
                Body::new(
                    GET_ASSET_BY_OWNER_METHOD,
                    json!(generate_get_assets_by_owner_params(key, None, None)),
                )
            })
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_assets_by_owner_tested(),
            || self.metrics.inc_failed_get_assets_by_owner_tested(),
        )
        .await;

        Ok(())
    }

    pub async fn check_get_asset_by_group(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_groups_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| {
                Body::new(
                    GET_ASSET_BY_GROUP_METHOD,
                    json!(generate_get_assets_by_group_params(key, None, None)),
                )
            })
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_assets_by_group_tested(),
            || self.metrics.inc_failed_failed_get_assets_by_group_tested(),
        )
        .await;

        Ok(())
    }

    pub async fn check_get_asset_by_creator(&self) -> Result<(), IntegrityVerificationError> {
        let verification_required_keys = self
            .keys_fetcher
            .get_verification_required_creators_keys()
            .await
            .map_err(IntegrityVerificationError::FetchKeys)?;

        let requests = verification_required_keys
            .into_iter()
            .map(|key| {
                Body::new(
                    GET_ASSET_BY_CREATOR_METHOD,
                    json!(generate_get_assets_by_creator_params(key, None, None)),
                )
            })
            .collect::<Vec<_>>();

        self.check_requests(
            requests,
            || self.metrics.inc_total_get_assets_by_creator_tested(),
            || self.metrics.inc_failed_get_assets_by_creator_tested(),
        )
        .await;

        Ok(())
    }
}

impl<T> DiffChecker<T>
where
    T: IntegrityVerificationKeysFetcher + Send + Sync,
{
    async fn try_collect_slots(&self, req: &Body, reference_response: &Value) {
        let collect_tools = match &self.collect_slots_tools {
            None => return,
            Some(collect_tools) => collect_tools,
        };

        if req.method == GET_ASSET_PROOF_METHOD {
            let asset_id = match req.params["id"].as_str() {
                None => {
                    error!("cannot get asset id: {:?}", &req.params);
                    return;
                }
                Some(asset_id) => asset_id,
            };
            let tree_id = match reference_response["result"]["tree_id"].as_str() {
                None => {
                    error!("cannot get tree id: {:?}", &reference_response);
                    return;
                }
                Some(tree_id) => tree_id,
            };
            let slot = match self.get_slot().await {
                Ok(slot) => slot,
                Err(e) => {
                    error!("get_slot: {}", e);
                    return;
                }
            };
            collect_tools.collect_slots(asset_id, tree_id, slot).await;
        }
    }

    async fn get_slot(&self) -> Result<u64, IntegrityVerificationError> {
        let resp = self
            .api
            .make_request(
                &self.reference_host,
                &json!(Body::new(GET_SLOT_METHOD, json!([]),)).to_string(),
            )
            .await?;
        let slot = match resp["result"].as_u64() {
            None => return Err(IntegrityVerificationError::CannotGetSlot(resp.to_string())),
            Some(slot) => slot,
        };

        Ok(slot)
    }
}

impl CollectSlotsTools {
    async fn collect_slots(&self, asset: &str, tree_key: &str, slot: u64) {
        let slots_collector = SlotsCollector::new(
            Arc::new(FileSlotsDumper::new(self.format_filename(tree_key, asset))),
            self.bigtable_client.big_table_inner_client.clone(),
            slot,
            0,
            self.metrics.clone(),
        );

        info!("Start collecting slots for {}", tree_key);
        slots_collector
            .collect_slots(&format!("{}/", tree_key))
            .await;
        info!("Collected slots for {}", tree_key);
    }

    fn format_filename(&self, tree_key: &str, asset: &str) -> String {
        format!("{}/{}-{}.txt", self.slots_collect_path, tree_key, asset)
    }
}

#[cfg(test)]
mod tests {
    use crate::diff_checker::DiffChecker;
    use crate::file_keys_fetcher::FileKeysFetcher;
    use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
    use metrics_utils::utils::start_metrics;
    use metrics_utils::{
        BackfillerMetricsConfig, IntegrityVerificationMetrics, IntegrityVerificationMetricsConfig,
        MetricsTrait,
    };
    use regex::Regex;
    use serde_json::json;

    // this function used only inside tests under rpc_tests and bigtable_tests features, that do not running in our CI
    #[allow(dead_code)]
    async fn create_test_diff_checker() -> DiffChecker<FileKeysFetcher> {
        let mut metrics = IntegrityVerificationMetrics::new(
            IntegrityVerificationMetricsConfig::new(),
            BackfillerMetricsConfig::new(),
        );
        metrics.register_metrics();
        start_metrics(metrics.registry, Some(6001)).await;

        DiffChecker::new(
            "https://test".to_string(),
            "".to_string(),
            FileKeysFetcher::new("./test_keys.txt").await.unwrap(),
            metrics.integrity_verification_metrics.clone(),
            metrics.slot_collector_metrics.clone(),
            Some(String::from("../../creds.json")),
            Some("./".to_string()),
            true,
        )
        .await
    }

    #[cfg(feature = "rpc_tests")]
    #[tokio::test]
    async fn test_get_slot() {
        let slot = create_test_diff_checker().await.get_slot().await.unwrap();

        assert_ne!(slot, 0)
    }

    #[cfg(feature = "bigtable_tests")]
    #[tokio::test]
    async fn test_save_slots_to_file() {
        create_test_diff_checker()
            .await
            .collect_slots_tools
            .unwrap()
            .collect_slots(
                "BAtEs7TuGm2hP2owc9cTit2TNfVzpPFyQAAvkDWs6tDm",
                "4FZcSBJkhPeNAkXecmKnnqHy93ABWzi3Q5u9eXkUfxVE",
                244259062,
            )
            .await;
    }

    #[tokio::test]
    async fn test_regex() {
        let reference_response = json!({
            "jsonrpc": "2.0",
            "result": {
                    "files": [
                        {
                            "uri": "https://assets.pinit.io/3Qru1Gjz9SFd4nESynRQytL65nXNcQGwc1eVbZz24ijG/ZyFU9Lt94Rb57y2hZpAssPCRQU6qXoWzkPhd6bEHKep/731.jpeg",
                            "cdn_uri": "https://cdn.helius-rpc.com/cdn-cgi/image//https://assets.pinit.io/3Qru1Gjz9SFd4nESynRQytL65nXNcQGwc1eVbZz24ijG/ZyFU9Lt94Rb57y2hZpAssPCRQU6qXoWzkPhd6bEHKep/731.jpeg",
                            "mime": "image/jpeg"
                        }
                    ],
                    "metadata": {
                        "description": "GK #731 - Generated and deployed on LaunchMyNFT.",
                        "name": "NFT #731",
                        "symbol": "SYM",
                        "token_standard": "NonFungible"
                    },
                },
            "id": 0
        });

        let testing_response1 = json!({
        "jsonrpc": "2.0",
        "result": {
                "files": [
                    {
                        "uri": "https://assets.pinit.io/3Qru1Gjz9SFd4nESynRQytL65nXNcQGwc1eVbZz24ijG/ZyFU9Lt94Rb57y2hZpAssPCRQU6qXoWzkPhd6bEHKep/731.jpeg",
                        "mime": "image/jpeg"
                    }
                ],
                "metadata": {
                    "description": "GK #731 - Generated and deployed on LaunchMyNFT.",
                    "name": "NFT #731",
                    "symbol": "SYM",
                },
            },
            "id": 0
        });

        let res = assert_json_matches_no_panic(
            &reference_response,
            &testing_response1,
            Config::new(CompareMode::Strict),
        )
        .err()
        .unwrap();

        let re1 = Regex::new(r#"json atom at path \".*?\.token_standard\" is missing from rhs\n*"#)
            .unwrap();
        let re2 =
            Regex::new(r#"json atom at path \".*?\.cdn_uri\" is missing from rhs\n*"#).unwrap();
        let res = re1.replace_all(&res, "").to_string();
        let res = re2.replace_all(&res, "").to_string();

        assert_eq!(0, res.len());

        let testing_response2 = json!({
        "jsonrpc": "2.0",
        "result": {
                "files": [
                    {
                        "uri": "https://assets.pinit.io/3Qru1Gjz9SFd4nESynRQytL65nXNcQGwc1eVbZz24ijG/ZyFU9Lt94Rb57y2hZpAssPCRQU6qXoWzkPhd6bEHKep/731.jpeg",
                        "mime": "image/jpeg"
                    }
                ],
                "mutable": false,
                "metadata": {
                    "description": "GK #731 - Generated and deployed on LaunchMyNFT.",
                    "name": "NFT #731",
                    "symbol": "SYM",
                },
            },
            "id": 0
        });

        let res = assert_json_matches_no_panic(
            &reference_response,
            &testing_response2,
            Config::new(CompareMode::Strict),
        )
        .err()
        .unwrap();

        let res = re1.replace_all(&res, "").to_string();
        let res = re2.replace_all(&res, "").to_string();

        assert_eq!(
            "json atom at path \".result.mutable\" is missing from lhs",
            res.trim()
        );
    }
}
