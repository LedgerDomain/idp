mod content_type;
mod content_typeable;
mod generated;
mod nonce;
mod plum_body_builder;
mod plum_body_seal;
mod plum_builder;
mod plum_head;
mod plum_head_builder;
mod plum_head_seal;
mod plum_relation;
mod plum_relations_builder;
mod plum_relations_seal;
mod plum_verify_error;
mod seal;
mod sha256sum;
mod unix_nanoseconds;

pub use crate::{
    content_type::{},
    content_typeable::ContentTypeable,
    generated::idp::{
        branch_set_head_request, pull_request, pull_response, push_request, push_response,
        BranchCreateRequest, BranchCreateResponse, BranchDeleteRequest, BranchDeleteResponse,
        BranchGetHeadRequest, BranchGetHeadResponse, BranchSetHeadRequest,
        BranchSetHeadResponse, Acknowledgement, ContentType, Id, Nonce, Plum, PlumBody,
        PlumBodySeal, PlumHead, PlumHeadAndRelations, PlumHeadSeal, PlumHeadSealAndRelations, PlumRelation, Path, PathState,
        PlumRelationFlagsMapping, PlumRelationFlagsRaw, PlumRelations, PlumRelationsSeal,
        PullRequest, PullResponse, PushRequest, PushResponse, Seal, Sha256Sum, UnixNanoseconds,
    },
    nonce::{},
    plum_body_builder::PlumBodyBuilder,
    plum_body_seal::{},
    plum_builder::PlumBuilder,
    plum_head::{},
    plum_head_builder::PlumHeadBuilder,
    plum_head_seal::{},
    plum_relation::{PlumRelationFlags, PlumRelational},
    plum_relations_builder::PlumRelationsBuilder,
    plum_relations_seal::{},
    plum_verify_error::{PlumVerifyError},
    seal::{},
    sha256sum::{},
    unix_nanoseconds::{},
};

#[cfg(feature = "client")]
pub use crate::generated::idp::indoor_data_plumbing_client::IndoorDataPlumbingClient;

#[cfg(feature = "server")]
pub use crate::generated::idp::indoor_data_plumbing_server::{IndoorDataPlumbing, IndoorDataPlumbingServer};

/*
//
// PushRequest
//

impl From<PushHeadRequest> for PushRequest {
    fn from(value: PushHeadRequest) -> Self {
        PushRequest {
            value: Some(push_request::Value::PushHeadRequest(value)),
        }
    }
}

impl From<PushBodyRequest> for PushRequest {
    fn from(value: PushBodyRequest) -> Self {
        PushRequest {
            value: Some(push_request::Value::PushBodyRequest(value)),
        }
    }
}

impl From<PushHeadAndBodyRequest> for PushRequest {
    fn from(value: PushHeadAndBodyRequest) -> Self {
        PushRequest {
            value: Some(push_request::Value::PushHeadAndBodyRequest(value)),
        }
    }
}

//
// PushResponse
//

impl From<PushHeadResponse> for PushResponse {
    fn from(value: PushHeadResponse) -> Self {
        PushResponse {
            value: Some(push_response::Value::PushHeadResponse(value)),
        }
    }
}

impl From<PushBodyResponse> for PushResponse {
    fn from(value: PushBodyResponse) -> Self {
        PushResponse {
            value: Some(push_response::Value::PushBodyResponse(value)),
        }
    }
}

impl From<PushHeadAndBodyResponse> for PushResponse {
    fn from(value: PushHeadAndBodyResponse) -> Self {
        PushResponse {
            value: Some(push_response::Value::PushHeadAndBodyResponse(value)),
        }
    }
}

//
// PullRequest
//

impl From<PullHeadRequest> for PullRequest {
    fn from(value: PullHeadRequest) -> Self {
        PullRequest {
            value: Some(pull_request::Value::PullHeadRequest(value)),
        }
    }
}

impl From<PullBodyRequest> for PullRequest {
    fn from(value: PullBodyRequest) -> Self {
        PullRequest {
            value: Some(pull_request::Value::PullBodyRequest(value)),
        }
    }
}

impl From<PullHeadAndBodyRequest> for PullRequest {
    fn from(value: PullHeadAndBodyRequest) -> Self {
        PullRequest {
            value: Some(pull_request::Value::PullHeadAndBodyRequest(value)),
        }
    }
}

//
// PullResponse
//

impl From<PullHeadResponse> for PullResponse {
    fn from(value: PullHeadResponse) -> Self {
        PullResponse {
            value: Some(pull_response::Value::PullHeadResponse(value)),
        }
    }
}

impl From<PullBodyResponse> for PullResponse {
    fn from(value: PullBodyResponse) -> Self {
        PullResponse {
            value: Some(pull_response::Value::PullBodyResponse(value)),
        }
    }
}

impl From<PullHeadAndBodyResponse> for PullResponse {
    fn from(value: PullHeadAndBodyResponse) -> Self {
        PullResponse {
            value: Some(pull_response::Value::PullHeadAndBodyResponse(value)),
        }
    }
}

//
// DelRequest
//

impl From<DelHeadRequest> for DelRequest {
    fn from(value: DelHeadRequest) -> Self {
        DelRequest {
            value: Some(del_request::Value::DelHeadRequest(value)),
        }
    }
}

impl From<DelBodyRequest> for DelRequest {
    fn from(value: DelBodyRequest) -> Self {
        DelRequest {
            value: Some(del_request::Value::DelBodyRequest(value)),
        }
    }
}

impl From<DelHeadAndBodyRequest> for DelRequest {
    fn from(value: DelHeadAndBodyRequest) -> Self {
        DelRequest {
            value: Some(del_request::Value::DelHeadAndBodyRequest(value)),
        }
    }
}

//
// DelResponse
//

impl From<DelHeadResponse> for DelResponse {
    fn from(value: DelHeadResponse) -> Self {
        DelResponse {
            value: Some(del_response::Value::DelHeadResponse(value)),
        }
    }
}

impl From<DelBodyResponse> for DelResponse {
    fn from(value: DelBodyResponse) -> Self {
        DelResponse {
            value: Some(del_response::Value::DelBodyResponse(value)),
        }
    }
}

impl From<DelHeadAndBodyResponse> for DelResponse {
    fn from(value: DelHeadAndBodyResponse) -> Self {
        DelResponse {
            value: Some(del_response::Value::DelHeadAndBodyResponse(value)),
        }
    }
}
*/
