use self::messages::{
    communication_server::Communication, serving_server::Serving, serving_server::ServingServer,
    FileList, FileReply, FileRequest, Ping,
};
use crate::ui::app::PathItem;

use std::{fs, path::PathBuf, vec};
use tokio::{
    net::windows::named_pipe,
    sync::mpsc::{self, Receiver},
    task::futures,
};
use tonic::{async_trait, service::interceptor, transport::Server, Request, Response, Status};

pub mod messages {
    tonic::include_proto!("file_hosting");
}

pub struct FileService {
    password: String,
    file_list: Vec<PathItem>,
}

#[derive(serde::Serialize)]
pub struct ServerReply {
    file: Option<std::vec::Vec<u8>>,
    error: Option<String>,
}

impl ServerReply {
    fn new(path: PathBuf) -> Self {
        match fs::read(path) {
            Ok(bytes) => Self {
                file: Some(bytes),
                error: None,
            },
            Err(err) => Self {
                file: None,
                error: Some(err.to_string()),
            },
        }
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[async_trait]
impl Serving for FileService {
    async fn provide_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileReply>, Status> {
        let request = request.into_inner();

        let msg_password = request.password;
        if msg_password == self.password {
            return Ok(Response::new(FileReply {
                serialized_file: ServerReply::new(request.path.into()).serialize(),
            }));
        } else {
            return Ok(Response::new(FileReply {
                serialized_file: ServerReply {
                    file: None,
                    error: Some("Invalid password".to_string()),
                }
                .serialize(),
            }));
        }
    }
}

#[derive(serde::Serialize)]
pub struct ServerPing {
    file: Option<String>,
    error: Option<String>,
}

impl ServerPing {
    fn new(file_list: Vec<PathItem>) -> Self {
        match serde_json::to_string(&file_list) {
            Ok(vec_string) => Self {
                file: Some(vec_string),
                error: None,
            },
            Err(err) => Self {
                file: None,
                error: Some(err.to_string()),
            },
        }
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[async_trait]
impl Communication for FileService {
    async fn provide_list(&self, request: Request<Ping>) -> Result<Response<FileList>, Status> {
        if self.password == request.into_inner().password {
            Ok(Response::new(FileList {
                list: ServerPing::new(self.file_list.clone()).serialize(),
            }))
        } else {
            Ok(Response::new(FileList {
                list: ServerPing {
                    file: None,
                    error: None,
                }
                .serialize(),
            }))
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
