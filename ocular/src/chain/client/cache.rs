use std::collections::HashSet;

use crate::error::CacheError;

/// Broad cache object that can mange all ocular cache initialization
pub struct Cache {
    grpcEndpointCache: Box<dyn GrpcCache>,
}

/// Cache accessor defintions 
pub trait GrpcCache {
    fn is_initialized(&self) -> bool;
    fn add_item(&self, item: String) -> Result<(), CacheError>;
    fn remove_item(&self, item: String) -> Result<(), CacheError>;
}

/// Cache initialization definitions
impl Cache {
    pub fn is_cache_initialized(&self) -> bool {
        Ok(())
    }

    pub fn create_file_cache(&self, path: Option<String>) -> Result<Cache, CacheError> {
        Ok(())
    }
 
    pub fn create_memory_cache(&self) -> Result<Cache, CacheError> {
        Ok(())
    }
}

/// File based cache
pub struct FileCache {
    path: String,
}

impl GrpcCache for FileCache {
}

/// Memory based cache
pub struct MemoryCache {
    endpoints: HashSet<String>,
}

impl GrpcCache for MemoryCache {
}
