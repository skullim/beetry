use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anymap3::Map;

type SyncSendMap = Map<dyn Any + Send + Sync>;

pub struct Blackboard {
    storage: RwLock<SyncSendMap>,
}

impl Blackboard {
    fn new() -> Self {
        Self {
            storage: RwLock::new(Map::new()),
        }
    }

    pub fn shared() -> SharedBlackboard {
        Arc::new(Blackboard::new())
    }

    pub fn len(&self) -> usize {
        self.blackboard().len()
    }

    pub fn is_empty(&self) -> bool {
        self.blackboard().is_empty()
    }

    pub fn clear(&self) {
        self.blackboard_mut().clear();
    }

    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.blackboard().contains::<T>()
    }

    pub fn insert<T: Send + Sync + 'static>(&self, value: T) -> Option<T> {
        self.blackboard_mut().insert(value)
    }

    pub fn remove<T: Send + Sync + 'static>(&self) -> Option<T> {
        self.blackboard_mut().remove::<T>()
    }

    pub fn blackboard(&self) -> RwLockReadGuard<'_, SyncSendMap> {
        self.storage.read().unwrap()
    }

    pub fn blackboard_mut(&self) -> RwLockWriteGuard<'_, SyncSendMap> {
        self.storage.write().unwrap()
    }
}

pub type SharedBlackboard = Arc<Blackboard>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_shared_blackboard_creation() {
        let bb = Blackboard::shared();
        assert!(bb.is_empty());
        assert_eq!(bb.len(), 0);
    }

    #[test]
    fn test_insert_and_contains() {
        let bb = Blackboard::shared();

        assert_eq!(bb.insert(42i32), None);
        assert_eq!(bb.insert(String::from("hello")), None);
        assert_eq!(bb.insert(3.1f64), None);

        assert!(bb.contains::<i32>());
        assert!(bb.contains::<String>());
        assert!(bb.contains::<f64>());
        assert!(!bb.contains::<bool>());

        assert_eq!(bb.len(), 3);
        assert!(!bb.is_empty());
    }

    #[test]
    fn test_insert_replace() {
        let bb = Blackboard::shared();
        assert_eq!(bb.insert(42i32), None);
        assert_eq!(bb.insert(100i32), Some(42));
        assert_eq!(bb.blackboard_mut().get::<i32>().unwrap(), &100);
        assert_eq!(bb.len(), 1);
    }

    #[test]
    fn test_remove() {
        let bb = Blackboard::shared();
        assert_eq!(bb.remove::<i32>(), None);

        bb.insert(42i32);
        bb.insert(String::from("test"));

        assert_eq!(bb.remove::<i32>(), Some(42));
        assert_eq!(bb.len(), 1);
        assert!(!bb.contains::<i32>());
        assert!(bb.contains::<String>());
    }

    #[test]
    fn test_clear() {
        let bb = Blackboard::shared();

        bb.insert(42i32);
        bb.insert(String::from("test"));
        bb.insert(true);
        assert_eq!(bb.len(), 3);

        bb.clear();
        assert!(bb.is_empty());
        assert_eq!(bb.len(), 0);
        assert!(!bb.contains::<i32>());
        assert!(!bb.contains::<String>());
        assert!(!bb.contains::<bool>());
    }

    #[test]
    fn test_multi_writes() {
        let shared = Blackboard::shared();
        let mut handles = vec![];
        type TestVec = Vec<usize>;

        shared.insert(TestVec::new());
        let n_threads = 10usize;
        let n_elements = 100usize;
        for _ in 0..n_threads {
            let bb = Arc::clone(&shared);
            let handle = thread::spawn(move || {
                for i in 0..n_elements {
                    let mut guard = bb.blackboard_mut();
                    if let Some(vec) = guard.get_mut::<TestVec>() {
                        vec.push(i);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(shared.contains::<TestVec>());
        let v = shared.remove::<TestVec>().unwrap();
        assert_eq!(v.len(), n_threads * n_elements);
    }

    #[test]
    fn test_complex_types() {
        let bb = Blackboard::shared();

        #[derive(Debug, PartialEq)]
        struct CustomStruct {
            id: u32,
            name: String,
        }

        let custom = CustomStruct {
            id: 1,
            name: "test".to_string(),
        };

        bb.insert(custom);
        assert!(bb.contains::<CustomStruct>());

        let retrieved = bb.remove::<CustomStruct>().unwrap();
        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.name, "test");
    }

    #[test]
    fn test_blackboard_guards() {
        let bb = Blackboard::shared();
        bb.insert(42i32);

        {
            let guard = bb.blackboard();
            assert!(guard.contains::<i32>());
            assert_eq!(guard.len(), 1);
        }

        {
            let mut guard = bb.blackboard_mut();
            let test = String::from("test");
            guard.insert(test.clone());
            assert_eq!(guard.len(), 2);
            assert_eq!(guard.get::<String>().unwrap(), &test);
        }

        assert_eq!(bb.len(), 2);
    }

    #[test]
    fn test_shared_references() {
        let bb1 = Blackboard::shared();
        let bb2 = Arc::clone(&bb1);

        bb1.insert(42i32);

        assert!(bb2.contains::<i32>());
        assert_eq!(bb2.len(), 1);
        assert_eq!(bb2.remove::<i32>(), Some(42));
        assert!(bb1.is_empty());
    }
}
