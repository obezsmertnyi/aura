pub mod errors;
pub mod utils;
use chrono::Utc;
use std::fmt;
use std::sync::Arc;

use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;

pub struct IntegrityVerificationMetrics {
    pub integrity_verification_metrics: Arc<IntegrityVerificationMetricsConfig>,
    pub registry: Registry,
}

impl IntegrityVerificationMetrics {
    pub fn new(integrity_verification_metrics: IntegrityVerificationMetricsConfig) -> Self {
        Self {
            integrity_verification_metrics: Arc::new(integrity_verification_metrics),
            registry: Registry::default(),
        }
    }
}

#[derive(Debug)]
pub struct MetricState {
    pub ingester_metrics: Arc<IngesterMetricsConfig>,
    pub api_metrics: Arc<ApiMetricsConfig>,
    pub json_downloader_metrics: Arc<JsonDownloaderMetricsConfig>,
    pub backfiller_metrics: Arc<BackfillerMetricsConfig>,
    pub rpc_backfiller_metrics: Arc<RpcBackfillerMetricsConfig>,
    pub synchronizer_metrics: Arc<SynchronizerMetricsConfig>,
    pub registry: Registry,
}

impl MetricState {
    pub fn new(
        ingester_metrics: IngesterMetricsConfig,
        api_metrics: ApiMetricsConfig,
        json_downloader_metrics: JsonDownloaderMetricsConfig,
        backfiller_metrics: BackfillerMetricsConfig,
        rpc_backfiller_metrics: RpcBackfillerMetricsConfig,
        synchronizer_metrics: SynchronizerMetricsConfig,
    ) -> Self {
        Self {
            ingester_metrics: Arc::new(ingester_metrics),
            api_metrics: Arc::new(api_metrics),
            json_downloader_metrics: Arc::new(json_downloader_metrics),
            registry: Registry::default(),
            backfiller_metrics: Arc::new(backfiller_metrics),
            rpc_backfiller_metrics: Arc::new(rpc_backfiller_metrics),
            synchronizer_metrics: Arc::new(synchronizer_metrics),
        }
    }
}

pub trait MetricsTrait {
    fn register_metrics(&mut self);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MethodLabel {
    pub method_name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MetricLabel {
    pub name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum MetricStatus {
    SUCCESS,
    FAILURE,
    DELETED,
}

impl fmt::Display for MetricStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetricStatus::SUCCESS => write!(f, "success"),
            MetricStatus::FAILURE => write!(f, "failure"),
            MetricStatus::DELETED => write!(f, "deleted"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MetricLabelWithStatus {
    pub name: String,
    pub status: MetricStatus,
}

#[derive(Debug, Clone)]
pub struct BackfillerMetricsConfig {
    slots_collected: Family<MetricLabelWithStatus, Counter>,
    data_processed: Family<MetricLabel, Counter>, // slots & transactions
    last_processed_slot: Family<MetricLabel, Gauge>,
}

impl Default for BackfillerMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BackfillerMetricsConfig {
    pub fn new() -> Self {
        Self {
            slots_collected: Family::<MetricLabelWithStatus, Counter>::default(),
            data_processed: Family::<MetricLabel, Counter>::default(),
            last_processed_slot: Family::<MetricLabel, Gauge>::default(),
        }
    }

    pub fn inc_slots_collected(&self, label: &str, status: MetricStatus) -> u64 {
        self.slots_collected
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_owned(),
                status,
            })
            .inc()
    }

    pub fn inc_data_processed(&self, label: &str) -> u64 {
        self.data_processed
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .inc()
    }

    pub fn set_last_processed_slot(&self, label: &str, slot: i64) -> i64 {
        self.last_processed_slot
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .set(slot)
    }
}

#[derive(Debug, Clone)]
pub struct RpcBackfillerMetricsConfig {
    fetch_signatures: Family<MetricLabelWithStatus, Counter>,
    fetch_transactions: Family<MetricLabelWithStatus, Counter>,
    transactions_processed: Family<MetricLabelWithStatus, Counter>,
    run_fetch_signatures: Family<MetricLabelWithStatus, Counter>,
}

impl Default for RpcBackfillerMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcBackfillerMetricsConfig {
    pub fn new() -> Self {
        Self {
            fetch_signatures: Family::<MetricLabelWithStatus, Counter>::default(),
            fetch_transactions: Family::<MetricLabelWithStatus, Counter>::default(),
            transactions_processed: Family::<MetricLabelWithStatus, Counter>::default(),
            run_fetch_signatures: Family::<MetricLabelWithStatus, Counter>::default(),
        }
    }

    pub fn inc_transactions_processed(&self, label: &str, status: MetricStatus) -> u64 {
        self.transactions_processed
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_string(),
                status,
            })
            .inc()
    }

    pub fn inc_fetch_transactions(&self, label: &str, status: MetricStatus) -> u64 {
        self.fetch_transactions
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_string(),
                status,
            })
            .inc()
    }

    pub fn inc_fetch_signatures(&self, label: &str, status: MetricStatus) -> u64 {
        self.fetch_signatures
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_string(),
                status,
            })
            .inc()
    }

    pub fn inc_run_fetch_signatures(&self, label: &str, status: MetricStatus) -> u64 {
        self.run_fetch_signatures
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_string(),
                status,
            })
            .inc()
    }
}

#[derive(Debug, Clone)]
pub struct SynchronizerMetricsConfig {
    number_of_records_synchronized: Family<MetricLabel, Counter>,
    last_synchronized_slot: Family<MetricLabel, Gauge>,
}

impl Default for SynchronizerMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SynchronizerMetricsConfig {
    pub fn new() -> Self {
        Self {
            number_of_records_synchronized: Family::<MetricLabel, Counter>::default(),
            last_synchronized_slot: Family::<MetricLabel, Gauge>::default(),
        }
    }

    pub fn inc_number_of_records_synchronized(&self, label: &str, num_of_records: u64) -> u64 {
        self.number_of_records_synchronized
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .inc_by(num_of_records)
    }

    pub fn set_last_synchronized_slot(&self, label: &str, slot: i64) -> i64 {
        self.last_synchronized_slot
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .set(slot)
    }
}

#[derive(Debug, Clone)]
pub struct ApiMetricsConfig {
    requests: Family<MethodLabel, Counter>,
    search_asset_requests: Family<MethodLabel, Counter>,
    start_time: Gauge,
    latency: Family<MethodLabel, Histogram>,
}

impl ApiMetricsConfig {
    pub fn new() -> Self {
        Self {
            requests: Family::<MethodLabel, Counter>::default(),
            search_asset_requests: Family::<MethodLabel, Counter>::default(),
            start_time: Default::default(),
            latency: Family::<MethodLabel, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(20.0, 1.8, 10))
            }),
        }
    }

    pub fn inc_requests(&self, label: &str) -> u64 {
        self.requests
            .get_or_create(&MethodLabel {
                method_name: label.to_owned(),
            })
            .inc()
    }

    pub fn inc_search_asset_requests(&self, label: &str) -> u64 {
        self.search_asset_requests
            .get_or_create(&MethodLabel {
                method_name: label.to_owned(),
            })
            .inc()
    }

    pub fn start_time(&self) -> i64 {
        self.start_time.set(Utc::now().timestamp())
    }

    pub fn set_latency(&self, label: &str, duration: f64) {
        self.latency
            .get_or_create(&MethodLabel {
                method_name: label.to_owned(),
            })
            .observe(duration);
    }
}

impl Default for ApiMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsTrait for MetricState {
    fn register_metrics(&mut self) {
        self.api_metrics.start_time();
        self.ingester_metrics.start_time();
        self.json_downloader_metrics.start_time();

        self.registry.register(
            "api_http_requests",
            "The number of HTTP requests made",
            self.api_metrics.requests.clone(),
        );
        self.registry.register(
            "api_http_search_asset_requests",
            "The number of searchAsset requests made",
            self.api_metrics.search_asset_requests.clone(),
        );
        self.registry.register(
            "api_call_latency",
            "A histogram of the request duration",
            self.api_metrics.latency.clone(),
        );
        self.registry.register(
            "api_start_time",
            "Binary start time",
            self.api_metrics.start_time.clone(),
        );

        self.registry.register(
            "ingester_start_time",
            "Binary start time",
            self.ingester_metrics.start_time.clone(),
        );

        self.registry.register(
            "ingester_parsed_data",
            "Total amount of parsed data",
            self.ingester_metrics.parsers.clone(),
        );
        self.registry.register(
            "ingester_processed",
            "Total amount of processed data",
            self.ingester_metrics.process.clone(),
        );
        self.registry.register(
            "ingester_parsed_data_latency",
            "Histogram of data parsing duration",
            self.ingester_metrics.latency.clone(),
        );
        self.registry.register(
            "ingester_buffers",
            "Buffer size",
            self.ingester_metrics.buffers.clone(),
        );
        self.registry.register(
            "ingester_query_retries",
            "Total amount of query retries data",
            self.ingester_metrics.retries.clone(),
        );
        self.registry.register(
            "ingester_bublegum_instructions",
            "Total number of bubblegum instructions processed",
            self.ingester_metrics.instructions.clone(),
        );
        self.registry.register(
            "ingester_rocksdb_backup_latency",
            "Histogram of rocksdb backup duration",
            self.ingester_metrics.rocksdb_backup_latency.clone(),
        );
        self.registry.register(
            "ingester_last_processed_slot",
            "The last processed slot by ingester",
            self.ingester_metrics.last_processed_slot.clone(),
        );

        self.registry.register(
            "json_downloader_latency_task",
            "A histogram of task execution time",
            self.json_downloader_metrics.latency_task_executed.clone(),
        );

        self.registry.register(
            "json_downloader_tasks_count",
            "The total number of tasks made",
            self.json_downloader_metrics.tasks.clone(),
        );

        self.registry.register(
            "json_downloader_tasks_to_execute",
            "The number of tasks that need to be executed",
            self.json_downloader_metrics.tasks_to_execute.clone(),
        );

        self.registry.register(
            "json_downloader_start_time",
            "Binary start time",
            self.json_downloader_metrics.start_time.clone(),
        );

        self.registry.register(
            "backfiller_slots_collected",
            "The number of slots backfiller collected and prepared to parse",
            self.backfiller_metrics.slots_collected.clone(),
        );

        self.registry.register(
            "backfiller_data_processed",
            "The number of data processed by backfiller",
            self.backfiller_metrics.data_processed.clone(),
        );

        self.registry.register(
            "backfiller_last_processed_slot",
            "The last processed slot by backfiller",
            self.backfiller_metrics.last_processed_slot.clone(),
        );

        self.registry.register(
            "rpc_backfiller_transactions_processed",
            "Count of transactions, processed by RPC backfiller",
            self.rpc_backfiller_metrics.transactions_processed.clone(),
        );

        self.registry.register(
            "rpc_backfiller_fetch_signatures",
            "Count of RPC fetch_signatures calls",
            self.rpc_backfiller_metrics.fetch_signatures.clone(),
        );

        self.registry.register(
            "rpc_backfiller_fetch_transactions",
            "Count of RPC fetch_transactions calls",
            self.rpc_backfiller_metrics.fetch_transactions.clone(),
        );

        self.registry.register(
            "rpc_backfiller_run_fetch_signatures",
            "Count of fetch_signatures restarts",
            self.rpc_backfiller_metrics.run_fetch_signatures.clone(),
        );

        self.registry.register(
            "synchronizer_number_of_records_synchronized",
            "Count of records, synchronized by synchronizer",
            self.synchronizer_metrics
                .number_of_records_synchronized
                .clone(),
        );

        self.registry.register(
            "synchronizer_last_synchronized_slot",
            "The last synchronized slot by synchronizer",
            self.synchronizer_metrics.last_synchronized_slot.clone(),
        );
    }
}

impl MetricsTrait for IntegrityVerificationMetrics {
    fn register_metrics(&mut self) {
        self.integrity_verification_metrics.start_time();
    }
}

#[derive(Debug, Clone)]
pub struct IngesterMetricsConfig {
    start_time: Gauge,
    latency: Family<MetricLabel, Histogram>,
    parsers: Family<MetricLabelWithStatus, Counter>,
    process: Family<MetricLabelWithStatus, Counter>,
    buffers: Family<MetricLabel, Gauge>,
    retries: Family<MetricLabel, Counter>,
    rocksdb_backup_latency: Histogram,
    instructions: Family<MetricLabel, Counter>,
    last_processed_slot: Family<MetricLabel, Gauge>,
}

impl IngesterMetricsConfig {
    pub fn new() -> Self {
        Self {
            start_time: Default::default(),
            latency: Family::<MetricLabel, Histogram>::new_with_constructor(|| {
                Histogram::new([1.0, 10.0, 50.0, 100.0].into_iter())
            }),
            parsers: Family::<MetricLabelWithStatus, Counter>::default(),
            process: Family::<MetricLabelWithStatus, Counter>::default(),
            buffers: Family::<MetricLabel, Gauge>::default(),
            retries: Family::<MetricLabel, Counter>::default(),
            rocksdb_backup_latency: Histogram::new(
                [
                    60.0, 300.0, 600.0, 1200.0, 1800.0, 3600.0, 5400.0, 7200.0, 9000.0, 10800.0,
                ]
                .into_iter(),
            ),
            instructions: Family::<MetricLabel, Counter>::default(),
            last_processed_slot: Family::<MetricLabel, Gauge>::default(),
        }
    }

    pub fn start_time(&self) -> i64 {
        self.start_time.set(Utc::now().timestamp())
    }

    pub fn set_latency(&self, label: &str, duration: f64) {
        self.latency
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .observe(duration);
    }

    pub fn set_rocksdb_backup_latency(&self, duration: f64) {
        self.rocksdb_backup_latency.observe(duration);
    }
    pub fn set_buffer(&self, label: &str, buffer_size: i64) {
        self.buffers
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .set(buffer_size);
    }

    pub fn inc_parser(&self, label: &str, status: MetricStatus) -> u64 {
        self.parsers
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_owned(),
                status,
            })
            .inc()
    }

    pub fn inc_process(&self, label: &str, status: MetricStatus) -> u64 {
        self.process
            .get_or_create(&MetricLabelWithStatus {
                name: label.to_owned(),
                status,
            })
            .inc()
    }

    pub fn inc_query_db_retries(&self, label: &str) -> u64 {
        self.retries
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .inc()
    }

    pub fn inc_instructions(&self, label: &str) -> u64 {
        self.instructions
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .inc()
    }

    pub fn set_last_processed_slot(&self, label: &str, slot: i64) -> i64 {
        self.last_processed_slot
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .set(slot)
    }
}

impl Default for IngesterMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsonDownloaderMetricsConfig {
    start_time: Gauge,
    latency_task_executed: Family<MetricLabel, Histogram>,
    tasks: Family<MetricLabel, Counter>,
    tasks_to_execute: Gauge,
}

impl JsonDownloaderMetricsConfig {
    pub fn new() -> Self {
        Self {
            tasks: Family::<MetricLabel, Counter>::default(),
            start_time: Default::default(),
            tasks_to_execute: Default::default(),
            latency_task_executed: Family::<MetricLabel, Histogram>::new_with_constructor(|| {
                Histogram::new([100.0, 500.0, 1000.0, 2000.0].into_iter())
            }),
        }
    }

    pub fn inc_tasks(&self, label: MetricStatus) -> u64 {
        self.tasks
            .get_or_create(&MetricLabel {
                name: label.to_string(),
            })
            .inc()
    }

    pub fn set_tasks_to_execute(&self, count: i64) -> i64 {
        self.tasks_to_execute.set(count)
    }

    pub fn start_time(&self) -> i64 {
        self.start_time.set(Utc::now().timestamp())
    }

    pub fn set_latency_task_executed(&self, label: &str, duration: f64) {
        self.latency_task_executed
            .get_or_create(&MetricLabel {
                name: label.to_owned(),
            })
            .observe(duration);
    }
}

impl Default for JsonDownloaderMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IntegrityVerificationMetricsConfig {
    start_time: Gauge,
}

impl Default for IntegrityVerificationMetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrityVerificationMetricsConfig {
    pub fn new() -> Self {
        Self {
            start_time: Default::default(),
        }
    }

    pub fn start_time(&self) -> i64 {
        self.start_time.set(Utc::now().timestamp())
    }
}
