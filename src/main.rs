use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    method: String,
    number: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    method: String,
    prime: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        let (mut socket, _address) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(error) => {
                        eprintln!("socket read failed: {:?}", error);
                        return;
                    }
                };

                let request: Request = match serde_json::from_slice(&buf[0..n])
                {
                    Ok(request) => dbg!(request),
                    Err(error) => {
                        eprintln!("JSON decode failed: {:?}", error);
                        return;
                    }
                };
                let response: Response = Response {
                    method: request.method,
                    prime: false,
                };

                if let Err(error) = socket
                    .write_all(
                        serde_json::to_string(&response).unwrap().as_bytes(),
                    )
                    .await
                {
                    eprintln!("socket write failed: {:?}", error);
                    return;
                }
            }
        });
    }
}
