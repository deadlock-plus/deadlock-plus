# dp-gc

Recovers Deadlock match salts through the Steam Game Coordinator, using the user's own remembered Steam login. It is ring 2 because it encodes Valve's session storage and GC protocol. It does no file or network I/O except the GC connection itself.

## Public API

- `quota::QuotaWindow` is a rolling 24 hour window capped at 40 fetches. `try_consume`, `remaining` and `exhaust` take the clock as an argument.
- `auth::recover_all(steam_dir)` returns an `AuthContext` for every remembered account whose refresh token decrypts. `connect_cache_blob`, `steam_id_from_jwt`, `decrypt_aes_blob` and `local_vdf_path` are its steps.
- `client::GcSession::connect(ctx)` logs in and handshakes the Deadlock GC. `fetch_match_salts(id)` makes one `GetMatchMetaData` call and returns `RecoveredSalts`. `interpret_salts_response` is the pure mapping.
- `picks::fresh_newest_first(ids, processed, take)` chooses which matches to fetch.
- `pass::fetch_loop(fetcher, host, picks, state, processed)` runs the fetches. The caller implements `Host` for pacing, the stop condition and delivery. `AccountState` holds the quota and the backoff deadline and derives `Serialize` so the app can persist it.
- `error::GcError`.

## Dependencies

- `steam-vent` and `valveprotos`, git dependencies pinned to commits in `deadlock-api` repositories
- `tokio`, `prost`, `aes`, `cbc`, `sha2`, `keyvalues-parser`, `crc32fast`, `hex`, `base64`, `serde`, `serde_json`, `log`
- `windows-dpapi` on Windows
- No workspace crates.

## Gotchas

- `AuthContext::refresh_token` is a live account credential. Keep it in memory. Its `Debug` output hides it and no error message may include it.
- Windows decrypts with DPAPI and the account name as entropy, so it only works for the signed-in Windows user. Other systems use AES with a key derived from the account name.
- Steam under Wine, Proton or Whisky stores a DPAPI blob that a native build cannot decrypt. Those accounts are skipped.
- `ConnectCache` is keyed by the CRC32 of the account name. A token whose Steam id differs from the `loginusers.vdf` entry is rejected.
- A rate-limited fetch exhausts the whole window and does not mark the match as processed, so another account can still take it.
- A failed fetch is not retried. It costs no quota.
- The caller must not run a pass while Deadlock is open. Steam routes GC traffic to the game's own pipe.

## Testing

```
cargo test -p dp-gc
```

The tests need no Steam install and make no network calls. The GC connection itself is not covered.
