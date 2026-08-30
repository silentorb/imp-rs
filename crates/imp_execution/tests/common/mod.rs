use imp_collection_transforms::collection_transforms_library;
use imp_core_types::core_node_library;
use imp_pathing::pathing_library;
use imp_registry::{create_registry, load_library, Registry};

pub fn test_registry() -> Registry {
    load_library(
        load_library(
            load_library(create_registry(), core_node_library()).unwrap(),
            collection_transforms_library(),
        )
        .unwrap(),
        pathing_library(),
    )
    .unwrap()
}
