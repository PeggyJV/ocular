use crate::error::CacheError;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// Constants
pub const DEFAULT_FILE_CACHE_DIR: &str = ".ocular/cache";
pub const DEFAULT_FILE_CACHE_NAME: &str = "grpc_endpoints.toml";
/// Unix permissions for dir
const FILE_CACHE_DIR_PERMISSIONS: u32 = 0o700;

// Toml structs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GrpcEndpointToml {
    pub endpoints: Vec<GrpcEndpoint>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GrpcEndpoint {
    pub address: String,
    pub connsecutive_failed_connections: u8,
}

/// Broad cache object that can mange all ocular cache initialization
pub struct Cache {
    pub grpc_endpoint_cache: Box<dyn GrpcCache>,
}

/// Cache accessor defintions
pub trait GrpcCache {
    /// Check if cache has been initialized
    fn is_initialized(&self) -> bool;
    /// Add item to cache, overrides connsecutive_failed_connections if item already exists
    fn add_item(
        &mut self,
        item: String,
        connsecutive_failed_connections: u8,
    ) -> Result<(), CacheError>;
    /// Remove item from cache
    fn remove_item(&mut self, item: String) -> Result<(), CacheError>;
    /// Increments connsecutive_failed_connections if item already exists, or creates item with 1 failed connection if it DNE
    fn increment_failed_connections(&mut self, item: String) -> Result<(), CacheError>;
    /// Retrieves a copy of all items from cache
    fn get_all_items(&self) -> Result<HashMap<String, u8>, CacheError>;
    /// Retrieves connections failure threshold
    fn get_connsecutive_failed_connections_threshold(&self) -> u8;
}

/// Cache initialization definitions
impl Cache {
    /// Constructor for file cache path. Must include filename with ".toml" suffix for file type.
    ///
    /// If override_if_exists is true it will wipe the previous file found if it exists.
    /// Otherwise if set to false, it will use what is found at the file or create a new one if not found.
    ///
    /// Toml will be required to be structured as so:
    ///
    /// ```toml
    /// [[endpoints]]
    ///     address = "35.230.37.28:9090"
    ///     connsecutive_failed_connections = 0
    /// ```
    pub fn create_file_cache(
        file_path: Option<&str>,
        connsecutive_failed_connections_threshold: u8,
        override_if_exists: bool,
    ) -> Result<Cache, CacheError> {
        // If none, create at default: (e.g. ~/.ocular/grpc_endpoints.toml)
        let path: PathBuf = match file_path {
            Some(path) => PathBuf::from(path),
            None => {
                let mut p = dirs::home_dir().unwrap();

                p.push(DEFAULT_FILE_CACHE_DIR);
                p.push(DEFAULT_FILE_CACHE_NAME);

                p
            }
        };

        dbg!(format!("Attempting to use path {:#?}", path));

        // Verify path formatting
        if path.is_dir() {
            return Err(CacheError::Initialization(String::from(
                "Path is a dir; must be a file.",
            )));
        } else if path.extension().unwrap().to_str().unwrap() != "toml" {
            return Err(CacheError::Initialization(String::from(
                "Only files with extension .toml are supported.",
            )));
        }

        // Create files/dirs based on override settings
        // First just create dirs safely regardless of override settings
        let save_path = path.parent().unwrap();

        // Create dir if doesn't exist
        if !save_path.exists() {
            match std::fs::create_dir_all(save_path) {
                Ok(_res) => _res,
                Err(err) => return Err(CacheError::FileIO(err.to_string())),
            };
        }

        #[cfg(unix)]
        match std::fs::set_permissions(
            save_path,
            std::fs::Permissions::from_mode(FILE_CACHE_DIR_PERMISSIONS),
        ) {
            Ok(_res) => _res,
            Err(err) => return Err(CacheError::FileIO(err.to_string())),
        };

        let mut endpoints = HashMap::new();

        // Load endpoints if they exist
        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(result) => result,
                Err(err) => {
                    return Err(CacheError::FileIO(err.to_string()));
                }
            };

            // Possible contents is empty, check to avoid parsing errors
            if !content.is_empty() {
                let toml: GrpcEndpointToml = match toml::from_str(&content) {
                    Ok(result) => result,
                    Err(err) => {
                        return Err(CacheError::Toml(err.to_string()));
                    }
                };

                dbg!(&toml);

                for endpt in &toml.endpoints {
                    endpoints.insert(
                        endpt.address.to_string(),
                        endpt.connsecutive_failed_connections,
                    );
                }
            }
        }

        // Finally we can manipulate the actual file after checking the override settings
        if override_if_exists || !path.exists() {
            // Note this creates a new file or truncates the existing one
            if let Err(err) = File::create(&path) {
                return Err(CacheError::FileIO(err.to_string()));
            }
        }

        Ok(Cache {
            grpc_endpoint_cache: Box::new(FileCache {
                path,
                endpoints,
                connsecutive_failed_connections_threshold,
            }),
        })
    }

    /// Constructor for in memory cache.
    pub fn create_memory_cache(
        endpoints: Option<HashMap<String, u8>>,
        connsecutive_failed_connections_threshold: u8,
    ) -> Result<Cache, CacheError> {
        let cache = match endpoints {
            Some(endpoints) => MemoryCache {
                endpoints,
                connsecutive_failed_connections_threshold,
            },
            None => MemoryCache {
                endpoints: HashMap::new(),
                connsecutive_failed_connections_threshold,
            },
        };

        Ok(Cache {
            grpc_endpoint_cache: Box::new(cache),
        })
    }
}

/// File based cache
pub struct FileCache {
    path: PathBuf,
    endpoints: HashMap<String, u8>,
    connsecutive_failed_connections_threshold: u8,
}

impl GrpcCache for FileCache {
    fn is_initialized(&self) -> bool {
        self.path.capacity() != 0
    }

    fn add_item(
        &mut self,
        item: String,
        connsecutive_failed_connections: u8,
    ) -> Result<(), CacheError> {
        self.endpoints
            .insert(item.clone(), connsecutive_failed_connections);

        let content = match std::fs::read_to_string(&self.path) {
            Ok(result) => result,
            Err(err) => {
                return Err(CacheError::FileIO(err.to_string()));
            }
        };

        let mut toml: GrpcEndpointToml = GrpcEndpointToml::default();

        // Possible contents is empty, check to avoid parsing errors
        if !content.is_empty() {
            toml = match toml::from_str(&content) {
                Ok(result) => result,
                Err(err) => {
                    return Err(CacheError::Toml(err.to_string()));
                }
            };

            dbg!(&toml);
        }

        // Add new item
        toml.endpoints.push(GrpcEndpoint {
            address: item,
            connsecutive_failed_connections,
        });

        let toml_string = toml::to_string(&toml).expect("Could not encode toml value.");

        dbg!(&toml_string);

        // Rewrite file
        match std::fs::write(&self.path, toml_string) {
            Ok(_) => Ok(()),
            Err(err) => Err(CacheError::FileIO(err.to_string())),
        }
    }

    fn remove_item(&mut self, item: String) -> Result<(), CacheError> {
        self.endpoints.remove(&item);

        let mut toml: GrpcEndpointToml = GrpcEndpointToml::default();

        let content = match std::fs::read_to_string(&self.path) {
            Ok(result) => result,
            Err(err) => {
                return Err(CacheError::FileIO(err.to_string()));
            }
        };

        // If we were able to get to this point, contents should never be empty, so no need to check.
        let old_toml: GrpcEndpointToml = match toml::from_str(&content) {
            Ok(result) => result,
            Err(err) => {
                return Err(CacheError::Toml(err.to_string()));
            }
        };

        dbg!(&old_toml);

        // Remove item by virtue of excludng it from new toml
        toml.endpoints = old_toml
            .endpoints
            .into_iter()
            .filter(|ep| ep.address.trim() != item.as_str())
            .collect();

        let toml_string = toml::to_string(&toml).expect("Could not encode toml value.");

        dbg!(&toml_string);

        // Rewrite file
        match std::fs::write(&self.path, toml_string) {
            Ok(_) => Ok(()),
            Err(err) => Err(CacheError::FileIO(err.to_string())),
        }
    }

    fn get_all_items(&self) -> Result<HashMap<String, u8>, CacheError> {
        Ok(self.endpoints.clone())
    }

    fn increment_failed_connections(&mut self, item: String) -> Result<(), CacheError> {
        if self.endpoints.contains_key(&item) {
            self.endpoints
                .insert(item.clone(), self.endpoints.get(&item).unwrap() + 1);
        } else {
            self.endpoints.insert(item.clone(), 1);
        }

        // Check if element now at removal threshold
        if self.endpoints.get(&item).unwrap() >= &self.connsecutive_failed_connections_threshold {
            self.remove_item(item)
                .expect("Error removing item from cache.");
        }

        // Update file
        let mut toml: GrpcEndpointToml = GrpcEndpointToml::default();

        for endpt in &self.endpoints {
            toml.endpoints.push(GrpcEndpoint {
                address: endpt.0.to_string(),
                connsecutive_failed_connections: *endpt.1,
            });
        }

        let toml_string = toml::to_string(&toml).expect("Could not encode toml value.");

        // Rewrite file
        match std::fs::write(&self.path, toml_string) {
            Ok(_) => Ok(()),
            Err(err) => Err(CacheError::FileIO(err.to_string())),
        }
    }

    fn get_connsecutive_failed_connections_threshold(&self) -> u8 {
        self.connsecutive_failed_connections_threshold
    }
}

/// Memory based cache
pub struct MemoryCache {
    endpoints: HashMap<String, u8>,
    connsecutive_failed_connections_threshold: u8,
}

impl GrpcCache for MemoryCache {
    fn is_initialized(&self) -> bool {
        // No special intialization process so it can always be considered initialized for now.
        true
    }

    fn add_item(
        &mut self,
        item: String,
        connsecutive_failed_connections: u8,
    ) -> Result<(), CacheError> {
        self.endpoints.insert(item, connsecutive_failed_connections);

        Ok(())
    }

    fn remove_item(&mut self, item: String) -> Result<(), CacheError> {
        self.endpoints.remove(&item);

        Ok(())
    }

    fn get_all_items(&self) -> Result<HashMap<String, u8>, CacheError> {
        Ok(self.endpoints.clone())
    }

    fn increment_failed_connections(&mut self, item: String) -> Result<(), CacheError> {
        if self.endpoints.contains_key(&item) {
            self.endpoints
                .insert(item.clone(), self.endpoints.get(&item).unwrap() + 1);
        } else {
            self.endpoints.insert(item.clone(), 1);
        }

        // Check if element now at removal threshold
        if self.endpoints.get(&item).unwrap() >= &self.connsecutive_failed_connections_threshold {
            self.remove_item(item)
                .expect("Error removing item from cache.");
        }

        Ok(())
    }

    fn get_connsecutive_failed_connections_threshold(&self) -> u8 {
        self.connsecutive_failed_connections_threshold
    }
}
