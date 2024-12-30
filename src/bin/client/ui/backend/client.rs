use messages::HostRequest;



use tokio::sync::mpsc::{Receiver, Sender};


pub mod messages {
    tonic::include_proto!("file_hosting");
}

use CommonDefinitions::ClientRequest;

//We use the reciver to get what the main thread wants to recive, we use the sender to send back the response from the server

pub async fn connect(
    ip: String,
    password: String,
    main_sx: Sender<String>,

    //We add an option to client_request therefor we can shut down gracefully, when we ask for a None
    mut this_rx: Receiver<Option<ClientRequest>>,
) -> anyhow::Result<()> {
    let mut client =
        messages::serving_client::ServingClient::connect(format!("http://{}", ip)).await?;

    let list = client
        .server_provide(HostRequest {
            serialized_request: { ClientRequest::ListRequest.serialize() },
            password: password.clone(),
        })
        .await?
        .into_inner()
        .serialized_reply;

    //Send back the list, this wont be mutable this is a constant
    main_sx.send(list).await?;

    //download requests here
    loop {
        //Block until main asks us for something, unwrap is called cuz of the lib
        let main_need = this_rx.recv().await;

        if let Some(main_need) = main_need {
            //if the main thread asked us for a None we exit
            if let Some(need) = main_need {
                //Send whatever we get back to the main thread
                main_sx
                    .send(
                        client
                            .server_provide(HostRequest {
                                serialized_request: need.serialize(),
                                password: password.clone(),
                            })
                            .await?
                            .into_inner()
                            .serialized_reply,
                    )
                    .await?;
            } else {
                break;
            }
        }
    }

    Ok(())
}
