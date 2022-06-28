use crate::{
    registry::{self, paths::IBCPath},
    ChainRegistryError,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

use super::paths::Tag;

// TO-DO:
// - Option to load from local repo clone
// - Currently don't see a need to cache chain/asset info but might need it in the future
#[derive(Default, Deserialize, Serialize)]
pub struct RegistryCache {
    paths: HashMap<String, IBCPath>,
}

/// Used to cache IBC path data from the chain registry for easy filtering.
impl RegistryCache {
    /// Returns a cached [`IBCPath`] representing a channel between [`chain_a`] and `chain_b` if it exists
    pub async fn get_path(
        &self,
        chain_a: &str,
        chain_b: &str,
    ) -> Result<Option<IBCPath>, ChainRegistryError> {
        let path_name = match chain_a.cmp(chain_b) {
            Ordering::Less => chain_a.to_string() + "-" + chain_b,
            Ordering::Equal => return Ok(None),
            Ordering::Greater => chain_b.to_string() + "-" + chain_a,
        };

        Ok(self.paths.get(&path_name).cloned())
    }

    /// Returns cached [`IBCPath`] that match a provided [`Tag`]
    pub async fn get_paths_filtered(&self, tag: Tag) -> Result<Vec<IBCPath>, ChainRegistryError> {
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

    /// Creates a new cache by retrieving and deserializing each [`IBCPath`] from the Cosmos Chain Registry for easy filtering
    pub async fn try_new() -> Result<RegistryCache, ChainRegistryError> {
        let path_names = registry::list_paths().await?;
        let mut paths = HashMap::<String, IBCPath>::default();

        for pn in path_names {
            let cn: Vec<&str> = pn.split('-').collect();

            // this unwrap is safe becauase we query the path directly from the list of path .json file names
            // retrieved earlier, therefore the Option returned should never be None.
            paths.insert(
                pn.clone(),
                registry::get_path(cn[0], cn[1])
                    .await?
                    .expect("path returned None"),
            );
        }

        Ok(RegistryCache { paths })
    }
}
