use crate::error::CacheError;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;

// Constants
const DEFAULT_FILE_CACHE_DIR: &str = "/.ocular/cache";
const DEFAULT_FILE_CACHE_NAME: &str = "grpc_endpoints.toml";
/// Unix permissions for dir
const FILE_CACHE_DIR_PERMISSIONS: u32 = 0o700;

// Toml structs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GrpcEndpointToml<'a> {
    #[serde(borrow)]
    pub endpoint: Vec<GrpcEndpoint<'a>>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GrpcEndpoint<'a> {
    pub address: &'a str,
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
    fn add_item(&self, item: String) -> Result<(), CacheError>;
    /// Remove item from cache
    fn remove_item(&self, item: String) -> Result<(), CacheError>;
    /// Retrieves all items from cache
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
    /// [[endpoint]]
    ///     address = "35.230.37.28:9090"
    pub fn create_file_cache(
        file_path: Option<&str>,
        override_if_exists: bool,
    ) -> Result<Cache, CacheError> {
        // If none, create at default: (e.g. ~/.ocular/grpc_endpoints.toml)
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
        let _cache =
            Cache::create_file_cache(Some(test_filepath), false).expect("Could not create cache.");

        // Write to file a bit to test overrides
        let mut file = GrpcEndpointToml::default();
        file.endpoint.push(GrpcEndpoint {
            address: "localhost:8080",
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
        let _cache_2 =
            Cache::create_file_cache(Some(test_filepath), false).expect("Could not create cache.");
        let file_output_check =
            std::fs::read_to_string(test_filepath).expect("Could not read file.");

        // Assert not empty and equals old file
        assert!(!file_output_check.is_empty());
        assert_eq!(file_output_check, file_output);

        // Test override
        let _cache_3 =
            Cache::create_file_cache(Some(test_filepath), true).expect("Could not create cache.");

        // Verify file contents was overriden
        let file_override_check =
            std::fs::read_to_string(test_filepath).expect("Could not read file.");
        assert!(file_override_check.is_empty());

        // Clean up testing dir
        std::fs::remove_dir_all(new_dir)
            .expect(&format!("Failed to delete test directory {}", new_dir));

        // Assert deleted
        let result = std::panic::catch_unwind(|| std::fs::metadata(new_dir).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn memory_cache_init() {
        // Attempt creation with no endpoints
        assert!(Cache::create_memory_cache(None).is_ok());

        // Attempt creation with some endpoints
        let mut endpts = HashSet::new();
        endpts.insert(String::from("localhost"));

        assert!(Cache::create_memory_cache(Some(endpts)).is_ok());
    }
}
