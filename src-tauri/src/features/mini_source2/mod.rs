//! Deliberately minimal readers/writers for the Source 2 formats Deadlock+ touches: VPK v2 and
//! text KV3. Independent of the app so the module can be lifted into its own crate.

pub mod kv3;
pub mod vpk;
