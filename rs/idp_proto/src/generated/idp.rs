#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContentType {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Id {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(serde::Deserialize, derive_more::From, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Nonce {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(
    derive_more::Deref,
    serde::Deserialize,
    Eq,
    derive_more::From,
    Hash,
    Ord,
    PartialOrd,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Seal {
    /// TEMP HACK -- it should support more seal types
    #[prost(message, required, tag = "1")]
    pub sha256sum: Sha256Sum,
}
#[derive(
    derive_more::Deref,
    serde::Deserialize,
    Eq,
    derive_more::From,
    Hash,
    Ord,
    PartialOrd,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sha256Sum {
    #[prost(bytes = "vec", required, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// Storing nanoseconds in an int64 gives 292.27 years range around the Unix epoch, 1970-01-01 UTC.
#[derive(
    Copy,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnixNanoseconds {
    #[prost(int64, required, tag = "1")]
    pub value: i64,
}
#[derive(
    derive_more::Deref,
    serde::Deserialize,
    Eq,
    derive_more::From,
    Hash,
    Ord,
    PartialOrd,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHeadSeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[derive(
    derive_more::Deref,
    serde::Deserialize,
    Eq,
    derive_more::From,
    Hash,
    Ord,
    PartialOrd,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelationsSeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[derive(derive_more::Deref, serde::Deserialize, derive_more::From, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumBodySeal {
    #[prost(message, required, tag = "1")]
    pub value: Seal,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHead {
    /// Optional nonce for preventing dictionary attacks.  This would be left as None e.g. in storing "a plain file"
    /// or otherwise for data that has no need for the protection the nonce provides.
    #[prost(message, optional, tag = "1")]
    pub plum_head_nonce_o: ::core::option::Option<Nonce>,
    /// Optional PlumRelationsSeal uniquely identifies a PlumRelations (for authentication of PlumRelations-es).
    /// This would be left as None e.g. in storing "a plain file", or otherwise for data that doesn't have any
    /// formal plum_relations.
    #[prost(message, optional, tag = "2")]
    pub plum_relations_seal_o: ::core::option::Option<PlumRelationsSeal>,
    /// PlumBodySeal uniquely identifies a PlumBody (for authentication and lookup into the DB/store of PlumBody-s)
    #[prost(message, required, tag = "3")]
    pub plum_body_seal: PlumBodySeal,
    /// Optional owner DID.  This would be left as None e.g. in storing "a plain file", or otherwise for data that
    /// has no need for a formal owner.
    #[prost(message, optional, tag = "4")]
    pub owner_id_o: ::core::option::Option<Id>,
    /// Optional Plum creation timestamp.
    #[prost(message, optional, tag = "5")]
    pub created_at_o: ::core::option::Option<UnixNanoseconds>,
    /// Optional, unstructured metadata.  This would be left as None e.g. in storing "a plain file", or otherwise
    /// for data that has no need for metadata.
    #[prost(bytes = "vec", optional, tag = "6")]
    pub metadata_o: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
/// A set of Relations, encoded as bitflags.
#[derive(Copy, serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelationFlagsRaw {
    #[prost(uint32, required, tag = "1")]
    pub value: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelationFlagsMapping {
    #[prost(message, required, tag = "1")]
    pub target_plum_head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub plum_relation_flags_raw: PlumRelationFlagsRaw,
}
/// This encapsulates the Relations from a given Plum to all others, and is derived from its PlumBody.
/// The reason this is separate is because there are situations where the PlumBody won't be present
/// but that Plum's Relations are needed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumRelations {
    /// Optional nonce can be used to prevent dictionary attacks.
    #[prost(message, optional, tag = "1")]
    pub plum_relations_nonce_o: ::core::option::Option<Nonce>,
    /// PlumBodySeal of the Plum that these relations come from.
    #[prost(message, required, tag = "2")]
    pub source_plum_body_seal: PlumBodySeal,
    /// Content of the plum_relations itself.  This consists of entries to add to the plum_relations DB table.
    #[prost(message, repeated, tag = "3")]
    pub plum_relation_flags_mapping_v: ::prost::alloc::vec::Vec<
        PlumRelationFlagsMapping,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumBody {
    /// Optional nonce can be used to prevent dictionary attacks.
    #[prost(message, optional, tag = "1")]
    pub plum_body_nonce_o: ::core::option::Option<Nonce>,
    /// Number of bytes in the Plum body itself.
    #[prost(uint64, required, tag = "2")]
    pub plum_body_content_length: u64,
    /// Content type for the Plum body.
    #[prost(message, required, tag = "3")]
    pub plum_body_content_type: ContentType,
    /// Content of the plum itself.  The content type of the bytes is given in the PlumHead.
    #[prost(bytes = "vec", required, tag = "4")]
    pub plum_body_content: ::prost::alloc::vec::Vec<u8>,
}
/// This represents a single data entry; it's a head (metadata), plum_relations, and a body (file content).
/// Yes, a stupid name, and I hate cute names in software, but it is distinct, and it's a noun.
/// And at least it doesn't end with "ly".
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Plum {
    #[prost(message, required, tag = "1")]
    pub plum_head: PlumHead,
    #[prost(message, optional, tag = "2")]
    pub plum_relations_o: ::core::option::Option<PlumRelations>,
    #[prost(message, required, tag = "3")]
    pub plum_body: PlumBody,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Acknowledgement {}
/// These stream from client to server, starting with the PlumHeadSeal, and the server streams
/// responses to say which PlumHeadSeals it already has Plums for (and therefore the client
/// doesn't have to push the Plum or recurse on its dependencies).  Thus there won't be much
/// wasted bandwidth.
/// TODO: break it apart into sending plum head, plum plum_relations, plum body.  This requires
/// the server responding with which ones are needed for a given PlumHeadSeal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushRequest {
    #[prost(oneof = "push_request::Value", tags = "1, 2")]
    pub value: ::core::option::Option<push_request::Value>,
}
/// Nested message and enum types in `PushRequest`.
pub mod push_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        ShouldISendThisPlum(super::PlumHeadSeal),
        #[prost(message, tag = "2")]
        HereHaveAPlum(super::Plum),
    }
}
/// TODO: Potentially could respond with a boolean, as long as the client can reliably
/// pair the response with the request in the bidirectional streaming.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushResponse {
    #[prost(oneof = "push_response::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<push_response::Value>,
}
/// Nested message and enum types in `PushResponse`.
pub mod push_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// Response to here_have_a_plum; carries no information.  TODO: This isn't actually necessary,
        /// as long as the request/response streaming doesn't need to be 1-to-1
        #[prost(message, tag = "1")]
        Ok(super::Acknowledgement),
        /// Positive response to should_i_send_this_plum.  TODO: Maybe rename to i_want_this_plum.
        #[prost(message, tag = "2")]
        SendThisPlum(super::PlumHeadSeal),
        /// Negative response to should_i_send_this_plum.  TODO: This isn't actually necessary,
        /// as long as the request/response streaming doesn't need to be 1-to-1
        #[prost(message, tag = "3")]
        DontSendThisPlum(super::PlumHeadSeal),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHeadAndRelations {
    #[prost(message, required, tag = "1")]
    pub plum_head: PlumHead,
    #[prost(message, required, tag = "2")]
    pub plum_relations: PlumRelations,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlumHeadSealAndRelations {
    #[prost(message, required, tag = "1")]
    pub plum_head_seal: PlumHeadSeal,
    #[prost(message, required, tag = "2")]
    pub plum_relations: PlumRelations,
}
/// These stream from client to server, starting with the PlumHeadSeal, and the server streams
/// responses to say which PlumHeadSeals it already has Plums for (and therefore the client
/// doesn't have to push the Plum or recurse on its dependencies).  Thus there won't be much
/// wasted bandwidth.
/// TODO: break it apart into sending plum head, plum plum_relations, plum body.  This requires
/// the server responding with which ones are needed for a given PlumHeadSeal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullRequest {
    #[prost(oneof = "pull_request::Value", tags = "1")]
    pub value: ::core::option::Option<pull_request::Value>,
}
/// Nested message and enum types in `PullRequest`.
pub mod pull_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// TEMP HACK -- simple for now.
        #[prost(message, tag = "1")]
        IWantThisPlum(super::PlumHeadSeal),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullResponse {
    #[prost(oneof = "pull_response::Value", tags = "1, 2")]
    pub value: ::core::option::Option<pull_response::Value>,
}
/// Nested message and enum types in `PullResponse`.
pub mod pull_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// TEMP HACK -- simple for now.
        #[prost(message, tag = "1")]
        Plum(super::Plum),
        #[prost(message, tag = "2")]
        IDontHaveThisPlum(super::PlumHeadSeal),
    }
}
#[derive(
    derive_more::Deref,
    serde::Deserialize,
    derive_more::Display,
    derive_more::From,
    serde::Serialize
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Path {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PathState {
    #[prost(message, required, tag = "1")]
    pub path: Path,
    /// TODO: Consider including the "updated at" timestamp
    #[prost(message, required, tag = "2")]
    pub current_state_plum_head_seal: PlumHeadSeal,
}
/// The requester should have already pushed the BranchNode Plum referred to in this request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchCreateRequest {
    #[prost(message, required, tag = "1")]
    pub branch_path_state: PathState,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchCreateResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchDeleteRequest {
    #[prost(message, required, tag = "1")]
    pub branch_path: Path,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchDeleteResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchGetHeadRequest {
    #[prost(message, required, tag = "1")]
    pub branch_path: Path,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchGetHeadResponse {
    #[prost(message, required, tag = "1")]
    pub branch_head_plum_head_seal: PlumHeadSeal,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchSetHeadRequest {
    #[prost(message, required, tag = "1")]
    pub branch_path: Path,
    #[prost(oneof = "branch_set_head_request::Value", tags = "2, 3, 4")]
    pub value: ::core::option::Option<branch_set_head_request::Value>,
}
/// Nested message and enum types in `BranchSetHeadRequest`.
pub mod branch_set_head_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "2")]
        BranchFastForwardToPlumHeadSeal(super::PlumHeadSeal),
        #[prost(message, tag = "3")]
        BranchRewindToPlumHeadSeal(super::PlumHeadSeal),
        #[prost(message, tag = "4")]
        BranchForceResetToPlumHeadSeal(super::PlumHeadSeal),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BranchSetHeadResponse {}
/// This defines what plum_relations are possible from one Plum to another.
#[derive(serde::Deserialize, num_derive::FromPrimitive, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PlumRelation {
    ContentDependency = 0,
    MetadataDependency = 1,
}
impl PlumRelation {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PlumRelation::ContentDependency => "CONTENT_DEPENDENCY",
            PlumRelation::MetadataDependency => "METADATA_DEPENDENCY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CONTENT_DEPENDENCY" => Some(Self::ContentDependency),
            "METADATA_DEPENDENCY" => Some(Self::MetadataDependency),
            _ => None,
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod indoor_data_plumbing_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct IndoorDataPlumbingClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IndoorDataPlumbingClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
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
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> IndoorDataPlumbingClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            IndoorDataPlumbingClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn push(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::PushRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::PushResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/Push",
            );
            self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
        pub async fn pull(
            &mut self,
            request: impl tonic::IntoRequest<super::PullRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::PullResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/Pull",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// TEMP HACK
        /// TODO: Consider moving these into a separate GRPC service
        pub async fn branch_create(
            &mut self,
            request: impl tonic::IntoRequest<super::BranchCreateRequest>,
        ) -> Result<tonic::Response<super::BranchCreateResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/BranchCreate",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn branch_delete(
            &mut self,
            request: impl tonic::IntoRequest<super::BranchDeleteRequest>,
        ) -> Result<tonic::Response<super::BranchDeleteResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/BranchDelete",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn branch_get_head(
            &mut self,
            request: impl tonic::IntoRequest<super::BranchGetHeadRequest>,
        ) -> Result<tonic::Response<super::BranchGetHeadResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/BranchGetHead",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn branch_set_head(
            &mut self,
            request: impl tonic::IntoRequest<super::BranchSetHeadRequest>,
        ) -> Result<tonic::Response<super::BranchSetHeadResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/idp.IndoorDataPlumbing/BranchSetHead",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod indoor_data_plumbing_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with IndoorDataPlumbingServer.
    #[async_trait]
    pub trait IndoorDataPlumbing: Send + Sync + 'static {
        /// Server streaming response type for the Push method.
        type PushStream: futures_core::Stream<
                Item = Result<super::PushResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn push(
            &self,
            request: tonic::Request<tonic::Streaming<super::PushRequest>>,
        ) -> Result<tonic::Response<Self::PushStream>, tonic::Status>;
        /// Server streaming response type for the Pull method.
        type PullStream: futures_core::Stream<
                Item = Result<super::PullResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn pull(
            &self,
            request: tonic::Request<super::PullRequest>,
        ) -> Result<tonic::Response<Self::PullStream>, tonic::Status>;
        /// TEMP HACK
        /// TODO: Consider moving these into a separate GRPC service
        async fn branch_create(
            &self,
            request: tonic::Request<super::BranchCreateRequest>,
        ) -> Result<tonic::Response<super::BranchCreateResponse>, tonic::Status>;
        async fn branch_delete(
            &self,
            request: tonic::Request<super::BranchDeleteRequest>,
        ) -> Result<tonic::Response<super::BranchDeleteResponse>, tonic::Status>;
        async fn branch_get_head(
            &self,
            request: tonic::Request<super::BranchGetHeadRequest>,
        ) -> Result<tonic::Response<super::BranchGetHeadResponse>, tonic::Status>;
        async fn branch_set_head(
            &self,
            request: tonic::Request<super::BranchSetHeadRequest>,
        ) -> Result<tonic::Response<super::BranchSetHeadResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct IndoorDataPlumbingServer<T: IndoorDataPlumbing> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: IndoorDataPlumbing> IndoorDataPlumbingServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for IndoorDataPlumbingServer<T>
    where
        T: IndoorDataPlumbing,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/idp.IndoorDataPlumbing/Push" => {
                    #[allow(non_camel_case_types)]
                    struct PushSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::StreamingService<super::PushRequest>
                    for PushSvc<T> {
                        type Response = super::PushResponse;
                        type ResponseStream = T::PushStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::PushRequest>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).push(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/Pull" => {
                    #[allow(non_camel_case_types)]
                    struct PullSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::ServerStreamingService<super::PullRequest>
                    for PullSvc<T> {
                        type Response = super::PullResponse;
                        type ResponseStream = T::PullStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PullRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).pull(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PullSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/BranchCreate" => {
                    #[allow(non_camel_case_types)]
                    struct BranchCreateSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::UnaryService<super::BranchCreateRequest>
                    for BranchCreateSvc<T> {
                        type Response = super::BranchCreateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BranchCreateRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).branch_create(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BranchCreateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/BranchDelete" => {
                    #[allow(non_camel_case_types)]
                    struct BranchDeleteSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::UnaryService<super::BranchDeleteRequest>
                    for BranchDeleteSvc<T> {
                        type Response = super::BranchDeleteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BranchDeleteRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).branch_delete(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BranchDeleteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/BranchGetHead" => {
                    #[allow(non_camel_case_types)]
                    struct BranchGetHeadSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::UnaryService<super::BranchGetHeadRequest>
                    for BranchGetHeadSvc<T> {
                        type Response = super::BranchGetHeadResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BranchGetHeadRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).branch_get_head(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BranchGetHeadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/idp.IndoorDataPlumbing/BranchSetHead" => {
                    #[allow(non_camel_case_types)]
                    struct BranchSetHeadSvc<T: IndoorDataPlumbing>(pub Arc<T>);
                    impl<
                        T: IndoorDataPlumbing,
                    > tonic::server::UnaryService<super::BranchSetHeadRequest>
                    for BranchSetHeadSvc<T> {
                        type Response = super::BranchSetHeadResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BranchSetHeadRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).branch_set_head(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BranchSetHeadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: IndoorDataPlumbing> Clone for IndoorDataPlumbingServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: IndoorDataPlumbing> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: IndoorDataPlumbing> tonic::server::NamedService
    for IndoorDataPlumbingServer<T> {
        const NAME: &'static str = "idp.IndoorDataPlumbing";
    }
}
