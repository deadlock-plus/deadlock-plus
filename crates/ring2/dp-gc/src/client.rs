use std::future::Future;
use std::time::Duration;

use prost::Message as _;
use steam_vent::proto::MsgKind;
use steam_vent::{Connection, ConnectionTrait, GameCoordinator, RawNetMessage, ServerList, UntypedMessage};
use valveprotos::deadlock::c_msg_client_to_gc_get_match_meta_data_response::EResult;
use valveprotos::deadlock::{
    CMsgClientToGcGetMatchMetaData, CMsgClientToGcGetMatchMetaDataResponse, EgcCitadelClientMessages,
};

use crate::auth::AuthContext;
use crate::error::GcError;

const DEADLOCK_APP_ID: u32 = 1422450;
const GC_CALL_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveredSalts {
    pub match_id: u64,
    pub cluster_id: Option<u32>,
    pub metadata_salt: Option<u32>,
    pub replay_salt: Option<u32>,
    pub account_id: u32,
}

/// A response with neither salt is an error so it is never posted.
pub fn interpret_salts_response(
    match_id: u64,
    account_id: u32,
    resp: &CMsgClientToGcGetMatchMetaDataResponse,
) -> Result<RecoveredSalts, GcError> {
    match resp.result {
        Some(r) if r == EResult::KEResultRateLimited as i32 => return Err(GcError::GcRateLimited),
        Some(r) if r == EResult::KEResultSuccess as i32 => {}
        r => return Err(GcError::GcUnavailable(format!("salts result {r:?}"))),
    }
    if resp.metadata_salt.is_none() && resp.replay_salt.is_none() {
        return Err(GcError::GcUnavailable("response has no salts".into()));
    }
    Ok(RecoveredSalts {
        match_id,
        cluster_id: resp.replay_group_id,
        metadata_salt: resp.metadata_salt,
        replay_salt: resp.replay_salt,
        account_id,
    })
}

async fn with_timeout<T, E: std::fmt::Display>(
    label: &str,
    fut: impl Future<Output = Result<T, E>>,
) -> Result<T, GcError> {
    match tokio::time::timeout(GC_CALL_TIMEOUT, fut).await {
        Ok(result) => result.map_err(|e| GcError::GcUnavailable(format!("{label}: {e}"))),
        Err(_) => Err(GcError::GcUnavailable(format!("{label}: timed out after {GC_CALL_TIMEOUT:?}"))),
    }
}

/// A live GC session for one account. Drop it between passes.
pub struct GcSession {
    gc: GameCoordinator,
    _conn: Connection,
    account_id: u32,
}

impl GcSession {
    /// Logs in through the user's own account and handshakes the Deadlock GC. Fails when the
    /// account does not own the game or Steam is unreachable.
    pub async fn connect(ctx: &AuthContext) -> Result<Self, GcError> {
        let servers = with_timeout("server discovery", ServerList::discover()).await?;
        let conn =
            with_timeout("CM login", Connection::access(&servers, &ctx.account_name, &ctx.refresh_token)).await?;
        let gc = with_timeout("GC handshake", GameCoordinator::new(&conn, DEADLOCK_APP_ID)).await?;
        Ok(Self { gc, _conn: conn, account_id: ctx.account_id() })
    }

    /// One `GetMatchMetaData` round trip with no retries, so a failed fetch costs at most one
    /// quota unit.
    pub async fn fetch_match_salts(&self, match_id: u64) -> Result<RecoveredSalts, GcError> {
        let req = CMsgClientToGcGetMatchMetaData { match_id: Some(match_id), ..Default::default() };
        let kind = MsgKind(EgcCitadelClientMessages::KEMsgClientToGcGetMatchMetaData as i32);
        let raw: RawNetMessage =
            with_timeout("salts", self.gc.job_untyped(UntypedMessage(req.encode_to_vec()), kind, true)).await?;
        let resp = CMsgClientToGcGetMatchMetaDataResponse::decode(raw.data.as_ref())
            .map_err(|e| GcError::GcUnavailable(format!("bad salts response: {e}")))?;
        interpret_salts_response(match_id, self.account_id, &resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(result: EResult, metadata: Option<u32>, replay: Option<u32>) -> CMsgClientToGcGetMatchMetaDataResponse {
        CMsgClientToGcGetMatchMetaDataResponse {
            result: Some(result as i32),
            metadata_salt: metadata,
            replay_salt: replay,
            replay_group_id: Some(404),
            ..Default::default()
        }
    }

    #[test]
    fn success_carries_both_salts_and_the_account() {
        let got = interpret_salts_response(42, 7, &response(EResult::KEResultSuccess, Some(1), Some(2))).unwrap();
        assert_eq!(
            got,
            RecoveredSalts {
                match_id: 42,
                cluster_id: Some(404),
                metadata_salt: Some(1),
                replay_salt: Some(2),
                account_id: 7
            }
        );
    }

    #[test]
    fn one_salt_is_enough() {
        assert!(interpret_salts_response(42, 7, &response(EResult::KEResultSuccess, None, Some(2))).is_ok());
    }

    #[test]
    fn success_without_salts_is_an_error() {
        let e = interpret_salts_response(42, 7, &response(EResult::KEResultSuccess, None, None)).unwrap_err();
        assert!(matches!(e, GcError::GcUnavailable(_)));
    }

    #[test]
    fn rate_limit_is_its_own_error() {
        let e = interpret_salts_response(42, 7, &response(EResult::KEResultRateLimited, Some(1), Some(2))).unwrap_err();
        assert_eq!(e, GcError::GcRateLimited);
    }

    #[test]
    fn missing_result_is_an_error() {
        let resp = CMsgClientToGcGetMatchMetaDataResponse { metadata_salt: Some(1), ..Default::default() };
        assert!(matches!(interpret_salts_response(42, 7, &resp), Err(GcError::GcUnavailable(_))));
    }
}
