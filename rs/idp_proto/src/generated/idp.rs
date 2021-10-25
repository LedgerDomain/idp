// import public "idp_common.proto";

//
// Helper types
//

#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContentType {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "String")]
#[diesel(serialize_as = "String")]
#[sql_type = "diesel::sql_types::Text"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Did {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Nonce {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
// message Seal {
//     oneof value {
//         Sha256Sum Sha256Sum = 1;
//         // TODO: Crypto signature types
//     }
// }

#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Seal {
    /// TEMP HACK -- it should support more seal types
    #[prost(message, required, tag = "1")]
    pub sha256sum: Sha256Sum,
}
#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sha256Sum {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "i64")]
#[diesel(serialize_as = "i64")]
#[sql_type = "diesel::sql_types::BigInt"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnixSeconds {
    #[prost(int64, required, tag = "1")]
    pub value: i64,
}
//
// Plum-specific types
//

#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHeadSeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[derive(diesel::AsExpression, Eq, Hash, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelationsSeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[derive(diesel::AsExpression, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "Vec<u8>")]
#[diesel(serialize_as = "Vec<u8>")]
#[sql_type = "diesel::sql_types::Binary"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumBodySeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHead {
    /// PlumBodySeal uniquely identifies a PlumBody (for authentication and lookup into the DB/store of PlumBody-s)
    #[prost(message, required, tag = "1")]
    pub body_seal: PlumBodySeal,
    /// Content type for the Plum body.
    #[prost(message, required, tag = "2")]
    pub body_content_type: ContentType,
    /// Number of bytes in the Plum body itself.
    #[prost(uint64, required, tag = "3")]
    pub body_length: u64,
    /// Optional nonce for preventing dictionary attacks.
    #[prost(message, optional, tag = "4")]
    pub head_nonce_o: ::core::option::Option<Nonce>,
    /// Optional owner DID.
    #[prost(message, optional, tag = "5")]
    pub owner_did_o: ::core::option::Option<Did>,
    /// Optional Plum creation timestamp.
    #[prost(message, optional, tag = "6")]
    pub created_at_o: ::core::option::Option<UnixSeconds>,
    /// Optional, unstructured metadata.
    #[prost(bytes = "vec", optional, tag = "7")]
    pub metadata_o: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    /// Optional PlumRelationsSeal uniquely identifies a PlumRelations (for authentication of PlumRelations-es)
    #[prost(message, optional, tag = "8")]
    pub relations_seal_o: ::core::option::Option<PlumRelationsSeal>,
}
/// A set of Relations, encoded as bitflags.
#[derive(diesel::AsExpression, Copy, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "i32")]
#[diesel(serialize_as = "i32")]
#[sql_type = "diesel::sql_types::Integer"]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationFlagsRaw {
    #[prost(uint32, required, tag = "1")]
    pub value: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelationFlagsMapping {
    #[prost(message, required, tag = "1")]
    pub target_head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub relation_flags_raw: RelationFlagsRaw,
}
/// This encapsulates the Relations from a given Plum to all others, and is derived from its PlumBody.
/// The reason this is separate is because there are situations where the PlumBody won't be present
/// but that Plum's Relations are needed.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelations {
    /// Optional nonce can be used to prevent dictionary attacks.
    #[prost(message, optional, tag = "1")]
    pub relations_nonce_o: ::core::option::Option<Nonce>,
    /// Content of the relations itself.  This consists of entries to add to the relations DB table.
    #[prost(message, repeated, tag = "2")]
    pub relation_flags_mappings: ::prost::alloc::vec::Vec<PlumRelationFlagsMapping>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumBody {
    /// Optional nonce can be used to prevent dictionary attacks.
    #[prost(message, optional, tag = "1")]
    pub body_nonce_o: ::core::option::Option<Nonce>,
    /// Content of the plum itself.  The content type of the bytes is given in the PlumHead.
    #[prost(bytes = "vec", required, tag = "2")]
    pub body_content: ::prost::alloc::vec::Vec<u8>,
}
/// This represents a single data entry; it's a head (metadata), relations, and a body (file content).
/// Yes, a stupid name, and I hate cute names in software, but it is distinct, and it's a noun.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Plum {
    #[prost(message, required, tag = "1")]
    pub head: PlumHead,
    #[prost(message, optional, tag = "2")]
    pub relations_o: ::core::option::Option<PlumRelations>,
    #[prost(message, required, tag = "3")]
    pub body: PlumBody,
}
//
// Requests and Responses
//

//
// Push
//

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushHeadRequest {
    #[prost(message, required, tag = "1")]
    pub head: PlumHead,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushBodyRequest {
    #[prost(message, required, tag = "1")]
    pub body: PlumBody,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushHeadAndBodyRequest {
    #[prost(message, required, tag = "1")]
    pub plum: Plum,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushRequest {
    #[prost(oneof = "push_request::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<push_request::Value>,
}
/// Nested message and enum types in `PushRequest`.
pub mod push_request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        PushHeadRequest(super::PushHeadRequest),
        #[prost(message, tag = "2")]
        PushBodyRequest(super::PushBodyRequest),
        #[prost(message, tag = "3")]
        PushHeadAndBodyRequest(super::PushHeadAndBodyRequest),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushHeadResponse {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushBodyResponse {
    #[prost(message, required, tag = "1")]
    pub body_seal: PlumBodySeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushHeadAndBodyResponse {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub body_seal: PlumBodySeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushResponse {
    #[prost(oneof = "push_response::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<push_response::Value>,
}
/// Nested message and enum types in `PushResponse`.
pub mod push_response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        PushHeadResponse(super::PushHeadResponse),
        #[prost(message, tag = "2")]
        PushBodyResponse(super::PushBodyResponse),
        #[prost(message, tag = "3")]
        PushHeadAndBodyResponse(super::PushHeadAndBodyResponse),
    }
}
//
// Pull
//

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullHeadRequest {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullBodyRequest {
    #[prost(message, required, tag = "1")]
    pub body_seal: PlumBodySeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullHeadAndBodyRequest {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub body_seal: PlumBodySeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullRequest {
    #[prost(oneof = "pull_request::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<pull_request::Value>,
}
/// Nested message and enum types in `PullRequest`.
pub mod pull_request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        PullHeadRequest(super::PullHeadRequest),
        #[prost(message, tag = "2")]
        PullBodyRequest(super::PullBodyRequest),
        #[prost(message, tag = "3")]
        PullHeadAndBodyRequest(super::PullHeadAndBodyRequest),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullHeadResponse {
    #[prost(message, required, tag = "1")]
    pub head: PlumHead,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullBodyResponse {
    #[prost(message, required, tag = "1")]
    pub body: PlumBody,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullHeadAndBodyResponse {
    #[prost(message, required, tag = "1")]
    pub plum: Plum,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullResponse {
    #[prost(oneof = "pull_response::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<pull_response::Value>,
}
/// Nested message and enum types in `PullResponse`.
pub mod pull_response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        PullHeadResponse(super::PullHeadResponse),
        #[prost(message, tag = "2")]
        PullBodyResponse(super::PullBodyResponse),
        #[prost(message, tag = "3")]
        PullHeadAndBodyResponse(super::PullHeadAndBodyResponse),
    }
}
//
// Del
//

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelHeadRequest {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelBodyRequest {
    #[prost(message, required, tag = "1")]
    pub body_seal: PlumBodySeal,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelHeadAndBodyRequest {
    #[prost(message, required, tag = "1")]
    pub head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub body_seal: PlumBodySeal,
}
/// TODO: Could implement bidirectional streaming of Del.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelRequest {
    #[prost(oneof = "del_request::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<del_request::Value>,
}
/// Nested message and enum types in `DelRequest`.
pub mod del_request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        DelHeadRequest(super::DelHeadRequest),
        #[prost(message, tag = "2")]
        DelBodyRequest(super::DelBodyRequest),
        #[prost(message, tag = "3")]
        DelHeadAndBodyRequest(super::DelHeadAndBodyRequest),
    }
}
/// Nothing needed
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelHeadResponse {}
/// Nothing needed
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelBodyResponse {}
/// Nothing needed
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelHeadAndBodyResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelResponse {
    #[prost(oneof = "del_response::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<del_response::Value>,
}
/// Nested message and enum types in `DelResponse`.
pub mod del_response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        DelHeadResponse(super::DelHeadResponse),
        #[prost(message, tag = "2")]
        DelBodyResponse(super::DelBodyResponse),
        #[prost(message, tag = "3")]
        DelHeadAndBodyResponse(super::DelHeadAndBodyResponse),
    }
}
/// This defines what relations are possible from one Plum to another.
#[derive(diesel::AsExpression, num_derive::FromPrimitive, serde::Deserialize, serde::Serialize)]
#[diesel(deserialize_as = "i32")]
#[diesel(serialize_as = "i32")]
#[sql_type = "diesel::sql_types::Integer"]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Relation {
    ContentDependency = 0,
    MetadataDependency = 1,
}
#[doc = r" Generated client implementations."]
pub mod indoor_data_plumbing_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct IndoorDataPlumbingClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IndoorDataPlumbingClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> IndoorDataPlumbingClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        #[doc = " TODO: Figure out what to pass so that the server could say \"i already have that\" and then"]
        #[doc = " not transfer the bulk of the data."]
        #[doc = " TODO: Could implement bidirectional streaming of Push."]
        pub async fn push(
            &mut self,
            request: impl tonic::IntoRequest<super::PushRequest>,
        ) -> Result<tonic::Response<super::PushResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/idp.IndoorDataPlumbing/Push");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " TODO: Could implement bidirectional streaming of Pull."]
        pub async fn pull(
            &mut self,
            request: impl tonic::IntoRequest<super::PullRequest>,
        ) -> Result<tonic::Response<super::PullResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/idp.IndoorDataPlumbing/Pull");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn del(
            &mut self,
            request: impl tonic::IntoRequest<super::DelRequest>,
        ) -> Result<tonic::Response<super::DelResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/idp.IndoorDataPlumbing/Del");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for IndoorDataPlumbingClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for IndoorDataPlumbingClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "IndoorDataPlumbingClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod indoor_data_plumbing_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with IndoorDataPlumbingServer."]
    #[async_trait]
    pub trait IndoorDataPlumbing: Send + Sync + 'static {
        #[doc = " TODO: Figure out what to pass so that the server could say \"i already have that\" and then"]
        #[doc = " not transfer the bulk of the data."]
        #[doc = " TODO: Could implement bidirectional streaming of Push."]
        async fn push(
            &self,
            request: tonic::Request<super::PushRequest>,
        ) -> Result<tonic::Response<super::PushResponse>, tonic::Status>;
        #[doc = " TODO: Could implement bidirectional streaming of Pull."]
        async fn pull(
            &self,
            request: tonic::Request<super::PullRequest>,
        ) -> Result<tonic::Response<super::PullResponse>, tonic::Status>;
        async fn del(
            &self,
            request: tonic::Request<super::DelRequest>,
        ) -> Result<tonic::Response<super::DelResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct IndoorDataPlumbingServer<T: IndoorDataPlumbing> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: IndoorDataPlumbing> IndoorDataPlumbingServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for IndoorDataPlumbingServer<T>
    where
        T: IndoorDataPlumbing,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/idp.IndoorDataPlumbing/Push" => {
                    #[allow(non_camel_case_types)]
                    struct PushSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<T: IndoorDataPlumbing> tonic::server::UnaryService<super::PushRequest> for PushSvc<T> {
                        type Response = super::PushResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PushRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).push(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = PushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/Pull" => {
                    #[allow(non_camel_case_types)]
                    struct PullSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<T: IndoorDataPlumbing> tonic::server::UnaryService<super::PullRequest> for PullSvc<T> {
                        type Response = super::PullResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PullRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).pull(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = PullSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/Del" => {
                    #[allow(non_camel_case_types)]
                    struct DelSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<T: IndoorDataPlumbing> tonic::server::UnaryService<super::DelRequest> for DelSvc<T> {
                        type Response = super::DelResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DelRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).del(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = DelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: IndoorDataPlumbing> Clone for IndoorDataPlumbingServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: IndoorDataPlumbing> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: IndoorDataPlumbing> tonic::transport::NamedService for IndoorDataPlumbingServer<T> {
        const NAME: &'static str = "idp.IndoorDataPlumbing";
    }
}
