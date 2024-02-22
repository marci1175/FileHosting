use self::messages::{
    communication_server::Communication, serving_server::Serving, serving_server::ServingServer,
    FileList, FileReply, FileRequest, Ping,
};
use std::{fs, future::Future, net::Incoming};
use tokio::sync::mpsc::{self, Receiver};
use tonic::{async_trait, service::interceptor, transport::Server, Request, Response, Status};

pub mod messages {
    tonic::include_proto!("file_hosting");
}

pub struct FileService {
    password: String,
    structure_reciver: mpsc::Receiver<String>,
}

#[async_trait]
impl Serving for FileService {
    async fn provide_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileReply>, Status> {
        let request = request.into_inner();

        let msg_password = request.password;
        return Ok(Response::new(FileReply {
            file: {
                if self.password == msg_password {
                    fs::read(request.path)?
                } else {
                    vec![]
                }
            },
        }));
    }
}

#[async_trait]
impl Communication for FileService {
    async fn provide_list(&self, request: Request<Ping>) -> Result<Response<FileList>, Status> {
        Ok(Response::new(FileList {
            list: "asd".to_string(),
        }))
    }
}

fn interceptor_fn(request: Request<()>) -> Result<Request<()>, Status> {
    
    Ok(request)
}

async fn signal(mut rx: Receiver<()>) {
    rx.recv().await;
}

pub async fn server_spawner(password: String, port: i32, rx: Receiver<()>) -> anyhow::Result<()> {
    let addr: std::net::SocketAddr = format!("[::]:{}", port).parse()?;

    let service = FileService {
        password,
        structure_reciver: mpsc::channel(42).1,
    };

    Server::builder()
        .add_service(ServingServer::with_interceptor(service, interceptor_fn))
        .serve_with_shutdown(addr, signal(rx)).await?;


    Ok(())
}
