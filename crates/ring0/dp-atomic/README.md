# dp-atomic

One function that writes a file so a crash or a full disk never leaves it half written. It is ring 0 because it uses only `std`.

## Public API

- `write_atomic(path, bytes)` writes `bytes` to `<path>.tmp` next to the target, calls `sync_all`, then renames the temp file over `path`. It returns `io::Result<()>`.

## Dependencies

None. No platform-specific code.

## Gotchas

- The parent folder must already exist. `write_atomic` does not create it, and a missing folder returns an error.
- The temp name is the target name plus `.tmp`. Two concurrent writers to the same path share that temp file, so callers must serialise writes to one path. `dp-kv` does this with a writer mutex.
- On any error the function removes the temp file and leaves the original file untouched.
- The function syncs the file but not the parent directory, so the rename itself may not survive a power loss.

## Testing

```
cargo test -p dp-atomic
```

The tests write under the system temp folder.
