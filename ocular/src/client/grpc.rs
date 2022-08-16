use std::any::TypeId;

use crate::client::ChainClient;

impl ChainClient {
    pub fn has_grpc_client<T: 'static>(&self) -> bool {
        let key = TypeId::of::<T>();
        self.grpc_pool.contains_key(&key)
    }
}
