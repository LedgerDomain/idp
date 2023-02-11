use idp_core::Datahost;
use idp_proto::{
    indoor_data_plumbing_server::{IndoorDataPlumbing, IndoorDataPlumbingServer},
    PushRequest, PushResponse,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

// Cloning IDPServer simply clones the Arc<RwLock<Datahost>>.  This struct should not contain anything else.
#[derive(Clone)]
pub struct IDPServer {
    datahost_la: Arc<RwLock<Datahost>>,
}

impl IDPServer {
    pub fn new(datahost_la: Arc<RwLock<Datahost>>) -> Self {
        IDPServer { datahost_la }
    }
    pub async fn listen_on(
        &self,
        addr: std::net::SocketAddr,
    ) -> Result<(), tonic::transport::Error> {
        tonic::transport::Server::builder()
            .add_service(IndoorDataPlumbingServer::new(self.clone()))
            // .add_service(IndoorDataPlumbingServer::with_interceptor(self.clone(), {
            //     let srv = self.clone();
            //     move |req: tonic::Request<()>| -> Result<tonic::Request<()>, tonic::Status> {
            //         srv.verify_authentication(req)
            //     }
            // }))
            .serve(addr)
            .await
    }
    pub fn verify_authentication(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // TEMP HACK
        let token = tonic::metadata::MetadataValue::from_static("Bearer some-secret-token");

        match req.metadata().get("authorization") {
            Some(t) if token == t => Ok(req),
            _ => Err(tonic::Status::unauthenticated("No valid auth token")),
        }
    }
    fn handle_push_request(
        &self,
        push_request: PushRequest,
    ) -> Result<PushResponse, tonic::Status> {
        log::trace!(
            "IDPServer::handle_push_request; push_request: {:?}",
            push_request
        );
        match push_request.value.ok_or_else(|| {
            tonic::Status::invalid_argument(
                "malformed PushRequest; encountered 'None' in request stream",
            )
        })? {
            idp_proto::push_request::Value::ShouldISendThisPlum(plum_head_seal) => {
                let value = if self
                    .datahost_la
                    .read()
                    .select_option_plum_head_row(&plum_head_seal)
                    .map_err(|e| tonic::Status::internal(e.to_string()))?
                    .is_some()
                {
                    // If the Datahost already has this Plum, the client shouldn't send it.
                    idp_proto::push_response::Value::DontSendThisPlum(plum_head_seal)
                } else {
                    // If the Datahost doesn't have this Plum, the client should send it.
                    idp_proto::push_response::Value::SendThisPlum(plum_head_seal)
                };
                Ok(PushResponse { value: Some(value) })
            }
            idp_proto::push_request::Value::HereHaveAPlum(plum) => {
                self.datahost_la
                    .read()
                    .store_plum(&plum)
                    .map_err(|e| tonic::Status::internal(e.to_string()))?;
                Ok(PushResponse {
                    value: Some(idp_proto::push_response::Value::Ok(
                        idp_proto::Acknowledgement {},
                    )),
                })
            }
        }
    }
}

// type IndoorDataPlumbingResult<T> = Result<tonic::Response<T>, tonic::Status>;

#[tonic::async_trait]
impl IndoorDataPlumbing for IDPServer {
    type PushStream =
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<PushResponse, tonic::Status>> + Send>>;
    async fn push(
        &self,
        request: tonic::Request<tonic::Streaming<PushRequest>>,
    ) -> Result<tonic::Response<Self::PushStream>, tonic::Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // this spawn here is required if you want to handle connection error.
        // If we just map `in_stream` and write it back as `out_stream` the `out_stream`
        // will be drooped when connection error occurs and error will never be propagated
        // to mapped version of `in_stream`.
        {
            // Make a clone of IDPServer to move into the async closure.  This simply clones the
            // Arc<RwLock<Datahost>> inside.
            let idp_server = self.clone();
            tokio::spawn(async move {
                // use futures::StreamExt;
                while let Some(push_request_r) = in_stream.next().await {
                    match push_request_r {
                        Ok(push_request) => {
                            tx.send(idp_server.handle_push_request(push_request))
                                .await
                                .expect("working rx");
                        }
                        Err(err) => {
                            if let Some(io_err) = match_for_io_error(&err) {
                                if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                                    // here you can handle special case when client
                                    // disconnected in unexpected way
                                    eprintln!("\tclient disconnected: broken pipe");
                                    break;
                                }
                            }

                            match tx.send(Err(err)).await {
                                Ok(_) => (),
                                Err(_err) => break, // response was droped
                            }
                        }
                    }
                }
                println!("\tstream ended");
            });
        }

        // echo just write the same data that was received
        let out_stream = ReceiverStream::new(rx);

        // Ok(tonic::Response::new(Box::pin(out_stream) as std::pin::Pin<Box<dyn futures::Stream<Item = Result<PushResponse, tonic::Status>> + Send>>))
        Ok(tonic::Response::new(
            Box::pin(out_stream) as Self::PushStream
        ))
    }

    // async fn pull(
    //     &self,
    //     _request: tonic::Request<PullRequest>,
    // ) -> Result<tonic::Response<PullResponse>, tonic::Status> {
    //     Err(tonic::Status::unimplemented("sorry!"))
    // }

    // async fn del(
    //     &self,
    //     _request: tonic::Request<DelRequest>,
    // ) -> Result<tonic::Response<DelResponse>, tonic::Status> {
    //     Err(tonic::Status::unimplemented("sorry!"))
    // }
}

/// Reference: https://github.com/hyperium/tonic/blob/82770713b58892203a83c307729b3e7bebe574e3/examples/src/streaming/server.rs
fn match_for_io_error(err_status: &tonic::Status) -> Option<&std::io::Error> {
    let mut err: &(dyn std::error::Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}
