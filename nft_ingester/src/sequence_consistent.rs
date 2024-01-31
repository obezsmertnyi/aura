use interface::slots_dumper::SlotsDumper;
use metrics_utils::SequenceConsistentGapfillMetricsConfig;
use rocks_db::key_encoders::decode_pubkey_u64;
use rocks_db::tree_seq::{TreeSeqIdx, TreesGaps};
use rocks_db::Storage;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tokio::task::{JoinError, JoinSet};
use tracing::{error, info, warn};
use usecase::slots_collector::{RowKeysGetter, SlotsCollector};

pub struct SequenceConsistentGapfiller<T, R>
where
    T: SlotsDumper + Sync + Send + 'static,
    R: RowKeysGetter + Sync + Send + 'static,
{
    data_layer: Arc<Storage>,
    slots_collector: Arc<SlotsCollector<T, R>>,
    metrics: Arc<SequenceConsistentGapfillMetricsConfig>,
    tasks: Arc<Mutex<JoinSet<Result<(), JoinError>>>>,
}

impl<T, R> SequenceConsistentGapfiller<T, R>
where
    T: SlotsDumper + Sync + Send + 'static,
    R: RowKeysGetter + Sync + Send + 'static,
{
    pub fn new(
        data_layer: Arc<Storage>,
        slots_collector: SlotsCollector<T, R>,
        metrics: Arc<SequenceConsistentGapfillMetricsConfig>,
        tasks: Arc<Mutex<JoinSet<Result<(), JoinError>>>>,
    ) -> Self {
        Self {
            data_layer,
            slots_collector: Arc::new(slots_collector),
            metrics,
            tasks,
        }
    }

    pub async fn collect_sequences_gaps(&self, rx: &mut Receiver<()>) {
        let tree_iterator = self.data_layer.tree_seq_idx.iter_start();
        let mut last_key_before_gap = (solana_program::pubkey::Pubkey::default(), 0);
        let mut tree = solana_program::pubkey::Pubkey::default();
        let mut seq = 0;
        let mut slot = 0;
        let mut find_gap_for_tree = false;
        for pair in tree_iterator {
            if !rx.is_empty() {
                info!("Stop iteration over tree iterator...");
                return;
            }
            let (key, value) = match pair {
                Ok(pair) => pair,
                Err(e) => {
                    error!("Get tree with seq pair: {}", e);
                    continue;
                }
            };
            let (current_tree, current_seq) = match decode_pubkey_u64(key.to_vec()) {
                Ok((current_tree, current_seq)) => (current_tree, current_seq),
                Err(e) => {
                    error!("Decode pubkey u64: {}", e);
                    continue;
                }
            };
            let current_slot = match bincode::deserialize::<TreeSeqIdx>(value.as_ref()) {
                Ok(slot) => slot.slot,
                Err(e) => {
                    error!("Deserialize slot: {}", e);
                    continue;
                }
            };
            if tree == current_tree && current_seq != seq + 1 {
                warn!(
                    "Find GAP for {} tree: sequences: [{}, {}], slots: [{}, {}]",
                    tree, seq, current_seq, slot, current_slot
                );
                if let Err(e) = self
                    .data_layer
                    .trees_gaps
                    .put_async(tree, TreesGaps {})
                    .await
                {
                    error!("Put tree gap: {}", e);
                };
                let slots_collector = self.slots_collector.clone();
                let mut rx_clone = rx.resubscribe();
                self.tasks.lock().await.spawn(tokio::spawn(async move {
                    slots_collector
                        .collect_slots(&format!("{}/", tree), current_slot, slot, &mut rx_clone)
                        .await;
                }));
                self.metrics.set_total_tree_with_gaps(
                    self.data_layer.trees_gaps.iter_start().count() as i64,
                );
                find_gap_for_tree = true;
            };
            if tree != current_tree {
                self.save_tree_gap_analyze(tree, last_key_before_gap, find_gap_for_tree)
                    .await;
                find_gap_for_tree = false
            }
            // If keys already deleted for some tree, we must not to delete other keys in this tree
            // in order to save gap and in future check, if we fix it
            if !find_gap_for_tree {
                last_key_before_gap = (current_tree, current_seq);
            }
            tree = current_tree;
            seq = current_seq;
            slot = current_slot;
        }
        // Handle last tree keys
        self.save_tree_gap_analyze(tree, last_key_before_gap, find_gap_for_tree)
            .await
    }

    async fn save_tree_gap_analyze(
        &self,
        tree: solana_program::pubkey::Pubkey,
        last_key_before_gap: (solana_program::pubkey::Pubkey, u64),
        find_gap_for_tree: bool,
    ) {
        if let Err(e) = self
            .data_layer
            .tree_seq_idx
            .delete_range((tree, 0), last_key_before_gap)
            .await
        {
            error!("Delete range: {}", e)
        }
        // No gap find for tree
        if !find_gap_for_tree {
            if let Err(e) = self.data_layer.trees_gaps.delete(tree) {
                error!("Delete tree gap: {}", e);
            };
        }
        self.metrics
            .set_total_tree_with_gaps(self.data_layer.trees_gaps.iter_start().count() as i64);
    }
}
