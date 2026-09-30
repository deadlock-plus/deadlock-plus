# dp-export

Checks and writes user-initiated exports. It is ring 0 because it has no platform code and needs only `dp-atomic` from the same ring.

## Public API

- `validate_request(default_name, extension)` checks the file name and extension the web view suggests for a save dialog. It returns `Result<(), String>`.
- `write_export(path, contents)` writes `contents` to an absolute path with `dp_atomic::write_atomic`. It returns `Result<(), String>`.

## Dependencies

- `dp-atomic`, a same-ring edge listed in the allowlist in `scripts/tiers.mjs`
- `log`

## Gotchas

- `validate_request` rejects an empty name, a name over 100 bytes, and a name containing `/`, `\`, `:` or `..`. It accepts only an extension of 1 to 8 ASCII letters or digits.
- `write_export` rejects a relative path and a path that is an existing folder. It does not open the dialog. The caller opens the dialog and passes the path the user picked, so the web view never supplies a path.
- A failed write is logged at warn level and the error text goes back to the caller.

## Testing

```
cargo test -p dp-export
```

The tests write under the system temp folder.
