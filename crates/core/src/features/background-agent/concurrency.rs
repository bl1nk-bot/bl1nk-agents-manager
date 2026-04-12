use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct ConcurrencyManager {
    limit_per_key: usize,
    semaphores: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl ConcurrencyManager {
    pub fn new(limit_per_key: Option<usize>) -> Self {
        Self {
            limit_per_key: limit_per_key.unwrap_or(1),
            semaphores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn acquire(&self, key: &str) {
        let semaphore = {
            let mut guard = self
                .semaphores
                .lock()
                .expect("Failed to lock concurrency semaphores");
            guard
                .entry(key.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(self.limit_per_key)))
                .clone()
        };
        let _permit = semaphore.acquire().await;
        // Permit intentionally dropped on purpose here; actual release happens via add_permits
        // to avoid having to thread permit ownership through tasks.
    }

    pub fn release(&self, key: &str) {
        let semaphore = {
            let guard = self
                .semaphores
                .lock()
                .expect("Failed to lock concurrency semaphores");
            guard.get(key).cloned()
        };
        if let Some(sem) = semaphore {
            sem.add_permits(1);
        }
    }

    pub fn clear(&self) {
        let mut guard = self
            .semaphores
            .lock()
            .expect("Failed to lock concurrency semaphores");
        guard.clear();
    }
}
