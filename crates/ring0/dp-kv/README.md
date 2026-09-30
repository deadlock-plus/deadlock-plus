# dp-kv

A small key-value store backed by one JSON file per named store. It is ring 0 because it is generic persistence and needs only `dp-versioned` from the same ring.

## Public API

- `KvStore` holds the loaded stores in memory. Build one with `KvStore::default()`.
- `KvStore::get(dir, store, key)` returns `Result<Option<Value>, String>`.
- `KvStore::set(dir, store, key, value)` stores a value and writes the file.
- `KvStore::delete(dir, store, key)` removes a key and writes the file.

`dir` is the folder that holds the files. `store` maps to `<dir>/<store>.json`.

## Dependencies

- `dp-versioned`, a same-ring edge listed in the allowlist in `scripts/tiers.mjs`
- `log`, `serde_json`

No platform-specific code.

## Gotchas

- Only names in the `STORES` constant are valid. Any other name returns `unknown store`, so the web view cannot pick a path. Add a new store there.
- A failed write leaves both the file and the in-memory copy unchanged. `change` writes the file first and updates memory second.
- A `writing` mutex serialises writers, so the fsync runs without holding the lock that reads use.
- A file from a newer build is not loaded. Reads see an empty store, writes fail, and the file is not touched.
- A corrupt file loads as an empty store and the next write replaces it.
- A flat JSON file without an envelope reads as schema 0 and keeps its keys.
- The internal locks recover from poisoning with `into_inner`, the same way `dp-sync` does. The crate does not depend on `dp-sync`.

## Testing

```
cargo test -p dp-kv
```

The tests write under the system temp folder.
