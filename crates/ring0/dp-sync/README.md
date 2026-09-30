# dp-sync

Lock helpers that keep working after another thread panics while holding the lock. It is ring 0 because it uses only `std` and knows nothing about the app.

## Public API

- `LockExt::lock_or_recover` on `Mutex<T>` returns the guard, even if the mutex is poisoned.
- `RwLockExt::read_or_recover` and `RwLockExt::write_or_recover` do the same for `RwLock<T>`.

Import the traits and call the methods where you would call `.lock().unwrap()`.

## Dependencies

None. The crate has no workspace or external dependencies and no platform-specific code.

## Gotchas

- Recovery calls `PoisonError::into_inner` and ignores the poison flag. This is only safe when the guarded data stays valid halfway through an update. Do not use these methods for data with invariants that a panic can break.
- The flag is never cleared, so `lock()` on the same mutex keeps returning `Err`. If you mix `lock().unwrap()` and `lock_or_recover()` on one mutex, the first will panic.

## Testing

```
cargo test -p dp-sync
```

The tests poison a mutex and an rwlock on purpose. No special setup is needed.
