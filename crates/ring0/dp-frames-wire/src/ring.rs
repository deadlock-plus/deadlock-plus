use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

struct Slot {
    /// Zero means the slot is empty or reserved but not yet published.
    value: AtomicU64,
    tag: AtomicU32,
}

/// Bounded queue: any number of producers, one consumer. Neither side blocks, allocates or takes a
/// lock. A full queue drops the new entry.
pub struct Ring {
    slots: Box<[Slot]>,
    head: AtomicU64,
    tail: AtomicU64,
    dropped: AtomicU64,
}

impl Ring {
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity.max(1)).map(|_| Slot { value: AtomicU64::new(0), tag: AtomicU32::new(0) }).collect();
        Self { slots, head: AtomicU64::new(0), tail: AtomicU64::new(0), dropped: AtomicU64::new(0) }
    }

    /// `value` must not be zero. Returns false when the entry was dropped.
    pub fn push(&self, value: u64, tag: u32) -> bool {
        debug_assert!(value != 0);
        let capacity = self.slots.len() as u64;
        loop {
            let head = self.head.load(Ordering::Relaxed);
            let tail = self.tail.load(Ordering::Acquire);
            if head.wrapping_sub(tail) >= capacity {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                return false;
            }
            if self.head.compare_exchange_weak(head, head + 1, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                let slot = &self.slots[(head % capacity) as usize];
                slot.tag.store(tag, Ordering::Relaxed);
                slot.value.store(value, Ordering::Release);
                return true;
            }
        }
    }

    /// Single consumer only.
    pub fn pop(&self) -> Option<(u64, u32)> {
        let tail = self.tail.load(Ordering::Relaxed);
        let slot = &self.slots[(tail % self.slots.len() as u64) as usize];
        let value = slot.value.load(Ordering::Acquire);
        if value == 0 {
            return None;
        }
        let tag = slot.tag.load(Ordering::Relaxed);
        slot.value.store(0, Ordering::Relaxed);
        self.tail.store(tail + 1, Ordering::Release);
        Some((value, tag))
    }

    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn pops_in_push_order() {
        let ring = Ring::new(4);
        for i in 1..=3 {
            assert!(ring.push(i, i as u32 * 10));
        }
        assert_eq!(ring.pop(), Some((1, 10)));
        assert_eq!(ring.pop(), Some((2, 20)));
        assert_eq!(ring.pop(), Some((3, 30)));
        assert_eq!(ring.pop(), None);
    }

    #[test]
    fn a_full_ring_drops_new_entries_and_counts_them() {
        let ring = Ring::new(2);
        assert!(ring.push(1, 0));
        assert!(ring.push(2, 0));
        assert!(!ring.push(3, 0));
        assert_eq!(ring.dropped(), 1);
        assert_eq!(ring.pop().map(|p| p.0), Some(1));
        assert!(ring.push(4, 0));
        assert_eq!(ring.pop().map(|p| p.0), Some(2));
        assert_eq!(ring.pop().map(|p| p.0), Some(4));
    }

    #[test]
    fn wraps_around_many_times() {
        let ring = Ring::new(3);
        for i in 1..=100u64 {
            assert!(ring.push(i, 0));
            assert_eq!(ring.pop().map(|p| p.0), Some(i));
        }
    }

    #[test]
    fn concurrent_producers_lose_nothing_that_was_accepted() {
        let ring = Arc::new(Ring::new(64));
        let producers: Vec<_> = (0..4u64)
            .map(|p| {
                let ring = ring.clone();
                std::thread::spawn(move || {
                    let mut accepted = 0u64;
                    for i in 0..5000u64 {
                        if ring.push(p * 1_000_000 + i + 1, p as u32) {
                            accepted += 1;
                        } else {
                            std::thread::yield_now();
                        }
                    }
                    accepted
                })
            })
            .collect();
        let mut seen = 0u64;
        let mut last = [0u64; 4];
        loop {
            let finished = producers.iter().all(|h| h.is_finished());
            while let Some((value, tag)) = ring.pop() {
                assert!(value > last[tag as usize], "per-producer order broken");
                last[tag as usize] = value;
                seen += 1;
            }
            if finished {
                break;
            }
        }
        let accepted: u64 = producers.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(seen, accepted);
    }
}
