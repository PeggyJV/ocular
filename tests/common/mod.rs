use ocular::QueryClient;

pub const HUB_TEST_ADDRESS: &str = "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08";

pub fn new_cosmos_client() -> QueryClient {
    QueryClient::new("https://cosmos-grpc.polkachu.com:14990")
}
