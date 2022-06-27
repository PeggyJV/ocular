use assay::assay;
use ocular::registry::{cache::RegistryCache, paths::Tag};

#[assay]
async fn registry_cache_happy_path() {
    assert!(RegistryCache::initialize().await.is_ok());

    let cache = RegistryCache::get_read_lock().await;
    assert!(cache.is_initialized());

    let chain_a = "cosmoshub";
    let chain_b = "osmosis";
    let result = cache
        .get_path(chain_a, chain_b)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.chain_1.chain_name, "cosmoshub");
    assert_eq!(result.chain_2.chain_name, "osmosis");

    // reverse order
    let result = cache
        .get_path(chain_b, chain_a)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.chain_1.chain_name, "cosmoshub");
    assert_eq!(result.chain_2.chain_name, "osmosis");

    // filter by tag values known to be present in at least one chain
    let dex = "osmosis".to_string();
    let result = cache
        .get_paths_filtered(Tag::Dex(dex.clone()))
        .await
        .unwrap();
    assert!(!result.is_empty());
    result
        .iter()
        .for_each(|r| assert!(r.channels[0].tags.dex.eq(&dex)));

    let preferred = true;
    let result = cache
        .get_paths_filtered(Tag::Preferred(preferred))
        .await
        .unwrap();
    assert!(!result.is_empty());
    result
        .iter()
        .for_each(|r| assert!(r.channels[0].tags.preferred.eq(&preferred)));

    let status = "live".to_string();
    let result = cache
        .get_paths_filtered(Tag::Status(status.clone()))
        .await
        .unwrap();
    assert!(!result.is_empty());
    result
        .iter()
        .for_each(|r| assert!(r.channels[0].tags.status.eq(&status)));
}
