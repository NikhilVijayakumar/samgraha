mod fixtures;

use registry::RegistryStore;
use schemas::search::{RetrievalLevel, SearchQuery};
use services::search::SearchService;

#[test]
fn test_registry_open_and_migrate() {
    let store = RegistryStore::open_in_memory().unwrap();
    let meta = store.check_integrity().unwrap();
    assert_eq!(meta.document_count, 0);
}

#[test]
fn test_registry_insert_and_query() {
    let store = RegistryStore::open_in_memory().unwrap();
    let doc = fixtures::sample_document(1, "architecture", "Test", "# Test\n\nContent");
    store.insert_document(&doc).unwrap();
    assert_eq!(store.document_count().unwrap(), 1);
    let retrieved = store.get_document(1).unwrap().unwrap();
    assert_eq!(retrieved.title, "Test");
}

#[test]
fn test_registry_search() {
    let store = RegistryStore::open_in_memory().unwrap();
    let docs = fixtures::sample_documents();
    for doc in &docs {
        store.insert_document(doc).unwrap();
    }

    let query = SearchQuery {
        query: "compilation".to_string(),
        level: RetrievalLevel::Metadata,
        max_results: 10,
        ..Default::default()
    };

    let docs = store.get_all_documents().unwrap();
    let results = SearchService::search(&docs, &query).unwrap();
    assert!(!results.results.is_empty());
    assert!(results.results[0].title.contains("Knowledge"));
}
