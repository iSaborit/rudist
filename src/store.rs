use std::{collections::HashMap};
use std::sync::{Arc, RwLock};

pub type Bytes = Arc<Vec<u8>>;

pub struct Store {
    data: RwLock<HashMap<String, Bytes>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: RwLock::new(HashMap::new()),
        }
    }

    // Generate test
    pub fn set(&self, key : String, value : Vec<u8>) {
        let mut write_lock = self.data.write().expect("Lock poisoned");
        write_lock.insert(key, Arc::new(value));
    }

    // Generate test
    pub fn get(&self, key : &str) -> Option<Bytes> {
        let mut guard = self.data.read().expect("Lock poisoned");
        guard.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let store = Store::new();
        let key = String::from("test_key");
        let value = vec![1, 2, 3, 4, 5];

        store.set(key.clone(), value.clone());

        let result = store.get(&key);
        assert_eq!(result, Some(Arc::new(value)));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let store = Store::new();
        let key = String::from("nonexistent_key");

        let result = store.get(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_set_overwrite() {
        let store = Store::new();
        let key = String::from("overwrite_key");
        let value1 = vec![1, 2, 3];
        let value2 = vec![4, 5, 6];

        store.set(key.clone(), value1.clone());
        store.set(key.clone(), value2.clone());

        let result = store.get(&key);
        assert_eq!(result, Some(Arc::new(value2)));
    }

    #[test]
    fn test_concurrent_set_and_get() {
        let store = Arc::new(Store::new());
        let key = String::from("concurrent_key");
        let value = vec![1, 2, 3];

        let store_clone = store.clone();
        let key_moved = key.clone();
        let value_moved = value.clone();
        let handle = std::thread::spawn(move || {
            store_clone.set(key_moved, value_moved);
        });

        handle.join().unwrap();

        let result = store.get(&key);
        assert_eq!(result, Some(Arc::new(value)));
    }

    #[test]
    fn test_concurrent_set_and_get_multiple_keys() {
        let store = Arc::new(Store::new());
        let keys = vec![
            String::from("key1"),
            String::from("key2"),
            String::from("key3"),
        ];
        let values = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];

        let store_clone = store.clone();
        let keys_moved = keys.clone();
        let value_moved = values.clone();
        let handle1 = std::thread::spawn(move || {
            for (key, value) in keys_moved.iter().zip(value_moved.iter()) {
                store_clone.set(key.clone(), value.clone());
            }
        });

        handle1.join().unwrap();

        for (key, value) in keys.iter().zip(values.iter()) {
            let result = store.get(key);
            assert_eq!(result, Some(Arc::new(value.clone())));
        }
    }
}
