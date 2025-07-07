use sentio_memory::memory_data::MemoryDataRepository;
use sentio_memory::models::MemoryCorpus;
use sentio_memory::repository::MemoryRepository;
use tempfile::tempdir;

#[tokio::test]
async fn test_memory_data_repository_basic() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_memory.json");
    let repo = MemoryDataRepository::new(file_path);
    
    // Initialize the repository
    repo.initialize().await.unwrap();
    
    let user_id = "test_user";
    let corpus = MemoryCorpus::new(user_id.to_string());

    // Test save
    repo.save_memory_corpus(&corpus).await.unwrap();

    // Test get
    let retrieved_corpus = repo.get_memory_corpus(user_id).await.unwrap();
    assert!(retrieved_corpus.is_some());
    assert_eq!(retrieved_corpus.unwrap().user_id, user_id);
    
    // Test health check
    assert!(repo.health_check().await.unwrap());
}

#[tokio::test]
async fn test_memory_data_repository_user_statistics() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_memory_stats.json");
    let repo = MemoryDataRepository::new(file_path);
    
    // Initialize the repository
    repo.initialize().await.unwrap();
    
    let user_id = "test_user_stats";
    let corpus = MemoryCorpus::new(user_id.to_string());
    repo.save_memory_corpus(&corpus).await.unwrap();

    let stats = repo.get_user_statistics(user_id).await.unwrap();
    assert_eq!(stats.user_id, user_id);
}

#[tokio::test]
async fn test_memory_data_repository_delete() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_memory_delete.json");
    let repo = MemoryDataRepository::new(file_path);
    
    // Initialize the repository
    repo.initialize().await.unwrap();
    
    let user_id = "test_user_delete";
    let corpus = MemoryCorpus::new(user_id.to_string());
    repo.save_memory_corpus(&corpus).await.unwrap();

    // Verify data exists
    assert!(repo.get_memory_corpus(user_id).await.unwrap().is_some());

    // Delete data
    repo.delete_user_data(user_id).await.unwrap();

    // Verify data is deleted
    assert!(repo.get_memory_corpus(user_id).await.unwrap().is_none());
}