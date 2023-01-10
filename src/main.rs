use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    method: String,
    number: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Response {
    method: String,
    prime: bool,
}

fn handle_request(buf: &[u8]) -> Result<Response, String> {
    let request: Request = match serde_json::from_slice(buf) {
        Ok(request) => request,
        Err(error) => {
            return Err(format!("JSON decode failed: {}", error));
        }
    };

    if request.method != "isPrime" {
        Err(format!("unknown method: {}", request.method))
    } else {
        let response: Response = Response {
            method: request.method,
            prime: is_prime(request.number),
        };

        Ok(response)
    }
}

fn is_prime(candidate: usize) -> bool {
    if candidate < 2 {
        return false;
    }

    let mut divisor = candidate / 2;
    loop {
        if divisor == 1 {
            return true;
        }

        if candidate % divisor == 0 {
            return false;
        } else {
            divisor -= 1;
        }
    }
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

                match handle_request(&buf[0..n]) {
                    Ok(response) => {
                        if let Err(error) = socket
                            .write_all(
                                serde_json::to_string(&response)
                                    .unwrap()
                                    .as_bytes(),
                            )
                            .await
                        {
                            eprintln!("socket write failed: {:?}", error);
                            return;
                        }
                    }
                    Err(error) => {
                        eprintln!("{}", error);
                        if let Err(error) =
                            socket.write_all(error.to_string().as_bytes()).await
                        {
                            eprintln!("socket write failed: {:?}", error);
                        }
                        return;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert_eq!(is_prime(0), false);
        assert_eq!(is_prime(1), false);
        assert_eq!(is_prime(2), true);
        assert_eq!(is_prime(3), true);
        assert_eq!(is_prime(4), false);
        assert_eq!(is_prime(5), true);
        assert_eq!(is_prime(6), false);
        assert_eq!(is_prime(7), true);
        assert_eq!(is_prime(8), false);
        assert_eq!(is_prime(9), false);
        assert_eq!(is_prime(10), false);
        assert_eq!(is_prime(1024), false);
        assert_eq!(is_prime(1033), true);
    }

    #[test]
    fn test_handle_request_valid() {
        assert_eq!(
            handle_request(br#"{"method":"isPrime","number": 7}"#),
            Ok(Response {
                method: "isPrime".into(),
                prime: true
            })
        );
    }

    #[test]
    fn test_handle_request_unknown_method() {
        assert_eq!(
            handle_request(br#"{"method":"isComposite","number": 7}"#),
            Err("unknown method: isComposite".into())
        );
    }

    #[test]
    fn test_handle_request_missing_method() {
        assert_eq!(
            handle_request(br#"{"number": 7}"#),
            Err("JSON decode failed: missing field `method` at line 1 column 13".into())
        );
    }
}
