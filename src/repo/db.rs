use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

/// InMemoryDatabase is a simple in-memory key-value store for testing.
#[derive(Default, Debug)]
// Note: Compared to C# which has both objects and structs, Rust has only structs.
//  - To allocate heap space for a struct, use `Box<InMemoryDatabase<K, V>>`.
pub struct InMemoryDatabase<K, V> {
    // Note: Struct-specific fields are defined here.
    /// A thread-safe HashMap to store key-value pairs.
    // Note:
    //  - `Arc`: Atomic reference counting, allowing shared ownership of the map across threads.
    //  - `RwLock`: Provides read-write locks, allowing multiple readers or one writer at a time.
    map: Arc<RwLock<HashMap<K, V>>>, // Note: Fields are private by default
}

// Note: `Send` and `Sync` traits are used to ensure that the database can be used across threads:
//  - `Send`: Allows the type to be transferred between threads.
//  - `Sync`: Allows the type to be referenced from multiple threads.
/// Database trait that defines the interface for accessing a key-value store.
pub trait KVDatabase<K: Eq + Hash + Clone + Send + Sync, V: Clone + Send + Sync> : Send + Sync {
    /// Insert a key-value pair into the database, or update existing key with the new value.
    /// # Arguments
    /// * `key`: The key to insert.
    /// * `value`: The value to insert.
    fn upsert(&mut self, key: &K, value: V);

    /// Read a value by key from the database.
    /// # Arguments
    /// * `key`: The key to read.
    /// # Returns
    /// * `Option<V>`: The value associated with the key, or `None` if the key does not exist.
    fn read(&self, key: &K) -> Option<V>;

    /// Remove a key-value pair from the database.
    /// # Arguments
    /// * `key`: The key to remove.
    fn remove(&self, key: &K);

    /// Update a key-value pair in the database.
    /// # Arguments
    /// * `key`: The key to update.
    /// * `new_value`: The new value to associate with the key.
    fn update(&mut self, key: &K, new_value: V);
}

// Note: Struct-specific methods are defined in the `impl` block. You can extend an external type / struct
//       with additional methods in this way, similar to C# extension methods.
//       Generic bounds are defined in the `impl` block header. Rust emphases zero-cost abstractions
//       and expressiveness, so generic definitions can be long. Trait objects (dyn Trait) is a slightly
//       more costly way to
impl<K: Eq + Hash + Clone + Send + Sync, V: Clone + Send + Sync> KVDatabase<K, V> for InMemoryDatabase<K, V> {
    fn upsert(&mut self, key: &K, value: V) {
        // Note: No need to clone `Arc<T>` explicitly as it implements the `Deref` trait:
        //       https://doc.rust-lang.org/std/sync/struct.Arc.html#deref-behavior
        let mut map = self
            .map
            .write()
            // Note: This is just a hacky way to bypass mutex poisoning for demo purposes.
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        map.insert(key.clone(), value);
    }

    // Note: `Option<V>` is an enum that can be `Some(value)` or `None`. There's no `null` in Rust.
    fn read(&self, key: &K) -> Option<V> {
        
        
        let map = self
            .map
            .read()
            // Note: This is just a hacky way to bypass mutex poisoning for demo purposes.
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        map.get(key).cloned() // Note: Not having ending colon means the function returns this value.
    }

    fn remove(&self, key: &K) {
        let mut map = self
            .map
            .write()
            // Note: This is just a hacky way to bypass mutex poisoning for demo purposes.
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        map.remove(key);
    }

    fn update(&mut self, key: &K, new_value: V) {
        let mut map = self
            .map
            .write()
            // Note: This is just a hacky way to bypass mutex poisoning for demo purposes.
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Update if the key exists.
        // Note: Unstable API `raw_entry` to avoid cloning the key.
        //  https://users.rust-lang.org/t/avoid-unnecessary-key-clone-when-accessing-hashmap-entry/33642
        map.entry(key.clone()).and_modify(|old| {
            *old = new_value;
        });
    }
}

// Note: A struct can have multiple `impl` blocks. Methods not part of a trait can be defined separately.
impl<K, V> InMemoryDatabase<K, V> {
    // Note: Implementing a "default constructor" (`new` is the idiomatic name).
    //       Same as `default()` from the `Default` trait if there's no additional logic.
    /// Creates a new empty instance of `InMemoryDatabase`.
    pub fn new() -> Self {
        InMemoryDatabase {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database() {
        let mut db = InMemoryDatabase::new();

        let key1 = String::from("key1");
        let old_value = String::from("old_value");
        let new_value = String::from("new_value");
        
        db.upsert(&key1, old_value);
        assert_eq!(db.read(&key1), Some("old_value".to_string()));

        db.update(&key1, new_value);
        assert_eq!(db.read(&key1), Some("new_value".to_string()));

        db.remove(&key1);
        assert_eq!(db.read(&key1), None);
    }
}
