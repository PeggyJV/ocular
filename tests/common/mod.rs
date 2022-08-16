use ocular::QueryClient;

pub fn new_cosmos_client() -> QueryClient {
    QueryClient::new(
        "http://cosmoshub.strange.love:26657",
        "http://cosmoshub.strange.love:9090",
    )
    .expect("failed to construct Cosmos client")
}
