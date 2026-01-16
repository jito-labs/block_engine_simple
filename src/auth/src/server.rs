use jito_protos::auth::{
    auth_service_server::AuthService, GenerateAuthChallengeRequest, GenerateAuthChallengeResponse,
    GenerateAuthTokensRequest, GenerateAuthTokensResponse, RefreshAccessTokenRequest,
    RefreshAccessTokenResponse, Token as PbToken,
};
use log::*;
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::{Request, Response, Status};
pub struct AuthServiceImpl {}

impl AuthServiceImpl {
    pub fn new() -> Self {
        AuthServiceImpl {}
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn generate_auth_challenge(
        &self,
        req: Request<GenerateAuthChallengeRequest>,
    ) -> Result<Response<GenerateAuthChallengeResponse>, Status> {
        info!("AuthService::generate_auth_challenge called from {:?}", req.remote_addr());
        Ok(Response::new(GenerateAuthChallengeResponse {
            challenge: "generate_auth_challenge".into(),
        }))
    }

    async fn generate_auth_tokens(
        &self,
        req: Request<GenerateAuthTokensRequest>,
    ) -> Result<Response<GenerateAuthTokensResponse>, Status> {
        info!("AuthService::generate_auth_tokens called from {:?}", req.remote_addr());

        let expiration_time = SystemTime::now()
            .add(Duration::from_secs(24 * 60 * 60))
            .duration_since(UNIX_EPOCH)
            .expect("expiration time calc");

        Ok(Response::new(GenerateAuthTokensResponse {
            access_token: Some(PbToken {
                value: "access_token".into(),
                expires_at_utc: Some(prost_types::Timestamp {
                    seconds: expiration_time.as_secs() as i64,
                    nanos: 0,
                }),
            }),
            refresh_token: Some(PbToken {
                value: "refresh_token".into(),
                expires_at_utc: Some(prost_types::Timestamp {
                    seconds: expiration_time.as_secs() as i64,
                    nanos: 0,
                }),
            }),
        }))
    }

    async fn refresh_access_token(
        &self,
        req: Request<RefreshAccessTokenRequest>,
    ) -> Result<Response<RefreshAccessTokenResponse>, Status> {
        info!("AuthService::refresh_access_token called from {:?}", req.remote_addr());

        let expiration_time = SystemTime::now()
            .add(Duration::from_secs(30 * 60))
            .duration_since(UNIX_EPOCH)
            .expect("expiration time calc");
        Ok(Response::new(RefreshAccessTokenResponse {
            access_token: Some(PbToken {
                value: "access_token".into(),
                expires_at_utc: Some(prost_types::Timestamp {
                    seconds: expiration_time.as_secs() as i64,
                    nanos: 0,
                }),
            }),
        }))
    }
}
