use ocular::grpc::GrpcClient;

pub const HUB_TEST_ADDRESS: &str = "cosmos1n6j7gnld9yxfyh6tflxhjjmt404zruuaf73t08";

pub async fn new_cosmos_client() -> GrpcClient {
    GrpcClient::new("https://cosmos-grpc.polkachu.com:14990")
        .await
        .unwrap()
}
