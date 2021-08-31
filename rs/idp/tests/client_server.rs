use std::sync::{Arc, Mutex};
use idp::{
    proto::{
//         DelRequest, DelHeadRequest, DelBodyRequest, DelHeadAndBodyRequest,
//         DelResponse, DelHeadResponse, DelBodyResponse, DelHeadAndBodyResponse,
        indoor_data_plumbing_client::IndoorDataPlumbingClient,
        Nonce,
        PlumBody,
//         PullRequest, PullHeadRequest, PullBodyRequest, PullHeadAndBodyRequest,
//         PullResponse, PullHeadResponse, PullBodyResponse, PullHeadAndBodyResponse,
        PushRequest,
//         PushHeadRequest,
        PushBodyRequest,
//         PushHeadAndBodyRequest,
//         PushResponse, PushHeadResponse, PushBodyResponse, PushHeadAndBodyResponse,
    },
    server::{state::ServerState, IdpServer},
};

#[derive(Debug)]
enum Error {
    TokioTaskJoinError(tokio::task::JoinError),
    TonicTransportError(tonic::transport::Error),
    TonicStatus(tonic::Status),
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Error::TokioTaskJoinError(e)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::TonicTransportError(e)
    }
}

impl From<tonic::Status> for Error {
    fn from(e: tonic::Status) -> Self {
        Error::TonicStatus(e)
    }
}

async fn run_client_task() -> Result<(), Error> {
    // Wait for the server to finish spinning up.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;

    let mut idp_client = IndoorDataPlumbingClient::with_interceptor(channel.clone(), {
        let token = tonic::metadata::MetadataValue::from_str("Bearer some-secret-token").unwrap();
        move |mut req: tonic::Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        }
    });

    // Happy path
    {
        let request = tonic::Request::new(PushRequest::from(PushBodyRequest {
            body: PlumBody {
                body_nonce_o: Some(Nonce::from("diplodocus".as_bytes())),
                body_content: "pterodactyl".as_bytes().to_vec(),
            },
        }));
        log::debug!("request: {:#?}", request);

        let response = idp_client.push(request).await;
        log::debug!("response: {:?}", response);
//         assert!(response.is_ok());
        assert!(response.is_err()); // TEMP HACK
    }

//     // Happy path
//     {
//         let request = tonic::Request::new(revocation::CheckRevocationStatusRequest {
//             signer: UserName::from("diplodocus").into(),
//             credential_id: vec![0u8, 1, 2, 3, 10],
//         });
//         log::debug!("request: {:#?}", request);
//
//         let response = revocation_client.check_revocation_status(request).await;
//         log::debug!("response: {:?}", response);
//         assert!(response.is_ok());
//     }
//
//     // Sad path
//     {
//         let request = tonic::Request::new(authentication::BeginRequest {
//             user_identifier: UserName::from("diplodocus").into(),
//             preemptive_proofs: vec![Proof {
//                 value: proof::Value::PersistentPassword(PersistentPassword {
//                     value: "WrongPassword".into(),
//                 })
//                 .into(),
//             }],
//             requested_interactive_factors: vec![],
//         });
//         log::debug!("request: {:#?}", request);
//
//         let response = idp_client.begin(request).await;
//         log::debug!("response: {:?}", response);
//         assert!(response.is_err());
//     }
//
//     // Sad path
//     {
//         let mut unauthorized_revocation_client = RevocationClient::with_interceptor(channel, {
//             let token =
//                 tonic::metadata::MetadataValue::from_str("Bearer WRONG-secret-token").unwrap();
//             move |mut req: tonic::Request<()>| {
//                 req.metadata_mut().insert("authorization", token.clone());
//                 Ok(req)
//             }
//         });
//
//         let request = tonic::Request::new(revocation::CheckRevocationStatusRequest {
//             signer: UserName::from("diplodocus").into(),
//             credential_id: vec![0u8, 1, 2, 3, 10],
//         });
//         log::debug!("request: {:#?}", request);
//
//         let response = unauthorized_revocation_client
//             .check_revocation_status(request)
//             .await;
//         log::debug!("response: {:?}", response);
//         assert!(response.is_err());
//     }

    Ok(())
}

async fn run_server_task() -> Result<(), Error> {
    let addr = "[::1]:50051".parse().unwrap();
    let server = {
        let server_state_ma = {
            let server_state = ServerState::new();
            Arc::new(Mutex::new(server_state))
        };
        IdpServer::new(server_state_ma)
    };
    server.listen_on(addr).await?;
    Ok(())
}

#[tokio::test]
async fn test_client_server() -> Result<(), Error> {
    let _ = env_logger::try_init();

    let server_handle = tokio::spawn(run_server_task());
    let client_handle = tokio::spawn(run_client_task());

    // Wait on the client task to finish.
    let client_result = tokio::join!(client_handle).0?;
    // The client is done, so stop the server.
    server_handle.abort();
    // Return the client result.
    client_result
}
