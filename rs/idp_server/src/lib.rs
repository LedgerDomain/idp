pub mod state;

use crate::state::ServerState;
use std::sync::{Arc, Mutex};
use idp_proto::{
    DelRequest, DelResponse,
    PullRequest, PullResponse,
    PushRequest, PushResponse,
    indoor_data_plumbing_server::{IndoorDataPlumbing, IndoorDataPlumbingServer},
};

// Cloning IdpServer simply clones the Arc to the ServerState Mutex.  This struct should not contain anything else.
#[derive(Clone)]
pub struct IdpServer {
    // Really this would just be an IDP core data instance
    server_state_ma: Arc<Mutex<ServerState>>,
}

impl IdpServer {
    pub fn new(server_state_ma: Arc<Mutex<ServerState>>) -> IdpServer {
        IdpServer { server_state_ma }
    }
    pub async fn listen_on(
        &self,
        addr: std::net::SocketAddr,
    ) -> Result<(), tonic::transport::Error> {
        tonic::transport::Server::builder()
//             .add_service(IndoorDataPlumbingServer::new(self.clone()))
            .add_service(IndoorDataPlumbingServer::with_interceptor(self.clone(), {
                let srv = self.clone();
                move |req: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
                    srv.verify_authentication(req)
                }
            }))
            .serve(addr)
            .await
    }
    pub fn verify_authentication(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let token = tonic::metadata::MetadataValue::from_str("Bearer some-secret-token").unwrap();

        match req.metadata().get("authorization") {
            Some(t) if token == t => Ok(req),
            _ => Err(tonic::Status::unauthenticated("No valid auth token")),
        }
    }
}

// type IndoorDataPlumbingResult<T> = Result<tonic::Response<T>, tonic::Status>;

#[tonic::async_trait]
impl IndoorDataPlumbing for IdpServer {
    async fn push(
        &self,
        _request: tonic::Request<PushRequest>,
    ) -> Result<tonic::Response<PushResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("sorry!"))
    }

    async fn pull(
        &self,
        _request: tonic::Request<PullRequest>,
    ) -> Result<tonic::Response<PullResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("sorry!"))
    }

    async fn del(
        &self,
        _request: tonic::Request<DelRequest>,
    ) -> Result<tonic::Response<DelResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("sorry!"))
    }
}
