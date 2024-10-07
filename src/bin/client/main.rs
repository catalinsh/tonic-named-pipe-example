use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::time;
use tonic::transport::Endpoint;
use tower::service_fn;
use windows::Win32::Foundation::ERROR_PIPE_BUSY;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(|_| async {
            let client = loop {
                match ClientOptions::new().open(r"\\.\pipe\tonic_named_pipe_test") {
                    Ok(client) => break client,
                    Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => (),
                    Err(e) => return Err(e),
                }

                time::sleep(Duration::from_millis(50)).await;
            };

            Ok::<_, std::io::Error>(TokioIo::new(client))
        }))
        .await?;

    let mut client = GreeterClient::new(channel);

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
