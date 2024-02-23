use self::messages::Ping;

use std::{fs, path::PathBuf, vec};
use tokio::{
    sync,
    sync::mpsc::{self, Receiver},
};
use tonic::{
    async_trait,
    transport::{Endpoint, Server},
    Request, Response, Status,
};

pub mod messages {
    tonic::include_proto!("file_hosting");
}

use CommonDefinitions::{ServerList, ServerFile};

pub async fn connect(ip: String, password: String) -> anyhow::Result<()> {
    println!("asd");

    let mut client =
        messages::serving_client::ServingClient::connect(format!("http://{}", ip)).await?;

    // let list = client
    //     .provide_list(Ping {
    //         password: password,
    //     })
    //     .await?
    //     .into_inner()
    //     .list;

    // let extracted_struct: ServerList = serde_json::from_str(&list)?;

    // dbg!(extracted_struct);

    Ok(())
}
