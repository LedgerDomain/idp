use anyhow::Result;
use idp_core::Datahost;
use idp_proto::{indoor_data_plumbing_client::IndoorDataPlumbingClient, PlumHeadSeal, PushRequest};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct IDPClient {
    datahost_la: Arc<RwLock<Datahost>>,
    // TODO: Handle to GRPC connection
    grpc_client: IndoorDataPlumbingClient<tonic::transport::Channel>,
}

impl IDPClient {
    // TODO: Add URL for server to connect to
    pub async fn connect(datahost_la: Arc<RwLock<Datahost>>) -> Result<Self> {
        // let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        //     .connect()
        //     .await?;

        // let mut grpc_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
        //     let token = tonic::metadata::MetadataValue::from_static("Bearer some-secret-token");
        //     move |mut req: tonic::Request<()>| {
        //         req.metadata_mut().insert("authorization", token.clone());
        //         Ok(req)
        //     }
        // });
        let grpc_client = IndoorDataPlumbingClient::connect("http://[::1]:50051").await?;
        Ok(Self {
            datahost_la,
            grpc_client,
        })
    }
    pub async fn push(&mut self, plum_head_seal: &PlumHeadSeal) -> Result<()> {
        // TODO: Stream the results from the DB query instead of reading the whole thing into memory.
        let relation_flags_m = self.datahost_la.read().accumulated_relations_recursive(
            plum_head_seal,
            idp_proto::RelationFlags::CONTENT_DEPENDENCY
                | idp_proto::RelationFlags::METADATA_DEPENDENCY,
        )?;
        // Accumulated relations for a plum should not include the plum itself.
        assert!(!relation_flags_m.contains_key(plum_head_seal));
        // NOTE: Because the relations are collected via HashMap, they will be in a random order,
        // which is not ideal for short-circuiting push operations when the target Datahost already
        // has most of the Plum-s.  One solution to this is to accumulate the relations and store
        // them in (probably) a breadth-first traversal order.  A better solution would be to stream
        // the relations accumulations and only bother recursing on Plum-s that the target Datahost
        // asks to receive.
        let mut accumulated_plum_head_seal_v = Vec::with_capacity(relation_flags_m.len() + 1);
        // First, add this Plum.
        accumulated_plum_head_seal_v.push(plum_head_seal.clone());
        accumulated_plum_head_seal_v.extend(relation_flags_m.keys().into_iter().cloned());

        let datahost_la = self.datahost_la.clone();
        use tokio_stream::StreamExt;
        self.grpc_client
            .push(
                tokio_stream::iter(accumulated_plum_head_seal_v).map(move |plum_head_seal| {
                    // log::trace!("IDPClient::push; pushing plum_head_seal {}", plum_head_seal);
                    log::trace!(
                        "IDPClient::push; pushing plum with plum_head_seal {}",
                        plum_head_seal
                    );
                    // TODO Temp hack and just send every Plum.  very wasteful, but just to test transport.
                    PushRequest {
                        // value: Some(idp_proto::push_request::Value::ShouldISendThisPlum(
                        //     plum_head_seal.clone(),
                        // )),
                        value: Some(idp_proto::push_request::Value::HereHaveAPlum(
                            datahost_la.read().load_plum(&plum_head_seal).unwrap(),
                        )),
                    }
                }),
            )
            .await?;
        Ok(())
    }
}
