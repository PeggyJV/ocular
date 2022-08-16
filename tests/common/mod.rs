use ocular::QueryClient;

pub const HUB_TEST_ADDRESS: &str = "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08";

pub fn new_cosmos_client() -> QueryClient {
    QueryClient::new(
        "http://cosmoshub.strange.love:26657",
        "http://cosmoshub.strange.love:9090",
    )
    .expect("failed to construct Cosmos client")
}
