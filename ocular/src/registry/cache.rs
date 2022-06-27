use crate::{
    registry::{self, paths::IBCPath},
    ChainRegistryError,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use super::paths::Tag;

lazy_static! {
    static ref REGISTRY_CACHE: Arc<RwLock<RegistryCache>> =
        Arc::<RwLock::<RegistryCache>>::default();
}

// TO-DO:
// - Option to load from local repo clone
// - Currently don't see a need to cache chain/asset info but might need it in the future
#[derive(Default, Deserialize, Serialize)]
pub struct RegistryCache {
    paths: HashMap<String, IBCPath>,
    path_names: Vec<String>,
}

/// Used to cache IBC path data from the chain registry for easy filtering. A static cache is initialized automatically.
/// To acquire a read lock for it, use [`get_read_lock()`]
impl RegistryCache {
    pub fn is_initialized(&self) -> bool {
        !self.path_names.is_empty()
    }

    /// Returns a cached [`IBCPath`] representing a channel between [`chain_a`] and `chain_b` if it exists
    pub async fn get_path(
        &self,
        chain_a: &str,
        chain_b: &str,
    ) -> Result<Option<IBCPath>, ChainRegistryError> {
        if !self.is_initialized() {
            Self::initialize().await?;
        }

        let path_name = match chain_a.cmp(&chain_b) {
            Ordering::Less => chain_a.to_string() + "-" + &chain_b,
            Ordering::Equal => return Ok(None),
            Ordering::Greater => chain_b.to_string() + "-" + &chain_a,
        };

        Ok(self.paths.get(&path_name).cloned())
    }

    /// Returns cached [`IBCPath`] that match a provided [`Tag`]
    pub async fn get_paths_filtered(&self, tag: Tag) -> Result<Vec<IBCPath>, ChainRegistryError> {
        if !self.is_initialized() {
            Self::initialize().await?;
        }

        Ok(self
            .paths
            .iter()
            .filter(|kv| match &tag {
                Tag::Dex(d) => kv.1.channels[0].tags.dex.eq(d),
                Tag::Preferred(p) => kv.1.channels[0].tags.preferred.eq(p),
                Tag::Properties(p) => kv.1.channels[0].tags.properties.eq(p),
                Tag::Status(s) => kv.1.channels[0].tags.status.eq(s),
            })
            .map(|kv| kv.1.to_owned())
            .collect())
    }

    /// Gets a read lock on the static [`RegistryCache`]. The cache can be pre-loaded with IBC path data using the
    /// [`load_ibc_paths()`] method, otherwise it will populate the first time data is requested from it.
    pub async fn get_read_lock() -> RwLockReadGuard<'static, RegistryCache> {
        REGISTRY_CACHE.read().await
    }

    /// Retrieves, deserializes, and caches each [`IBCPath`] from the Cosmos Chain Registry for easy filtering
    pub async fn initialize() -> Result<(), ChainRegistryError> {
        let mut cache = REGISTRY_CACHE.write().await;
        let path_names = registry::list_paths().await?;
        cache.path_names = path_names.clone();

        for path_name in path_names {
            let pn = path_name.clone();
            let cn: Vec<&str> = pn.split('-').collect();

            // this unwrap is safe becauase we query the path directly from the list of path .json file names
            // retrieved earlier, therefore the Option returned should never be None.
            cache.paths.insert(
                path_name,
                registry::get_path(cn[0], cn[1])
                    .await?
                    .expect("path returned None"),
            );
        }

        Ok(())
    }
}
