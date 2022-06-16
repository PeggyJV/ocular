use cosmrs::dev;
use ocular::chain::client::cache::*;
use std::collections::HashSet;

/// Chain ID to use for tests
const CHAIN_ID: &str = "cosmrs-test";

// We don't actually need the gaia node, but reusing it here for simplicty since single node chain test already has it configured.
// Also provides us with a simple way to interact with a single node chain in this test if ever desired.
const DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE: &str = "philipjames11/gaia-test";

#[test]
fn file_cache_init() {
    let docker_args = [
        "-d",
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        "test",
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
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
            let cache = Cache::create_file_cache(Some(test_filepath), false)
                .expect("Could not create cache.");

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
            let cache_2 = Cache::create_file_cache(Some(test_filepath), false)
                .expect("Could not create cache.");
            let file_output_check =
                std::fs::read_to_string(test_filepath).expect("Could not read file.");

            // Assert not empty and equals old file
            assert!(!file_output_check.is_empty());
            assert_eq!(file_output_check, file_output);

            // Test override
            let cache_3 = Cache::create_file_cache(Some(test_filepath), true)
                .expect("Could not create cache.");

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
        });
    });
}

#[test]
fn file_cache_accessor_test() {
    let docker_args = [
        "-d",
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        "test",
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
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
        });
    });
}

#[test]
fn memory_cache_init() {
    let docker_args = [
        "-d",
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        "test",
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
            // Attempt creation with no endpoints
            assert!(Cache::create_memory_cache(None).is_ok());

            // Attempt creation with some endpoints
            let mut endpts = HashSet::new();
            endpts.insert(String::from("localhost"));

            let cache = Cache::create_memory_cache(Some(endpts));

            assert!(cache.is_ok());

            // Check initialization
            assert!(cache.unwrap().grpc_endpoint_cache.is_initialized())
        });
    });
}

#[test]
fn memory_cache_accessor_test() {
    let docker_args = [
        "-d",
        DOCKER_HUB_GAIA_SINGLE_NODE_TEST_IMAGE,
        CHAIN_ID,
        "test",
    ];

    dev::docker_run(&docker_args, || {
        init_tokio_runtime().block_on(async {
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
        });
    });
}

/// Initialize Tokio runtime
fn init_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime")
}
