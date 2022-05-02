use crate::error::CacheError;

use std::collections::HashSet;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;

use super::ChainClient;

// Constants
const DEFAULT_FILE_CACHE_DIR: &str = "/.ocular/cache";
const DEFAULT_FILE_CACHE_NAME: &str = "grpc_endpoints.toml";
/// Unix permissions for dir
const FILE_CACHE_DIR_PERMISSIONS: u32 = 0o700;

/// Broad cache object that can mange all ocular cache initialization
pub struct Cache {
    pub grpc_endpoint_cache: Box<dyn GrpcCache>,
}

/// Cache accessor defintions
pub trait GrpcCache {
    /// Check if cache has been initialized
    fn is_initialized(&self) -> bool;
    /// Add item to cache
    fn add_item(&self, item: String) -> Result<(), CacheError>;
    /// Remove item from cache
    fn remove_item(&self, item: String) -> Result<(), CacheError>;
    /// Retrieves all items from cache
    fn get_all_items(&self) -> Result<HashSet<String>, CacheError>;
}

/// Cache initialization definitions
impl ChainClient {
    /// Constructor for file cache path. Must include filename with ".toml" suffix for file type.
    ///
    /// If override_if_exists is true it will wipe the previous file found if it exists.
    /// Otherwise if set to false, it will use what is found at the file or create a new one if not found.
    ///
    /// Toml will be required to be structured as so:
    /// [[endpoint]]
    ///     address = "35.230.37.28:9090"
    pub fn create_file_cache(
        &mut self,
        file_path: Option<&str>,
        override_if_exists: bool,
    ) -> Result<Cache, CacheError> {
        // If none, create at default, albeit with chain specific filename (e.g. ~/.ocular/sommelier_grpc_endpoints.toml)
        let path: String = if let Some(file_path) = file_path {
            file_path.to_string()
        } else {
            dirs::home_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
                + DEFAULT_FILE_CACHE_DIR
                + "/"
                + &self.config.chain_id
                + "_"
                + DEFAULT_FILE_CACHE_NAME
        };

        dbg!(format!("Attempting to use path {}", path));

        // Verify path formatting
        if !path.ends_with(".toml") {
            return Err(CacheError::Initialization(String::from(
                "Only toml files supported.",
            )));
        }

        // Create files/dirs based on override settings
        // First just create dirs safely regardless of override settings
        let save_path = &path[..path
            .to_string()
            .rfind('/')
            .expect("Could not process string.")];

        let dir_res = std::path::Path::new(&save_path).metadata();

        // Create dir if doesn't exist
        if dir_res.is_err() {
            match std::fs::create_dir_all(&save_path) {
                Ok(_res) => _res,
                Err(err) => return Err(CacheError::FileIO(err.to_string())),
            };
        }

        #[cfg(unix)]
        match std::fs::set_permissions(
            &save_path,
            std::fs::Permissions::from_mode(FILE_CACHE_DIR_PERMISSIONS),
        ) {
            Ok(_res) => _res,
            Err(err) => return Err(CacheError::FileIO(err.to_string())),
        };

        // Finally we can manipulate the actual file after checking the override settings
        if override_if_exists || !std::path::Path::new(&path).exists() {
            // Note this creates a new file or truncates the existing one
            let _res = match File::create(&path) {
                Ok(res) => res,
                Err(err) => return Err(CacheError::FileIO(err.to_string())),
            };
        }

        // If patch specified create
        Ok(Cache {
            grpc_endpoint_cache: Box::new(FileCache { path }),
        })
    }

    /// Constructor for in memory cache.
    pub fn create_memory_cache(
        &mut self,
        endpoints: Option<HashSet<String>>,
    ) -> Result<Cache, CacheError> {
        let cache = match endpoints {
            Some(endpoints) => MemoryCache { endpoints },
            None => MemoryCache {
                endpoints: HashSet::new(),
            },
        };

        Ok(Cache {
            grpc_endpoint_cache: Box::new(cache),
        })
    }
}

/// File based cache
pub struct FileCache {
    path: String,
}

impl GrpcCache for FileCache {
    fn is_initialized(&self) -> bool {
        todo!()
    }

    fn add_item(&self, item: String) -> Result<(), CacheError> {
        todo!()
    }

    fn remove_item(&self, item: String) -> Result<(), CacheError> {
        todo!()
    }

    fn get_all_items(&self) -> Result<HashSet<String>, CacheError> {
        todo!()
    }
}

/// Memory based cache
pub struct MemoryCache {
    endpoints: HashSet<String>,
}

impl GrpcCache for MemoryCache {
    fn is_initialized(&self) -> bool {
        todo!()
    }

    fn add_item(&self, item: String) -> Result<(), CacheError> {
        todo!()
    }

    fn remove_item(&self, item: String) -> Result<(), CacheError> {
        todo!()
    }

    fn get_all_items(&self) -> Result<HashSet<String>, CacheError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_cache_init() {}

    #[test]
    fn memory_cache_init() {}
}
