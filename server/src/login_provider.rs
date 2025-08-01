use crate::{
    Client,
    settings::{AdminRecord, Ban, Banlist, WhitelistRecord, banlist::NormalizedIpAddr},
};
use authc::{AuthClient, AuthClientError, AuthToken, Uuid};
use chrono::Utc;
use common::comp::AdminRole;
use common_net::msg::RegisterError;
use hashbrown::HashMap;
use specs::Component;
use std::{str::FromStr, sync::Arc};
use tokio::{runtime::Runtime, sync::oneshot};
use tracing::{error, info};

/// Determines whether a user is banned, given a ban record connected to a user,
/// the `AdminRecord` of that user (if it exists), and the current time.
pub fn ban_applies(
    ban: &Ban,
    admin: Option<&AdminRecord>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    // Make sure the ban is active, and that we can't override it.
    //
    // If we are an admin and our role is at least as high as the role of the
    // person who banned us, we can override the ban; we negate this to find
    // people who cannot override it.
    let exceeds_ban_role = |admin: &AdminRecord| {
        AdminRole::from(admin.role) >= AdminRole::from(ban.performed_by_role())
    };
    !ban.is_expired(now) && !admin.is_some_and(exceeds_ban_role)
}

fn derive_uuid(username: &str) -> Uuid {
    let mut state = 144066263297769815596495629667062367629;

    for byte in username.as_bytes() {
        state ^= *byte as u128;
        state = state.wrapping_mul(309485009821345068724781371);
    }

    Uuid::from_u128(state)
}

/// derive Uuid for "singleplayer" is a pub fn
pub fn derive_singleplayer_uuid() -> Uuid { derive_uuid("singleplayer") }

pub struct PendingLogin {
    pending_r: oneshot::Receiver<Result<(String, Uuid), RegisterError>>,
}

impl PendingLogin {
    pub(crate) fn new_success(username: String, uuid: Uuid) -> Self {
        let (pending_s, pending_r) = oneshot::channel();
        let _ = pending_s.send(Ok((username, uuid)));

        Self { pending_r }
    }
}

impl Component for PendingLogin {
    type Storage = specs::DenseVecStorage<Self>;
}

pub struct LoginProvider {
    runtime: Arc<Runtime>,
    auth_server: Option<Arc<AuthClient>>,
}

impl LoginProvider {
    pub fn new(auth_addr: Option<String>, runtime: Arc<Runtime>) -> Self {
        tracing::trace!(?auth_addr, "Starting LoginProvider");

        let auth_server = auth_addr.map(|addr| {
            let (scheme, authority) = addr.split_once("://").expect("invalid auth url");

            let scheme = scheme
                .parse::<authc::Scheme>()
                .expect("invalid auth url scheme");
            let authority = authority
                .parse::<authc::Authority>()
                .expect("invalid auth url authority");

            Arc::new(AuthClient::new(scheme, authority).expect("insecure auth scheme"))
        });

        Self {
            runtime,
            auth_server,
        }
    }

    pub fn verify(&self, username_or_token: &str) -> PendingLogin {
        let (pending_s, pending_r) = oneshot::channel();

        match &self.auth_server {
            // Token from auth server expected
            Some(srv) => {
                let srv = Arc::clone(srv);
                let username_or_token = username_or_token.to_string();
                self.runtime.spawn(async move {
                    let _ = pending_s.send(Self::query(srv, &username_or_token).await);
                });
            },
            // Username is expected
            None => {
                let username = username_or_token;
                let uuid = derive_uuid(username);
                let _ = pending_s.send(Ok((username.to_string(), uuid)));
            },
        }

        PendingLogin { pending_r }
    }

    pub(crate) fn login<R>(
        pending: &mut PendingLogin,
        client: &Client,
        admins: &HashMap<Uuid, AdminRecord>,
        whitelist: &HashMap<Uuid, WhitelistRecord>,
        banlist: &Banlist,
        player_count_exceeded: impl FnOnce(String, Uuid) -> (bool, R),
        make_ip_ban_upgrade: impl FnOnce(NormalizedIpAddr, Uuid, String),
    ) -> Option<Result<R, RegisterError>> {
        match pending.pending_r.try_recv() {
            Ok(Err(e)) => Some(Err(e)),
            Ok(Ok((username, uuid))) => {
                let now = Utc::now();
                // We ignore mpsc connections since those aren't to an external
                // process.
                let ip = client
                    .connected_from_addr()
                    .socket_addr()
                    .map(|s| s.ip())
                    .map(NormalizedIpAddr::from);
                // Hardcoded admins can always log in.
                let admin = admins.get(&uuid);
                if let Some(ban) = banlist
                    .uuid_bans()
                    .get(&uuid)
                    .and_then(|ban_entry| ban_entry.current.action.ban())
                    .into_iter()
                    .chain(ip.and_then(|ip| {
                        banlist
                            .ip_bans()
                            .get(&ip)
                            .and_then(|ban_entry| ban_entry.current.action.ban())
                    }))
                    .find(|ban| ban_applies(ban, admin, now))
                {
                    if let Some(ip) = ip
                        && ban.upgrade_to_ip
                    {
                        make_ip_ban_upgrade(ip, uuid, username.clone());
                    }

                    // Get ban info and send a copy of it
                    return Some(Err(RegisterError::Banned(ban.info())));
                }

                // non-admins can only join if the whitelist is empty (everyone can join)
                // or their name is in the whitelist.
                if admin.is_none() && !whitelist.is_empty() && !whitelist.contains_key(&uuid) {
                    return Some(Err(RegisterError::NotOnWhitelist));
                }

                // non-admins can only join if the player count has not been exceeded.
                let (player_count_exceeded, res) = player_count_exceeded(username, uuid);
                if admin.is_none() && player_count_exceeded {
                    return Some(Err(RegisterError::TooManyPlayers));
                }

                Some(Ok(res))
            },
            Err(oneshot::error::TryRecvError::Closed) => {
                error!("channel got closed to early, this shouldn't happen");
                Some(Err(RegisterError::AuthError(
                    "Internal Error verifying".to_string(),
                )))
            },
            Err(oneshot::error::TryRecvError::Empty) => None,
        }
    }

    async fn query(
        srv: Arc<AuthClient>,
        username_or_token: &str,
    ) -> Result<(String, Uuid), RegisterError> {
        info!(?username_or_token, "Validating token");
        // Parse token
        let token = AuthToken::from_str(username_or_token)
            .map_err(|e| RegisterError::AuthError(e.to_string()))?;
        // Validate token
        match async {
            let uuid = srv.validate(token).await?;
            let username = srv.uuid_to_username(uuid).await?;
            let r: Result<_, AuthClientError> = Ok((username, uuid));
            r
        }
        .await
        {
            Err(e) => Err(RegisterError::AuthError(e.to_string())),
            Ok((username, uuid)) => Ok((username, uuid)),
        }
    }

    pub fn username_to_uuid(&self, username: &str) -> Result<Uuid, AuthClientError> {
        match &self.auth_server {
            Some(srv) => {
                //TODO: optimize
                self.runtime.block_on(srv.username_to_uuid(&username))
            },
            None => Ok(derive_uuid(username)),
        }
    }

    pub fn uuid_to_username(
        &self,
        uuid: Uuid,
        fallback_alias: &str,
    ) -> Result<String, AuthClientError> {
        match &self.auth_server {
            Some(srv) => {
                //TODO: optimize
                self.runtime.block_on(srv.uuid_to_username(uuid))
            },
            None => Ok(fallback_alias.into()),
        }
    }
}
