use crate::error::CacheError;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// Constants
const DEFAULT_FILE_CACHE_DIR: &str = ".ocular/cache";
const DEFAULT_FILE_CACHE_NAME: &str = "grpc_endpoints.toml";
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
}

/// Broad cache object that can mange all ocular cache initialization
pub struct Cache {
    pub grpc_endpoint_cache: Box<dyn GrpcCache>,
}

/// Cache accessor defintions
pub trait GrpcCache {
    /// Check if cache has been initialized
    fn is_initialized(&self) -> bool;
    /// Add item to cache
    fn add_item(&mut self, item: String) -> Result<(), CacheError>;
    /// Remove item from cache
    fn remove_item(&mut self, item: String) -> Result<(), CacheError>;
    /// Retrieves a copy of all items from cache
    fn get_all_items(&self) -> Result<HashSet<String>, CacheError>;
}

/// Cache initialization definitions
impl Cache {
    /// Constructor for file cache path. Must include filename with ".toml" suffix for file type.
    ///
    /// If override_if_exists is true it will wipe the previous file found if it exists.
    /// Otherwise if set to false, it will use what is found at the file or create a new one if not found.
    ///
    /// Toml will be required to be structured as so:
    /// [[endpoints]]
    ///     address = "35.230.37.28:9090"
    pub fn create_file_cache(
        file_path: Option<&str>,
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
        if path.extension().unwrap().to_str().unwrap() != "toml" {
            return Err(CacheError::Initialization(String::from(
                "Only toml files supported.",
            )));
        } else if path.is_dir() {
            return Err(CacheError::Initialization(String::from(
                "Path is a dir; must be a file.",
            )));
        }

        // Create files/dirs based on override settings
        // First just create dirs safely regardless of override settings
        let save_path = path.parent().unwrap();

        let dir_res = save_path.metadata();

        // Create dir if doesn't exist
        if dir_res.is_err() {
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

        let mut endpoints = HashSet::new();

        // Load endpoints if they exist
        if std::path::Path::new(&path).exists() {
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
                    endpoints.insert(endpt.address.to_string());
                }
            }
        }

        // Finally we can manipulate the actual file after checking the override settings
        if override_if_exists || !std::path::Path::new(&path).exists() {
            // Note this creates a new file or truncates the existing one
            if let Err(err) = File::create(&path) {
                return Err(CacheError::FileIO(err.to_string()));
            }
        }

        // If patch specified create
        Ok(Cache {
            grpc_endpoint_cache: Box::new(FileCache { path, endpoints }),
        })
    }

    /// Constructor for in memory cache.
    pub fn create_memory_cache(endpoints: Option<HashSet<String>>) -> Result<Cache, CacheError> {
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
    path: PathBuf,
    endpoints: HashSet<String>,
}

impl GrpcCache for FileCache {
    fn is_initialized(&self) -> bool {
        self.path.capacity() != 0
    }

    fn add_item(&mut self, item: String) -> Result<(), CacheError> {
        self.endpoints.insert(item.clone());

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
        toml.endpoints.push(GrpcEndpoint { address: item });

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
        std::fs::write(&self.path, toml_string).expect("Could not write to file.");

        Ok(())
    }

    fn get_all_items(&self) -> Result<HashSet<String>, CacheError> {
        Ok(self.endpoints.clone())
    }
}

/// Memory based cache
pub struct MemoryCache {
    endpoints: HashSet<String>,
}

impl GrpcCache for MemoryCache {
    fn is_initialized(&self) -> bool {
        // No special intialization process so it can always be considered initialized for now.
        true
    }

    fn add_item(&mut self, item: String) -> Result<(), CacheError> {
        self.endpoints.insert(item);

        Ok(())
    }

    fn remove_item(&mut self, item: String) -> Result<(), CacheError> {
        self.endpoints.remove(&item);

        Ok(())
    }

    fn get_all_items(&self) -> Result<HashSet<String>, CacheError> {
        Ok(self.endpoints.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_cache_init() {
        // Get base testing dir
        let base_dir = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();

        let home_dir = dirs::home_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();

        // Testing dir
        let new_dir = &(base_dir + "/cache_test_1");

        // First check default configurations
        let _cache = Cache::create_file_cache(None, false).expect("Could not create cache.");

        let default_location = &(String::from(home_dir)
            + &String::from("/")
            + &String::from(DEFAULT_FILE_CACHE_DIR)
            + &String::from("/")
            + &String::from(DEFAULT_FILE_CACHE_NAME));
        dbg!(default_location);
        assert!(std::path::Path::new(default_location).exists());

        // Attempt to create new directory
        // Make sure new dir DNE.
        let test_filepath = &(String::from(new_dir) + "/test.toml");
        assert!(!std::path::Path::new(test_filepath).exists());

        // Create new without override
        let cache =
            Cache::create_file_cache(Some(test_filepath), false).expect("Could not create cache.");

        // Write to file a bit to test overrides
        let mut file = GrpcEndpointToml::default();
        file.endpoints.push(GrpcEndpoint {
            address: String::from("localhost:8080"),
        });
        let toml_string = toml::to_string(&file).expect("Could not encode toml value.");

        dbg!(&toml_string);
        dbg!(&test_filepath);

        std::fs::write(&test_filepath, toml_string).expect("Could not write to file.");

        // Store contents of file
        let file_output = std::fs::read_to_string(test_filepath).expect("Could not read file.");

        // Make sure it's not empty
        assert!(!file_output.is_empty());

        // Verify with override false, file contents still exists
        let cache_2 =
            Cache::create_file_cache(Some(test_filepath), false).expect("Could not create cache.");
        let file_output_check =
            std::fs::read_to_string(test_filepath).expect("Could not read file.");

        // Assert not empty and equals old file
        assert!(!file_output_check.is_empty());
        assert_eq!(file_output_check, file_output);

        // Test override
        let cache_3 =
            Cache::create_file_cache(Some(test_filepath), true).expect("Could not create cache.");

        // Verify file contents was overriden
        let file_override_check =
            std::fs::read_to_string(test_filepath).expect("Could not read file.");
        assert!(file_override_check.is_empty());

        // Finally check initialization methods
        assert!(cache.grpc_endpoint_cache.is_initialized());
        assert!(cache_2.grpc_endpoint_cache.is_initialized());
        assert!(cache_3.grpc_endpoint_cache.is_initialized());

        // Clean up testing dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn file_cache_accessor_test() {
        // Use ad hoc testing dir + file.
        let testing_dir = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            + "/cache_test_2";
        let test_file = &(String::from(&testing_dir) + "/test.toml");

        let mut cache =
            Cache::create_file_cache(Some(test_file), true).expect("Could not create cache");

        // Assert cache is empty to start in both memory and file
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .is_empty());
        assert!(std::fs::read_to_string(&test_file)
            .expect("Could not open file.")
            .is_empty());

        // Insert item
        cache
            .grpc_endpoint_cache
            .add_item(String::from("localhost:9090"))
            .expect("Could not add item to cache.");

        // Verify item exists in both memory and file
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .contains(&String::from("localhost:9090")));
        let contents = std::fs::read_to_string(&test_file).expect("Could not open file.");
        let toml: GrpcEndpointToml = toml::from_str(&contents).expect("Could not parse toml.");
        assert!(
            toml.endpoints.len() == 1
                && toml.endpoints[0].address == String::from("localhost:9090")
        );

        assert!(cache
            .grpc_endpoint_cache
            .add_item(String::from("localhost:9090"))
            .is_ok());

        // Remove item
        cache
            .grpc_endpoint_cache
            .remove_item(String::from("localhost:9090"))
            .expect("Could not remove item from cache.");

        // Verify removed in both memory and file
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .is_empty());
        let contents = std::fs::read_to_string(&test_file).expect("Could not open file.");
        let toml: GrpcEndpointToml = toml::from_str(&contents).expect("Could not parse toml.");
        assert!(toml.endpoints.len() == 0);

        assert!(cache
            .grpc_endpoint_cache
            .remove_item(String::from("localhost:9090"))
            .is_ok());

        // Clean up testing dir
        std::fs::remove_dir_all(&testing_dir)
            .expect(&format!("Failed to delete test directory {}", &testing_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(testing_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn memory_cache_init() {
        // Attempt creation with no endpoints
        assert!(Cache::create_memory_cache(None).is_ok());

        // Attempt creation with some endpoints
        let mut endpts = HashSet::new();
        endpts.insert(String::from("localhost"));

        let cache = Cache::create_memory_cache(Some(endpts));

        assert!(cache.is_ok());

        // Check initialization
        assert!(cache.unwrap().grpc_endpoint_cache.is_initialized())
    }

    #[test]
    fn memory_cache_accessor_test() {
        let mut cache = Cache::create_memory_cache(None).expect("Could not create cache");

        // Assert cache is empty to start
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .is_empty());

        // Insert item
        cache
            .grpc_endpoint_cache
            .add_item(String::from("localhost:9090"))
            .expect("Could not add item to cache.");

        // Verify item exists
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .contains(&String::from("localhost:9090")));

        assert!(cache
            .grpc_endpoint_cache
            .add_item(String::from("localhost:9090"))
            .is_ok());

        // Remove item
        cache
            .grpc_endpoint_cache
            .remove_item(String::from("localhost:9090"))
            .expect("Could not remove item from cache.");

        // Verify removed
        assert!(cache
            .grpc_endpoint_cache
            .get_all_items()
            .expect("Could not get cache contents.")
            .is_empty());

        assert!(cache
            .grpc_endpoint_cache
            .remove_item(String::from("localhost:9090"))
            .is_ok());
    }
}
