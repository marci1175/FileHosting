use self::messages::{serving_server::Serving, serving_server::ServingServer};
use tokio::sync::mpsc::Receiver;

use tonic::{async_trait, transport::Server, Request, Response, Status};
use common_definitions::{ClientRequest, PathItem, ServerFile, ServerList, ServerReply};

pub mod messages {
    tonic::include_proto!("file_hosting");
}

pub struct FileService {
    password: String,
    file_list: Vec<PathItem>,
}

use messages::{HostReply, HostRequest};

#[async_trait]
impl Serving for FileService {
    async fn server_provide(
        &self,
        request: Request<HostRequest>,
    ) -> Result<Response<HostReply>, Status> {
        let request = request.into_inner();

        dbg!(request.clone());

        let struct_string = request.serialized_request;
        let password = request.password;

        if password == self.password {
            if let Ok(req) = serde_json::from_str::<ClientRequest>(&struct_string) {
                let request = match req {
                    ClientRequest::FileRequest(path) => ServerReply::File(ServerFile::new(path)),
                    ClientRequest::ListRequest => {
                        ServerReply::List(ServerList::new(self.file_list.clone()))
                    }
                };

                return Ok(Response::new(HostReply {
                    serialized_reply: request.serialize(),
                }));
            } else {
                let response = HostReply {
                    serialized_reply: "Invalid message? CONTACT ADMIN".to_string(),
                };

                return Ok(Response::new(response));
            }
        } else {
            let response = HostReply {
                serialized_reply: "Invalid password!".to_string(),
            };

            return Ok(Response::new(response));
        }
    }
}

fn interceptor_fn(request: Request<()>) -> Result<Request<()>, Status> {
    Ok(request)
}

async fn signal_checker(mut signal: Receiver<()>) {
    signal.recv().await;
}

pub async fn server_spawner(
    password: String,
    port: i64,
    signal: Receiver<()>,
    file_list: Vec<PathItem>,
) -> anyhow::Result<()> {
    let addr: std::net::SocketAddr = format!("[::]:{}", port).parse()?;

    let service = FileService {
        password,
        file_list,
    };

    Server::builder()
        .add_service(ServingServer::with_interceptor(service, interceptor_fn))
        .serve_with_shutdown(addr, signal_checker(signal))
        .await?;

    Ok(())
}
