use crate::Datahost;
use anyhow::Result;
use async_lock::RwLock;
use idp_proto::{IndoorDataPlumbingClient, PlumHeadSeal, PullRequest, PushRequest};
use std::sync::Arc;

pub struct IDPClient {
    datahost_la: Arc<RwLock<Datahost>>,
    // TODO: Handle to GRPC connection
    grpc_client: IndoorDataPlumbingClient<tonic::transport::Channel>,
}

impl IDPClient {
    pub async fn connect(url: String, datahost_la: Arc<RwLock<Datahost>>) -> Result<Self> {
        // let channel = tonic::transport::Channel::from_static("http://0.0.0.0:50051")
        //     .connect()
        //     .await?;

        // let mut grpc_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
        //     let token = tonic::metadata::MetadataValue::from_static("Bearer some-secret-token");
        //     move |mut req: tonic::Request<()>| {
        //         req.metadata_mut().insert("authorization", token.clone());
        //         Ok(req)
        //     }
        // });
        // let grpc_client = IndoorDataPlumbingClient::connect("http://0.0.0.0:50051").await?;
        let grpc_client = IndoorDataPlumbingClient::connect(url).await?;
        Ok(Self {
            datahost_la,
            grpc_client,
        })
    }
    // Technically this could be &self, not mutable.
    pub async fn push(&mut self, plum_head_seal: &PlumHeadSeal) -> Result<()> {
        // TODO: Stream the results from the DB query instead of reading the whole thing into memory.
        let plum_relation_flags_m = self
            .datahost_la
            .read()
            .await
            .accumulated_relations_recursive(
                plum_head_seal,
                idp_proto::PlumRelationFlags::CONTENT_DEPENDENCY
                    | idp_proto::PlumRelationFlags::METADATA_DEPENDENCY,
                None,
            )
            .await?;
        log::trace!(
            "IDPClient::push({}); plum_relation_flags_m ({} entries):",
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
        assert!(!plum_relation_flags_m.contains_key(plum_head_seal));
        // NOTE: Because the plum_relations are collected via HashMap, they will be in a random order,
        // which is not ideal for short-circuiting push operations when the target Datahost already
        // has most of the Plum-s.  One solution to this is to accumulate the plum_relations and store
        // them in (probably) a breadth-first traversal order.  A better solution would be to stream
        // the plum_relations accumulations and only bother recursing on Plum-s that the target Datahost
        // asks to receive.
        let mut accumulated_plum_head_seal_v = Vec::with_capacity(plum_relation_flags_m.len() + 1);
        // First, add this Plum.
        accumulated_plum_head_seal_v.push(plum_head_seal.clone());
        accumulated_plum_head_seal_v.extend(plum_relation_flags_m.keys().into_iter().cloned());

        let datahost_la = self.datahost_la.clone();

        // TEMP HACK -- just retrieve all the plums to send, don't bother with streaming efficiently for now.
        let mut push_request_v = Vec::with_capacity(accumulated_plum_head_seal_v.len());
        for accumulated_plum_head_seal in accumulated_plum_head_seal_v.iter() {
            // log::trace!("IDPClient::push; pushing plum_head_seal {}", accumulated_plum_head_seal);
            log::trace!(
                "IDPClient::push; pushing plum with plum_head_seal {}",
                accumulated_plum_head_seal
            );
            // TODO Temp hack and just send every Plum.  very wasteful, but just to test transport.
            push_request_v.push(PushRequest {
                // value: Some(idp_proto::push_request::Value::ShouldISendThisPlum(
                //     accumulated_plum_head_seal.clone(),
                // )),
                value: Some(idp_proto::push_request::Value::HereHaveAPlum(
                    datahost_la
                        .read()
                        .await
                        .load_plum(&accumulated_plum_head_seal, None)
                        .await
                        .expect("TODO: handle this error for realsies"),
                )),
            });
        }

        // TODO: Figure out if this can be refactored to not require tokio_stream crate.
        // use tokio_stream::StreamExt;
        self.grpc_client
            .push(
                tokio_stream::iter(push_request_v),
                // tokio_stream::iter(accumulated_plum_head_seal_v).map(move |plum_head_seal| {
                //     // log::trace!("IDPClient::push; pushing plum_head_seal {}", plum_head_seal);
                //     log::trace!(
                //         "IDPClient::push; pushing plum with plum_head_seal {}",
                //         plum_head_seal
                //     );
                //     // TODO Temp hack and just send every Plum.  very wasteful, but just to test transport.
                //     PushRequest {
                //         // value: Some(idp_proto::push_request::Value::ShouldISendThisPlum(
                //         //     plum_head_seal.clone(),
                //         // )),
                //         value: Some(idp_proto::push_request::Value::HereHaveAPlum(
                //             datahost_la
                //                 .read()
                //                 .await
                //                 .load_plum(&plum_head_seal)
                //                 .await
                //                 .expect("TODO: handle this error for realsies"),
                //         )),
                //     }
                // }),
            )
            .await?;
        Ok(())
    }
    pub async fn pull(&mut self, plum_head_seal: &PlumHeadSeal) -> Result<()> {
        // TEMP HACK -- simply request the Plum, and the server will return all its recursive dependencies,
        // and the client will trust that the server is sending all the right stuff.
        let mut stream = self
            .grpc_client
            .pull(PullRequest {
                value: Some(idp_proto::pull_request::Value::IWantThisPlum(
                    plum_head_seal.clone(),
                )),
            })
            .await?
            .into_inner();

        use tokio_stream::StreamExt;
        while let Some(pull_response_r) = stream.next().await {
            let pull_response = pull_response_r?;
            match pull_response.value {
                Some(idp_proto::pull_response::Value::Plum(plum)) => {
                    // TODO: Check if we actually asked for this Plum.
                    self.datahost_la
                        .read()
                        .await
                        .store_plum(&plum, None)
                        .await?;
                }
                Some(idp_proto::pull_response::Value::IDontHaveThisPlum(plum_head_seal)) => {
                    anyhow::bail!(
                        "IDPServer indicated that it doesn't have requested Plum {}",
                        plum_head_seal
                    );
                }
                None => {
                    anyhow::bail!("IDPServer returned a malformed response");
                }
            }
        }
        // Stream is dropped here and the disconnect info is sent to server.  This is not ideal,
        // we would prefer to keep the connection and simply send more requests.  But for now,
        // just get it working and optimize later.

        // let mut traverse_this_plum_v = VecDeque::with_capacity(1);
        // traverse_this_plum_v.push_back(plum_head_seal.clone());

        // let mut pull_request_v: VecDeque<PullRequest> = VecDeque::new();

        // while let Some(traverse_this_plum) = traverse_this_plum_v.pop_front() {
        //     // Check if we have the components of this Plum.
        //     let has_plum_head = self.datahost_la.read().has_plum_head(&traverse_this_plum)?;
        //     let has_plum_relations = self
        //         .datahost_la
        //         .read()
        //         .has_plum_relations_for(&traverse_this_plum)?;
        //     match (has_plum_head, has_plum_relations) {
        //         (false, false) => {
        //             pull_request_v.push_back(PullRequest {
        //                 value: Some(
        //                     idp_proto::pull_request::Value::IWantThisPlumHeadAndRelations(
        //                         traverse_this_plum.clone(),
        //                     ),
        //                 ),
        //             });
        //         }
        //         (false, true) => {
        //             pull_request_v.push_back(PullRequest {
        //                 value: Some(idp_proto::pull_request::Value::IWantThisPlumHead(
        //                     traverse_this_plum.clone(),
        //                 )),
        //             });
        //         }
        //         (true, false) => {
        //             pull_request_v.push_back(PullRequest {
        //                 value: Some(idp_proto::pull_request::Value::IWantThisPlumRelations(
        //                     traverse_this_plum.clone(),
        //                 )),
        //             });
        //         }
        //         (true, true) => {
        //             // Traverse to the related Plums.
        //             unimplemented!("TODO");
        //         }
        //     }

        //     if has_plum_head {
        //         let plum_head = self
        //             .datahost_la
        //             .read()
        //             .select_plum_head_row(plum_head_seal)?;
        //         let has_plum_body = self
        //             .datahost_la
        //             .read()
        //             .has_plum_body(&plum_head.body_seal)?;
        //         if !has_plum_body {
        //             pull_request_v.push_back(PullRequest {
        //                 value: Some(idp_proto::pull_request::Value::IWantThisPlumBody(
        //                     plum_head.body_seal.clone(),
        //                 )),
        //             });
        //         }
        //     } else {
        //         // This is a bit inelegant, but if we don't have the PlumHead, then we're not able to
        //         // complete the Plum traversal, so put this traversal back at the front of the queue;
        //         // the server should send the PlumHead in the request below, and then we can resume
        //         // traversal.
        //         traverse_this_plum_v.push_front(traverse_this_plum);
        //     }

        //     // TODO: Stream all existing pull requests

        //     // Issue the next PullRequest in the deque (if there are any), and process its stream of responses.
        //     if let Some(pull_request) = pull_request_v.pop_front() {
        //         let mut stream = self
        //             .grpc_client
        //             .pull(pull_request)
        //             .await
        //             .unwrap()
        //             .into_inner();

        //         use tokio_stream::StreamExt;
        //         while let Some(pull_response_r) = stream.next().await {
        //             let pull_response = pull_response_r?;
        //             match pull_response.value {
        //                 Some(idp_proto::pull_response::Value::PlumHead(plum_head)) => {
        //                     // Compute the PlumHeadSeal ourselves.
        //                     let plum_head_seal = PlumHeadSeal::from(&plum_head);
        //                     if self.datahost_la.read().has_plum_head(&plum_head_seal)? {
        //                         panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumHead that we already have (and therefore didn't ask for)");
        //                     } else {
        //                         self.datahost_la.read().store_plum_head(&plum_head)?;
        //                     }
        //                 }
        //                 Some(idp_proto::pull_response::Value::PlumHeadSealAndRelations(
        //                     plum_head_seal_and_relations,
        //                 )) => {
        //                     // Compute the PlumRelationsSeal ourselves.
        //                     let plum_relations_seal = PlumRelationsSeal::from(
        //                         &plum_head_seal_and_relations.plum_relations,
        //                     );
        //                     // Check that the PlumRelationsSeal match that stored in the the PlumHead (that we already
        //                     // have -- NOTE: This assumes we already have the PlumHead, which may not be true.)
        //                     {
        //                         let plum_head = self
        //                             .datahost_la
        //                             .read()
        //                             .load_plum_head(&plum_head_seal_and_relations.plum_head_seal)
        //                             .context("PlumRelations received before PlumHead")?;
        //                         plum_head.verify_plum_relations_seal(plum_relations_seal)?;
        //                         // anyhow::ensure!(plum_head.plum_relations_seal_o.is_some());
        //                         // anyhow::ensure!(
        //                         //     plum_relations_seal
        //                         //         == *plum_head.plum_relations_seal_o.as_ref().unwrap()
        //                         // );
        //                     }
        //                     if self.datahost_la.read().has_plum_relations_for(
        //                         &plum_head_seal_and_relations.plum_head_seal,
        //                     )? {
        //                         panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumHead that we already have (and therefore didn't ask for)");
        //                     } else {
        //                         self.datahost_la.read().store_plum_relations(
        //                             &plum_head_seal_and_relations.plum_head_seal,
        //                             &plum_head_seal_and_relations.plum_relations,
        //                         )?;
        //                     }
        //                 }
        //                 Some(idp_proto::pull_response::Value::PlumHeadAndRelations(
        //                     plum_head_and_relations,
        //                 )) => {
        //                     // Compute the PlumHeadSeal ourselves.
        //                     let plum_head_seal =
        //                         PlumHeadSeal::from(&plum_head_and_relations.plum_head);
        //                     // Compute the PlumRelationsSeal ourselves.
        //                     let plum_relations_seal =
        //                         PlumRelationsSeal::from(&plum_head_and_relations.plum_relations);
        //                     // Check that the PlumRelationsSeal match that stored in the PlumHead.
        //                     {
        //                         anyhow::ensure!(plum_head_and_relations
        //                             .plum_head
        //                             .plum_relations_seal_o
        //                             .is_some());
        //                         anyhow::ensure!(
        //                             plum_relations_seal
        //                                 == *plum_head_and_relations
        //                                     .plum_head
        //                                     .plum_relations_seal_o
        //                                     .as_ref()
        //                                     .unwrap()
        //                         );
        //                     }
        //                     if self.datahost_la.read().has_plum_head(&plum_head_seal)? {
        //                         panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumHead that we already have (and therefore didn't ask for)");
        //                     } else {
        //                         self.datahost_la
        //                             .read()
        //                             .store_plum_head(&plum_head_and_relations.plum_head)?;
        //                     }
        //                     if self
        //                         .datahost_la
        //                         .read()
        //                         .has_plum_relations_for(&plum_head_seal)?
        //                     {
        //                         panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumHead that we already have (and therefore didn't ask for)");
        //                     } else {
        //                         self.datahost_la.read().store_plum_relations(
        //                             &plum_head_seal,
        //                             &plum_head_and_relations.plum_relations,
        //                         )?;
        //                     }
        //                 }
        //                 Some(idp_proto::pull_response::Value::PlumBody(plum_body)) => {
        //                     // Compute the PlumBodySeal ourselves.
        //                     // TODO: Figure out how to stream it -- this would involve streaming chunks of the
        //                     // PlumBody, and probably writing those chunks to the filesystem, since writing
        //                     // chunks to a DB is maybe not feasible.  Or the DB could store chunks directly,
        //                     // though that makes other operations difficult, such as deserializing a PlumBody.
        //                     let plum_body_seal = PlumBodySeal::from(&plum_body);
        //                     if self.datahost_la.read().has_plum_body(&plum_body_seal)? {
        //                         panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumBody that we already have (and therefore didn't ask for)");
        //                     } else {
        //                         self.datahost_la.read().store_plum_body(&plum_body)?;
        //                     }
        //                 }
        //                 Some(idp_proto::pull_response::Value::Plum(plum)) => {
        //                     let plum_head_seal = PlumHeadSeal::from(&plum.plum_head);
        //                     plum_head_seal.
        //                 }
        //                 None => {
        //                     anyhow::bail!("malformed PullResponse");
        //                 }
        //             }
        //         }
        //         // Stream is dropped here and the disconnect info is sent to server.  This is not ideal,
        //         // we would prefer to keep the connection and simply send more requests.  But for now,
        //         // just get it working and optimize later.
        //     }
        // }

        // let (has_plum_head, has_plum_relations) = {
        //     let datahost_g = self.datahost_la.read();
        //     let has_plum_head = datahost_g.has_plum_head(plum_head_seal)?;
        //     let has_plum_relations = datahost_g.has_plum_relations_for(plum_head_seal)?;
        //     if has_plum_head && has_plum_relations {
        //         let plum_head = datahost_g.select_plum_head_row(plum_head_seal)?;
        //         if datahost_g.has_plum_body(&plum_head.body_seal) {
        //             // If we already have this PlumHead, its PlumRelations, and its PlumBody, then
        //             // there's nothing to do (assuming the dependency graph hasn't been broken by
        //             // some outside process or action on the DB).
        //             // TODO: Could do a check that all recursive dependencies are present (i.e. the
        //             // DB is consistent).
        //             return Ok(());
        //         }
        //     }
        //     (has_plum_head, has_plum_relations)
        // };

        // // TODO: This could be made into a bidirectional streaming call, but for now we just want
        // // to get it working, and not prematurely optimize.

        // let mut pull_request_v = VecDeque::new();
        // match (has_plum_head, has_plum_relations) {
        //     (true, true)
        // }
        // if has_plum_head {
        //     if has_plum_relations {

        //     } else {

        //     }
        // } else {
        //     if has_plum_relations {

        //     } else {

        //     }
        // }
        // pull_request_v.push_back(PullRequest { value: idp_proto::pull_request::Value::IWantThisPlumHead(())})

        // while let Some(i_want_this_plum_head) = take_any_element_from_hash_set(&mut i_want_this_plum_head_s) {
        //     let mut stream = self
        //         .grpc_client
        //         .pull(PullRequest { value: Some(idp_proto::pull_request::Value::IWantThisPlumHead(i_want_this_plum)) })
        //         .await
        //         .unwrap()
        //         .into_inner();

        //     use tokio_stream::StreamExt;
        //     while let Some(pull_response_r) = stream.next().await {
        //         let pull_response = pull_response_r?;
        //         match pull_response.value {
        //             Some(idp_proto::pull_response::Value::PlumHead(plum_head)) => {
        //                 // Compute the PlumHeadSeal ourselves.
        //                 let plum_head_seal = PlumHeadSeal::from(&plum_head);
        //                 // Check if we already have (1) this PlumHead, (2) its PlumRelations, and (3) its PlumBody.
        //                 if self.datahost_la.read().has_plum_head(&plum_head_seal)? {
        //                     panic!("programmer error (potentially in the server): we shouldn't be receiving a PlumHead that we already have (and therefore didn't ask for)");
        //                 } else {
        //                     self.datahost_la.read().store_plum_head(&plum_head)?;
        //                 }

        //                 if !self.datahost_la.read().has_plum_relations_for(&plum_head_seal)? {
        //                     i_want_this_plum_relations_s.insert(plum_head_seal.clone());
        //                 }

        //                 if !self.datahost_la.read().has_plum_body(&plum_head.body_seal)? {
        //                     i_want_this_plum_body_s.insert(plum_head.body_seal.clone());
        //                 }
        //             }
        //             Some(idp_proto::pull_response::Value::PlumRelations(plum_relations)) => {
        //                 self.datahost_la.read().store_plum_relations(source_head_seal, &plum_relations)
        //             }
        //             Some(idp_proto::pull_response::Value::HereHaveAPlum(plum)) => {
        //                 self.datahost_la.read().store_plum(&plum)?;
        //             }
        //             None => {
        //                 anyhow::bail!("malformed PullResponse");
        //             }
        //         }
        //     }
        //     // Stream is dropped here and the disconnect info is sent to server.  This is not ideal,
        //     // we would prefer to keep the connection and simply send more requests.  But for now,
        //     // just get it working and optimize later.
        // }

        Ok(())
    }
}

// /// This will remove and return the "first" element (in the randomized order that the HashSet stores
// /// elements in), or None if the HashSet is empty.
// fn take_any_element_from_hash_set<T>(s: &mut HashSet<T>) -> Option<T>
// where
//     T: Clone + std::cmp::Eq + std::hash::Hash,
// {
//     // match s.iter().next() {
//     //     Some(value) => s.take
//     // }

//     if s.is_empty() {
//         return None;
//     }

//     // NOTE: This clone is only to be able to stop immutably borrowing s before calling s.take.
//     // TODO: Fix this, a clone is not strictly logically necessary
//     let value = s.iter().next().unwrap().clone();
//     s.take(&value)
// }
