use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct Node<T> {
    /// Zero once removed.
    key: AtomicUsize,
    next: *mut Node<T>,
    value: T,
}

/// Lock-free map from a non-zero key to a value, for the render thread to read without waiting.
/// Nodes are prepended and never freed, so a reference stays valid for the life of the process. A
/// removed key stays in the list as a dead node, which costs a few dozen bytes per Vulkan
/// instance or device ever created.
pub struct Registry<T> {
    head: AtomicPtr<Node<T>>,
}

// SAFETY: nodes are immutable after publication except for the atomic key, and `T: Sync` lets
// shared references cross threads.
unsafe impl<T: Send + Sync> Sync for Registry<T> {}
unsafe impl<T: Send + Sync> Send for Registry<T> {}

impl<T> Registry<T> {
    pub const fn new() -> Self {
        Self { head: AtomicPtr::new(ptr::null_mut()) }
    }

    pub fn insert(&self, key: usize, value: T) {
        debug_assert!(key != 0);
        let node = Box::into_raw(Box::new(Node { key: AtomicUsize::new(key), next: ptr::null_mut(), value }));
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            // SAFETY: `node` is ours until the exchange below publishes it.
            unsafe { (*node).next = head };
            match self.head.compare_exchange_weak(head, node, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return,
                Err(current) => head = current,
            }
        }
    }

    /// The newest live entry for `key`.
    pub fn get(&self, key: usize) -> Option<&T> {
        if key == 0 {
            return None;
        }
        let mut node = self.head.load(Ordering::Acquire);
        while !node.is_null() {
            // SAFETY: published nodes are never freed or moved.
            let n = unsafe { &*node };
            if n.key.load(Ordering::Acquire) == key {
                return Some(&n.value);
            }
            node = n.next;
        }
        None
    }

    pub fn remove(&self, key: usize) -> bool {
        if key == 0 {
            return false;
        }
        let mut node = self.head.load(Ordering::Acquire);
        while !node.is_null() {
            // SAFETY: as in `get`.
            let n = unsafe { &*node };
            if n.key.compare_exchange(key, 0, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                return true;
            }
            node = n.next;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn finds_what_was_inserted() {
        let r = Registry::new();
        r.insert(1, "a");
        r.insert(2, "b");
        assert_eq!(r.get(1), Some(&"a"));
        assert_eq!(r.get(2), Some(&"b"));
        assert_eq!(r.get(3), None);
    }

    #[test]
    fn removed_keys_disappear_and_can_be_reused() {
        let r = Registry::new();
        r.insert(1, "old");
        assert!(r.remove(1));
        assert_eq!(r.get(1), None);
        assert!(!r.remove(1));
        r.insert(1, "new");
        assert_eq!(r.get(1), Some(&"new"));
    }

    #[test]
    fn key_zero_never_matches_a_removed_entry() {
        let r = Registry::new();
        r.insert(5, "x");
        r.remove(5);
        assert_eq!(r.get(0), None);
        assert!(!r.remove(0));
    }

    #[test]
    fn concurrent_inserts_are_all_kept() {
        let r = Arc::new(Registry::new());
        let handles: Vec<_> = (0..8usize)
            .map(|t| {
                let r = r.clone();
                std::thread::spawn(move || {
                    for i in 0..200 {
                        r.insert(t * 1000 + i + 1, t);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        for t in 0..8usize {
            for i in 0..200 {
                assert_eq!(r.get(t * 1000 + i + 1), Some(&t));
            }
        }
    }
}
