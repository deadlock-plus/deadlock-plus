# dp-support

Builds the plain-text support report from probes. No I/O and no Tauri types; the app supplies the probes.

## Public API

- `trait Probe { fn collect(&self) -> Result<Fields, String> }`, implemented for any `Fn() -> Result<Fields, String>`. `Fields` is `Vec<(String, String)>`.
- `enum Section { App, Game, Features, Platform, Jobs, Settings }`.
- `build_report(sections: &[(Section, &dyn Probe)], log_tail: &str, crash_marker: Option<&CrashMarker>) -> String`.
- `is_secret_key(key)` and `MAX_REPORT_BYTES` (64 KiB).

A failing or panicking probe renders `unavailable: <error>`. The text is redacted with `dp_crash::redact`.

## Dependencies

- `dp-crash`
