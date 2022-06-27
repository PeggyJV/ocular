use crate::{
    chain::client::{query::BankQueryClient, ChainClient},
    error::{ChainInfoError, GrpcError, RpcError},
    registry, utils,
};
use rand::prelude::SliceRandom;
use rand::thread_rng;

impl ChainClient {
    pub async fn get_random_grpc_endpoint(&mut self) -> Result<String, ChainInfoError> {
        let endpoints = self.get_grpc_endpoints().await?;
        if let Some(endpoint) = endpoints.choose(&mut thread_rng()) {
            Ok(endpoint.to_string())
        } else {
            Err(RpcError::UnhealthyEndpoint("no available RPC endpoints".to_string()).into())
        }
    }

    pub async fn get_grpc_endpoints(&mut self) -> Result<Vec<String>, ChainInfoError> {
        let mut endpoints: Vec<String> = Vec::new();
        let mut refresh_cache = false;

        // Check if cache exists, if it doesn't, pull new cache every time
        if self.cache.is_some() {
            endpoints = self
                .cache
                .as_ref()
                .unwrap()
                .grpc_endpoint_cache
                .get_all_items()
                .expect("Could not access cache.")
                .keys()
                .cloned()
                .collect();
        }

        // Get api endpoints if cache was entirely empty or if caching is disabled
        if endpoints.is_empty() {
            endpoints = self.get_all_grpc_endpoints().await;
            if endpoints.is_empty() {
                return Err(GrpcError::MissingEndpoint(
                    "no valid endpoint found. endpoints must use http or https.".to_string(),
                )
                .into());
            }

            // this is not very efficient but i was getting annoyed trying to figure
            // out how to do filtering with an async method
            for (i, ep) in endpoints.clone().iter().enumerate() {
                if self.is_healthy_grpc(ep.as_str()).await.is_err() {
                    endpoints.remove(i);
                }
            }

            refresh_cache = true;
        }

        if endpoints.is_empty() {
            return Err(GrpcError::UnhealthyEndpoint(
                "no healthy endpoint found (connections could not be established)".to_string(),
            )
            .into());
        }

        // If cache being used and we had to refresh it, load new endpoints into it
        if self.cache.is_some() && refresh_cache {
            let thresh = self
                .cache
                .as_mut()
                .unwrap()
                .grpc_endpoint_cache
                .get_connsecutive_failed_connections_threshold();

            for endpt in &endpoints {
                let _res = self
                    .cache
                    .as_mut()
                    .expect("Error accessing cache.")
                    .grpc_endpoint_cache
                    .add_item(endpt.to_string(), thresh)?;
            }
        }

        Ok(endpoints)
    }

    async fn get_all_grpc_endpoints(&self) -> Vec<String> {
        let info = registry::get_chain(&self.config.chain_name)
            .await
            .expect("Could not get chain info.")
            .expect("Could not get chain info.");

        info.apis
            .grpc
            .iter()
            .filter_map(|grpc| utils::parse_or_build_grpc_endpoint(grpc.address.as_str()).ok())
            .filter(|uri| !uri.is_empty())
            .collect()
    }

    pub async fn is_healthy_grpc(&self, endpoint: &str) -> Result<(), ChainInfoError> {
        if BankQueryClient::connect(endpoint.to_string())
            .await
            .is_err()
        {
            return Err(
                GrpcError::UnhealthyEndpoint(format!("{} failed health check", endpoint)).into(),
            );
        }

        Ok(())
    }
}
