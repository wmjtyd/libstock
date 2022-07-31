//! The configuration file monitor.
//! 
//! WIP: It is stable for application use,
//! but has not abstracted and organized.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

// FIXME: remove anyhow
use anyhow::Context;
use notify::{EventHandler, EventKind};
use serde::Deserialize;
use tokio::{runtime::Handle, task::JoinHandle};
use tracing::instrument;

pub use arcstr::ArcStr;

pub type IpcType = usize;
pub type IpcUrl = ArcStr;
pub type IpcTaskCreator = dyn Fn(Handle, IpcType, IpcUrl) -> JoinHandle<()> + Send + Sync;
pub type IpcBasicRepr = (IpcType, IpcUrl);

#[derive(Deserialize)]
struct IpcConfigStructure {
    pub ipcs: Vec<IpcBasicRepr>,
}

#[derive(Default)]
struct IpcDiff {
    pub added: HashSet<IpcBasicRepr>,
    pub removed: HashSet<IpcBasicRepr>,
}

#[derive(Debug, Clone)]
struct IpcTask {
    pub ipc_type: IpcType,
    pub task_handle: Arc<JoinHandle<()>>,
}

pub struct DynamicConfigHandler {
    /// To allow spawning green thread in handler.
    handle: Handle,

    on_create: Box<IpcTaskCreator>,
    ipcs_map: HashMap<IpcUrl, IpcTask>,
}

impl DynamicConfigHandler {
    pub fn new(handle: Handle, on_create: Box<IpcTaskCreator>) -> Self {
        Self {
            handle,
            on_create,
            ipcs_map: HashMap::with_capacity(5),
        }
    }

    pub fn initiate(&mut self, config_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.register(deserialize_data(config_path)?.ipcs.into_iter());

        Ok(())
    }

    fn diff(&self, new_config: &IpcConfigStructure) -> IpcDiff {
        tracing::trace!("ipcs_map = {:?}", self.ipcs_map);

        // We convert all the IPC structure to IpcBasicRepr.
        let current_ipcs = self
            .ipcs_map
            .iter()
            .map(|(url, IpcTask { ipc_type, .. })| (*ipc_type, url.clone()))
            .collect::<HashSet<IpcBasicRepr>>();

        let new_ipcs = HashSet::<IpcBasicRepr>::from_iter(new_config.ipcs.iter().cloned());

        diff_ipc(current_ipcs, new_ipcs)
    }

    fn cleanup(&mut self, tasks_to_clean_up: impl Iterator<Item = IpcBasicRepr>) {
        for (_, ipc_url) in tasks_to_clean_up {
            let task = self.ipcs_map.remove(&ipc_url);
            if let Some(IpcTask { task_handle, .. }) = task {
                task_handle.abort();
            }
        }
    }

    fn register(&mut self, tasks_to_register: impl Iterator<Item = IpcBasicRepr>) {
        for (ipc_type, ipc_url) in tasks_to_register {
            self.ipcs_map.insert(
                ipc_url.clone(),
                IpcTask {
                    ipc_type,
                    task_handle: (self.on_create)(self.handle.clone(), ipc_type, ipc_url).into(),
                },
            );
        }
    }

    fn _handle_event(&mut self, event: notify::Result<notify::Event>) -> anyhow::Result<()> {
        let event = event.context("failed to handle data change")?;

        if let EventKind::Modify(_) = event.kind {
            let dir = event.paths.get(0).expect("should have at least a path");
            let new_config =
                deserialize_data(dir).context("failed to deserialize the configuration file")?;

            let IpcDiff { removed, added } = self.diff(&new_config);

            tracing::debug!("Removed IPCs: {:?}", removed);
            tracing::debug!("Added IPCs: {:?}", added);

            self.cleanup(removed.into_iter());
            self.register(added.into_iter());
        }

        Ok(())
    }
}

impl EventHandler for DynamicConfigHandler {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let span = tracing::info_span!("handle_event");
        let _span = span.enter();

        match self._handle_event(event) {
            Ok(()) => tracing::debug!("Successfully handled event"),
            Err(err) => tracing::error!("Failed to handle event: {}", err),
        };
    }
}

fn deserialize_data(path: impl AsRef<Path>) -> anyhow::Result<IpcConfigStructure> {
    let content = std::fs::read_to_string(path)?;

    Ok(serde_json::from_str(&content)?)
}

#[instrument]
fn diff_ipc(current: HashSet<IpcBasicRepr>, mut new: HashSet<IpcBasicRepr>) -> IpcDiff {
    let mut diff = IpcDiff::default();

    for ipc in current {
        let contained_in_new_ipcs = new.contains(&ipc);

        tracing::trace!("ipc = {ipc:?}, contains? = {contained_in_new_ipcs}");
        if contained_in_new_ipcs {
            // If `new_ipcs` has the IPC in current_ipcs,
            // we assume the new ipc.json still has this entry.
            // In other words, it is not changed.
            let result = new.remove(&ipc);
            debug_assert!(result);
        } else {
            // If `new_ipcs` do not have the IPC in current_ipcs,
            // we assume the new ipc.json has removed it.
            let result = diff.removed.insert(ipc.clone());
            debug_assert!(result);
        }
    }

    // The remaining `new_ipcs` entries are all the new entries.
    diff.added = new;

    diff
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_diff_ipc_rm_only() {
        let current = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
            (
                7,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc"),
            ),
        ]);

        let new = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
        ]);

        let diff = super::diff_ipc(current, new);
        assert_eq!(diff.added.len(), 0, "should has nothing added");
        assert_eq!(diff.removed.len(), 1, "should has 1 removed");
        assert_eq!(
            diff.removed.iter().next(),
            Some(&(
                7usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc")
            ))
        );
    }

    #[test]
    fn test_diff_ipc_add_only() {
        let current = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
        ]);

        let new = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
            (
                7,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc"),
            ),
        ]);

        let diff = super::diff_ipc(current, new);
        assert_eq!(diff.added.len(), 1, "should has 1 added");
        assert_eq!(diff.removed.len(), 0, "should has nothing removed");
        assert_eq!(
            diff.added.iter().next(),
            Some(&(
                7usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc")
            ))
        );
    }

    #[test]
    fn test_diff_ipc_add_and_rm() {
        let current = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
        ]);

        let new = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                6,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
            (
                7,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc"),
            ),
        ]);

        let diff = super::diff_ipc(current, new);
        assert_eq!(diff.added.len(), 1, "should has 1 added");
        assert_eq!(diff.removed.len(), 1, "should has 1 removed");
        assert_eq!(
            diff.added.iter().next(),
            Some(&(
                7usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc")
            ))
        );
        assert_eq!(
            diff.removed.iter().next(),
            Some(&(
                5usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc")
            ))
        );
    }

    #[test]
    fn test_diff_ipc_add_and_rm_with_same_element() {
        let current = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                3,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
        ]);

        let new = HashSet::from([
            (
                3usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l2_topk_BTCUSDT.ipc"),
            ),
            (
                3,
                arcstr::literal!("ipc:///tmp/binance_spot_l3_topk_BTCUSDT.ipc"),
            ),
            (
                4,
                arcstr::literal!("ipc:///tmp/binance_spot_l5_topk_BTCUSDT.ipc"),
            ),
            (
                5,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc"),
            ),
        ]);

        let diff = super::diff_ipc(current, new);
        assert_eq!(diff.added.len(), 1, "should has 1 added");
        assert_eq!(diff.removed.len(), 1, "should has 1 removed");
        assert_eq!(
            diff.added.iter().next(),
            Some(&(
                5usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l6_topk_BTCUSDT.ipc")
            ))
        );
        assert_eq!(
            diff.removed.iter().next(),
            Some(&(
                4usize,
                arcstr::literal!("ipc:///tmp/binance_spot_l4_topk_BTCUSDT.ipc")
            ))
        );
    }
}