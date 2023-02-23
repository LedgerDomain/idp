use async_lock::RwLock;
use idp_core::Datahost;
use idp_proto::{
    IndoorDataPlumbing, IndoorDataPlumbingServer, PlumHeadSeal, PullRequest, PullResponse,
    PushRequest, PushResponse,
};
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
    async fn handle_push_request(
        &self,
        push_request: PushRequest,
    ) -> Result<PushResponse, tonic::Status> {
        log::debug!(
            "IDPServer::handle_push_request; push_request: {:?}",
            push_request
        );
        match push_request.value.ok_or_else(|| {
            tonic::Status::invalid_argument(
                "malformed PushRequest; encountered 'None' in request stream",
            )
        })? {
            idp_proto::push_request::Value::ShouldISendThisPlum(plum_head_seal) => {
                log::debug!(
                    "IDPServer::handle_push_request; got ShouldISendThisPlum({})",
                    plum_head_seal
                );
                let value = if self
                    .datahost_la
                    .read()
                    .await
                    .has_plum(&plum_head_seal)
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?
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
                log::debug!(
                    "IDPServer::handle_push_request; got HereHaveAPlum(with plum head seal {})",
                    PlumHeadSeal::from(&plum.plum_head)
                );
                self.datahost_la
                    .read()
                    .await
                    .store_plum(&plum)
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?;
                Ok(PushResponse {
                    value: Some(idp_proto::push_response::Value::Ok(
                        idp_proto::Acknowledgement {},
                    )),
                })
            }
        }
    }
    // TODO: Make this streaming
    async fn handle_pull_request(
        &self,
        pull_request: PullRequest,
    ) -> Result<Vec<PullResponse>, tonic::Status> {
        log::debug!(
            "IDPServer::handle_pull_request; pull_request: {:?}",
            pull_request
        );

        match pull_request.value.ok_or_else(|| {
            tonic::Status::invalid_argument("malformed PullRequest; encountered 'None' in request")
        })? {
            idp_proto::pull_request::Value::IWantThisPlum(plum_head_seal) => {
                // TODO: Stream the results from the DB query instead of reading the whole thing into memory.
                let plum_relation_flags_m = self
                    .datahost_la
                    .read()
                    .await
                    .accumulated_relations_recursive(
                        &plum_head_seal,
                        idp_proto::PlumRelationFlags::CONTENT_DEPENDENCY
                            | idp_proto::PlumRelationFlags::METADATA_DEPENDENCY,
                    )
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?;
                log::trace!(
                    "IDPServer::handle_pull_request({}); plum_relation_flags_m ({} entries):",
                    plum_head_seal,
                    plum_relation_flags_m.len()
                );
                for plum_relation_flags in plum_relation_flags_m.iter() {
                    log::trace!(
                        "    {} -> {:?}",
                        plum_relation_flags.0,
                        plum_relation_flags.1
                    );
                }

                // Accumulated plum_relations for a plum should not include the plum itself.
                assert!(!plum_relation_flags_m.contains_key(&plum_head_seal));
                // NOTE: Because the plum_relations are collected via HashMap, they will be in a random order,
                // which is not ideal for short-circuiting push operations when the target Datahost already
                // has most of the Plum-s.  One solution to this is to accumulate the plum_relations and store
                // them in (probably) a breadth-first traversal order.  A better solution would be to stream
                // the plum_relations accumulations and only bother recursing on Plum-s that the target Datahost
                // asks to receive.
                let mut accumulated_plum_head_seal_v =
                    Vec::with_capacity(plum_relation_flags_m.len() + 1);
                // First, add this Plum.
                accumulated_plum_head_seal_v.push(plum_head_seal.clone());
                accumulated_plum_head_seal_v
                    .extend(plum_relation_flags_m.keys().into_iter().cloned());

                let mut pull_response_v = Vec::with_capacity(accumulated_plum_head_seal_v.len());

                for accumulated_plum_head_seal in accumulated_plum_head_seal_v.into_iter() {
                    if let Some(plum) = self
                        .datahost_la
                        .read()
                        .await
                        .load_option_plum(&accumulated_plum_head_seal)
                        .await
                        .map_err(|e| tonic::Status::internal(e.to_string()))?
                    {
                        pull_response_v.push(idp_proto::PullResponse {
                            value: Some(idp_proto::pull_response::Value::Plum(plum)),
                        });
                    } else {
                        pull_response_v.push(idp_proto::PullResponse {
                            value: Some(idp_proto::pull_response::Value::IDontHaveThisPlum(
                                accumulated_plum_head_seal,
                            )),
                        });
                    }
                }

                Ok(pull_response_v)
            }
        }

        // match pull_request.value.ok_or_else(|| {
        //     tonic::Status::invalid_argument(
        //         "malformed PushRequest; encountered 'None' in request stream",
        //     )
        // })? {
        //     Some(idp_proto::push_request::Value::IWantThisPlumHead(plum_head_seal)) => {
        //         if let Some(plum_head_row) = self
        //             .datahost_la
        //             .read()
        //             .select_option_plum_head_row(&plum_head_seal)
        //             .map_err(|e| tonic::Status::internal(e.to_string()))?
        //         {
        //             Ok(idp_proto::PullResponse {
        //                 value: Some(idp_proto::pull_response::Value::PlumHead(PlumHead::from(
        //                     plum_head_row,
        //                 ))),
        //             })
        //         } else {
        //             Ok(idp_proto::PullResponse {
        //                 value: Some(idp_proto::pull_response::Value::IDontHaveThisPlumHead(
        //                     plum_head_seal,
        //                 )),
        //             })
        //         }
        //     }
        //     Some(idp_proto::push_request::Value::IWantThisPlumRelations(plum_head_seal)) => {
        //         if let Some(plum_head_row) = self
        //             .datahost_la
        //             .read()
        //             .select_option_plum_head_row(&plum_head_seal)
        //             .map_err(|e| tonic::Status::internal(e.to_string()))?
        //         {
        //             Ok(idp_proto::PullResponse {
        //                 value: Some(idp_proto::pull_response::Value::PlumHead(PlumHead::from(
        //                     plum_head_row,
        //                 ))),
        //             })
        //         } else {
        //             Ok(idp_proto::PullResponse {
        //                 value: Some(idp_proto::pull_response::Value::IDontHaveThisPlumHead(
        //                     plum_head_seal,
        //                 )),
        //             })
        //         }
        //     }
        //     Some(idp_proto::push_request::Value::IWantThisPlumHeadAndRelations(plum_head_seal)) => {
        //     }
        //     Some(idp_proto::push_request::Value::IWantThisPlumBody(plum_body_seal)) => {}

        //     idp_proto::push_request::Value::ShouldISendThisPlum(plum_head_seal) => {
        //         let value = if self
        //             .datahost_la
        //             .read()
        //             .select_option_plum_head_row(&plum_head_seal)
        //             .map_err(|e| tonic::Status::internal(e.to_string()))?
        //             .is_some()
        //         {
        //             // If the Datahost already has this Plum, the client shouldn't send it.
        //             idp_proto::push_response::Value::DontSendThisPlum(plum_head_seal)
        //         } else {
        //             // If the Datahost doesn't have this Plum, the client should send it.
        //             idp_proto::push_response::Value::SendThisPlum(plum_head_seal)
        //         };
        //         Ok(PushResponse { value: Some(value) })
        //     }
        //     idp_proto::push_request::Value::HereHaveAPlum(plum) => {
        //         self.datahost_la
        //             .read()
        //             .store_plum(&plum)
        //             .map_err(|e| tonic::Status::internal(e.to_string()))?;
        //         Ok(PushResponse {
        //             value: Some(idp_proto::push_response::Value::Ok(
        //                 idp_proto::Acknowledgement {},
        //             )),
        //         })
        //     }
        // }
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
        // will be dropped when connection error occurs and error will never be propagated
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
                            // tx.send(idp_server.handle_push_request(push_request))
                            //     .await
                            //     .expect("tx error");

                            // tx.send(idp_server.handle_push_request(push_request))
                            //     .await
                            //     .map_err(|_| tonic::Status::internal("tx send error"))?;

                            match tx
                                .send(idp_server.handle_push_request(push_request).await)
                                .await
                            {
                                Ok(()) => {}
                                Err(error_f) => {
                                    // TODO: Send this error through the tx?  Though that may not work
                                    // if this send failed.  Maybe break here instead.
                                    // error_f.0.expect("tx error");
                                    error_f.0.expect("tx error");
                                }
                            }
                            // .expect("working rx");
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
                                Err(_err) => break, // response was dropped
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

    type PullStream =
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<PullResponse, tonic::Status>> + Send>>;
    async fn pull(
        &self,
        request: tonic::Request<PullRequest>,
    ) -> Result<tonic::Response<Self::PullStream>, tonic::Status> {
        let pull_request = request.into_inner();

        // let (tx, rx) = tokio::sync::mpsc::channel(128);

        let pull_response_v = self
            .handle_pull_request(pull_request)
            .await?
            .into_iter()
            .map(|pull_response| Ok(pull_response))
            .collect::<Vec<_>>();

        // let out_stream = ReceiverStream::new(rx);

        // Ok(tonic::Response::new(Box::pin(out_stream) as std::pin::Pin<Box<dyn futures::Stream<Item = Result<PullResponse, tonic::Status>> + Send>>))
        Ok(tonic::Response::new(
            Box::pin(tokio_stream::iter(pull_response_v)) as Self::PullStream,
        ))

        // let mut in_stream = request.into_inner();
        // let (tx, rx) = tokio::sync::mpsc::channel(128);

        // // this spawn here is required if you want to handle connection error.
        // // If we just map `in_stream` and write it back as `out_stream` the `out_stream`
        // // will be dropped when connection error occurs and error will never be propagated
        // // to mapped version of `in_stream`.
        // {
        //     // Make a clone of IDPServer to move into the async closure.  This simply clones the
        //     // Arc<RwLock<Datahost>> inside.
        //     let idp_server = self.clone();
        //     tokio::spawn(async move {
        //         // use futures::StreamExt;
        //         while let Some(pull_request_r) = in_stream.next().await {
        //             match pull_request_r {
        //                 Ok(pull_request) => {
        //                     tx.send(idp_server.handle_pull_request(pull_request))
        //                         .await
        //                         .expect("working rx");
        //                 }
        //                 Err(err) => {
        //                     if let Some(io_err) = match_for_io_error(&err) {
        //                         if io_err.kind() == std::io::ErrorKind::BrokenPipe {
        //                             // here you can handle special case when client
        //                             // disconnected in unexpected way
        //                             eprintln!("\tclient disconnected: broken pipe");
        //                             break;
        //                         }
        //                     }

        //                     match tx.send(Err(err)).await {
        //                         Ok(_) => (),
        //                         Err(_err) => break, // response was dropped
        //                     }
        //                 }
        //             }
        //         }
        //         println!("\tstream ended");
        //     });
        // }

        // let out_stream = ReceiverStream::new(rx);

        // // Ok(tonic::Response::new(Box::pin(out_stream) as std::pin::Pin<Box<dyn futures::Stream<Item = Result<PullResponse, tonic::Status>> + Send>>))
        // Ok(tonic::Response::new(
        //     Box::pin(out_stream) as Self::PullStream
        // ))
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
