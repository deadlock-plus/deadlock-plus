# dp-versioned

Reads and writes JSON files wrapped in a schema version envelope, with migrations between versions. It is ring 0 because it is generic file logic that needs only `dp-atomic` from the same ring.

## Public API

- `write(path, migrations, value)` wraps `value` as `{ "schema_version": N, "data": ... }` and writes it atomically. It creates the parent folder.
- `read::<T>(path, migrations)` returns `Ok(None)` for a missing file, or the migrated value.
- `current_version(migrations)` returns `1 + migrations.len()`.
- `Migration` is `fn(serde_json::Value) -> serde_json::Value`. `migrations[i]` turns version `i + 1` into `i + 2`.
- `ReadError` has the variants `Io`, `Corrupt(String)` and `Newer { found, supported }`.

## Dependencies

- `dp-atomic`, a same-ring edge listed in the allowlist in `scripts/tiers.mjs`
- `serde`, `serde_json`

No platform-specific code.

## Gotchas

- A file without both `schema_version` and `data` keys counts as schema 0. It reads as version 1 and then gets every migration.
- A file written by a newer build fails `read` with `ReadError::Newer`. `write` also refuses to replace it and returns a `PermissionDenied` error, so a downgrade cannot destroy newer data.
- `write` replaces an existing file that is not valid JSON.
- The slice index is the version number, so inserting or reordering migrations changes what every stored version means. Only append.

## Testing

```
cargo test -p dp-versioned
```

The tests write under the system temp folder.
