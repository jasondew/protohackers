use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:7777").await?;

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

                if let Err(error) = socket.write_all(&buf[0..n]).await {
                    eprintln!("socket write failed: {:?}", error);
                    return;
                }
            }
        });
    }
}
