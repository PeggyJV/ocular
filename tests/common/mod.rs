use ocular::QueryClient;

pub const HUB_TEST_ADDRESS: &str = "cosmos154d0p9xhrruhxvazumej9nq29afeura2alje4u";

pub fn new_cosmos_client() -> QueryClient {
    QueryClient::new(
        "http://cosmoshub.strange.love:26657",
        "http://cosmoshub.strange.love:9090",
    )
    .expect("failed to construct Cosmos client")
}
