use super::{BackgroundTask, BackgroundTaskStatus, LaunchInput};
use super::spawner::QueueItem;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct TaskStateManager {
    tasks: Arc<Mutex<HashMap<String, BackgroundTask>>>,
    notifications: Arc<Mutex<HashMap<String, Vec<BackgroundTask>>>>,
    pending_by_parent: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    queues_by_key: Arc<Mutex<HashMap<String, VecDeque<QueueItem>>>>,
    processing_keys: Arc<Mutex<HashSet<String>>>,
    completion_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl TaskStateManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            notifications: Arc::new(Mutex::new(HashMap::new())),
            pending_by_parent: Arc::new(Mutex::new(HashMap::new())),
            queues_by_key: Arc::new(Mutex::new(HashMap::new())),
            processing_keys: Arc::new(Mutex::new(HashSet::new())),
            completion_timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_parts(
        tasks: Arc<Mutex<HashMap<String, BackgroundTask>>>,
        notifications: Arc<Mutex<HashMap<String, Vec<BackgroundTask>>>>,
        pending_by_parent: Arc<Mutex<HashMap<String, HashSet<String>>>>,
        queues_by_key: Arc<Mutex<HashMap<String, VecDeque<QueueItem>>>>,
        processing_keys: Arc<Mutex<HashSet<String>>>,
        completion_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    ) -> Self {
        Self {
            tasks,
            notifications,
            pending_by_parent,
            queues_by_key,
            processing_keys,
            completion_timers,
        }
    }

    pub fn tasks(&self) -> Arc<Mutex<HashMap<String, BackgroundTask>>> {
        self.tasks.clone()
    }

    pub fn tasks_map(&self) -> Arc<Mutex<HashMap<String, BackgroundTask>>> {
        self.tasks.clone()
    }

    pub fn notifications(&self) -> Arc<Mutex<HashMap<String, Vec<BackgroundTask>>>> {
        self.notifications.clone()
    }

    pub fn pending_by_parent(&self) -> Arc<Mutex<HashMap<String, HashSet<String>>>> {
        self.pending_by_parent.clone()
    }

    pub fn queues_by_key(&self) -> Arc<Mutex<HashMap<String, VecDeque<QueueItem>>>> {
        self.queues_by_key.clone()
    }

    pub fn processing_keys(&self) -> Arc<Mutex<HashSet<String>>> {
        self.processing_keys.clone()
    }

    pub fn completion_timers(&self) -> Arc<Mutex<HashMap<String, JoinHandle<()>>>> {
        self.completion_timers.clone()
    }

    pub fn get_task(&self, id: &str) -> Option<BackgroundTask> {
        self.tasks.lock().ok()?.get(id).cloned()
    }

    pub fn find_by_session(&self, session_id: &str) -> Option<BackgroundTask> {
        let tasks = self.tasks.lock().ok()?;
        tasks
            .values()
            .find(|t| t.session_id.as_deref() == Some(session_id))
            .cloned()
    }

    pub fn get_tasks_by_parent_session(&self, session_id: &str) -> Vec<BackgroundTask> {
        let tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        tasks
            .values()
            .filter(|t| t.parent_session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn tasks_for_parent(&self, session_id: &str) -> Vec<BackgroundTask> {
        self.get_tasks_by_parent_session(session_id)
    }

    pub fn get_all_descendant_tasks(&self, session_id: &str) -> Vec<BackgroundTask> {
        let mut result = Vec::new();
        let direct = self.get_tasks_by_parent_session(session_id);
        for child in direct {
            result.push(child.clone());
            if let Some(session_id) = &child.session_id {
                result.extend(self.get_all_descendant_tasks(session_id));
            }
        }
        result
    }

    pub fn get_running_tasks(&self) -> Vec<BackgroundTask> {
        let tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        tasks
            .values()
            .filter(|t| t.status == BackgroundTaskStatus::Running)
            .cloned()
            .collect()
    }

    pub fn get_completed_tasks(&self) -> Vec<BackgroundTask> {
        let tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        tasks
            .values()
            .filter(|t| t.status != BackgroundTaskStatus::Running)
            .cloned()
            .collect()
    }

    pub fn has_running_tasks(&self) -> bool {
        let tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return false,
        };
        tasks.values().any(|t| t.status == BackgroundTaskStatus::Running)
    }

    pub fn get_concurrency_key_from_input(&self, input: &LaunchInput) -> String {
        if let Some(model) = &input.model {
            format!("{}/{}", model.provider_id, model.model_id)
        } else {
            input.agent.clone()
        }
    }

    pub fn get_concurrency_key_from_task(&self, task: &BackgroundTask) -> String {
        if let Some(model) = &task.model {
            format!("{}/{}", model.provider_id, model.model_id)
        } else {
            task.agent.clone()
        }
    }

    pub fn add_task(&self, task: BackgroundTask) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id.clone(), task);
    }

    pub fn remove_task(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.remove(task_id);
    }

    pub fn track_pending_task(&self, parent_session_id: &str, task_id: &str) {
        let mut pending = self.pending_by_parent.lock().unwrap();
        pending
            .entry(parent_session_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(task_id.to_string());
    }

    pub fn cleanup_pending_by_parent(&self, task: &BackgroundTask) {
        if let Some(parent_id) = &task.parent_session_id {
            let mut pending = self.pending_by_parent.lock().unwrap();
            if let Some(set) = pending.get_mut(parent_id) {
                set.remove(&task.id);
                if set.is_empty() {
                    pending.remove(parent_id);
                }
            }
        }
    }

    pub fn update_pending(&self, parent_id: &str, task_id: &str) {
        let mut pending = self.pending_by_parent.lock().unwrap();
        if let Some(set) = pending.get_mut(parent_id) {
            set.remove(task_id);
            if set.is_empty() {
                pending.remove(parent_id);
            }
        }
    }

    pub fn pending_set_size(&self, parent_id: &str) -> Option<usize> {
        let pending = self.pending_by_parent.lock().ok()?;
        pending.get(parent_id).map(|s| s.len())
    }

    pub fn mark_for_notification(&self, task: &BackgroundTask) {
        if let Some(parent_id) = &task.parent_session_id {
            let mut notifications = self.notifications.lock().unwrap();
            let queue = notifications.entry(parent_id.clone()).or_insert_with(Vec::new);
            queue.push(task.clone());
        }
    }

    pub fn get_pending_notifications(&self, session_id: &str) -> Vec<BackgroundTask> {
        let notifications = match self.notifications.lock() {
            Ok(n) => n,
            Err(_) => return Vec::new(),
        };
        notifications.get(session_id).cloned().unwrap_or_default()
    }

    pub fn clear_notifications(&self, session_id: &str) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.remove(session_id);
    }

    pub fn clear_notifications_for_task(&self, task_id: &str) {
        let mut notifications = self.notifications.lock().unwrap();
        for tasks in notifications.values_mut() {
            tasks.retain(|t| t.id != task_id);
        }
        notifications.retain(|_, tasks| !tasks.is_empty());
    }

    pub fn add_to_queue(&self, key: &str, item: QueueItem) {
        let mut queues = self.queues_by_key.lock().unwrap();
        let queue = queues.entry(key.to_string()).or_insert_with(VecDeque::new);
        queue.push_back(item);
    }

    pub fn get_queue(&self, key: &str) -> Option<VecDeque<QueueItem>> {
        let queues = self.queues_by_key.lock().ok()?;
        queues.get(key).cloned()
    }

    pub fn remove_from_queue(&self, key: &str, task_id: &str) -> bool {
        let mut queues = self.queues_by_key.lock().unwrap();
        let queue = match queues.get_mut(key) {
            Some(q) => q,
            None => return false,
        };
        if let Some(index) = queue.iter().position(|item| item.task.id == task_id) {
            queue.remove(index);
            if queue.is_empty() {
                queues.remove(key);
            }
            return true;
        }
        false
    }

    pub fn set_completion_timer(&self, task_id: &str, handle: JoinHandle<()>) {
        let mut timers = self.completion_timers.lock().unwrap();
        timers.insert(task_id.to_string(), handle);
    }

    pub fn clear_completion_timer(&self, task_id: &str) {
        let mut timers = self.completion_timers.lock().unwrap();
        if let Some(handle) = timers.remove(task_id) {
            handle.abort();
        }
    }

    pub fn clear_all_completion_timers(&self) {
        let mut timers = self.completion_timers.lock().unwrap();
        for (_, handle) in timers.drain() {
            handle.abort();
        }
    }

    pub fn clear(&self) {
        self.clear_all_completion_timers();
        self.tasks.lock().unwrap().clear();
        self.notifications.lock().unwrap().clear();
        self.pending_by_parent.lock().unwrap().clear();
        self.queues_by_key.lock().unwrap().clear();
        self.processing_keys.lock().unwrap().clear();
    }

    pub fn cancel_pending_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        let task = match tasks.get_mut(task_id) {
            Some(t) => t,
            None => return false,
        };
        if task.status != BackgroundTaskStatus::Pending {
            return false;
        }

        let key = self.get_concurrency_key_from_task(task);
        self.remove_from_queue(&key, task_id);

        task.status = BackgroundTaskStatus::Cancelled;
        task.completed_at = Some(SystemTime::now());

        self.cleanup_pending_by_parent(task);
        true
    }
}
