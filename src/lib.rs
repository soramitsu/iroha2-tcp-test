#[cfg(test)]
mod tests {

    use async_std::io;
    use async_std::net::TcpListener;
    use async_std::net::TcpStream;
    use async_std::prelude::*;

    use parity_scale_codec::{Decode, Encode};

    #[derive(PartialEq, Clone, Debug, Encode, Decode)]
    pub enum Transaction {
        Requested {
            instructions: Vec<String>,
            creation_time: u128,
            account_id: (String, String),
            signatures: Vec<String>,
        },
    }

    #[async_std::test]
    async fn highload() {
        let _request = vec![
            0, 4, 5, 32, 97, 99, 99, 111, 117, 110, 116, 49, 24, 100, 111, 109, 97, 105, 110, 32,
            97, 99, 99, 111, 117, 110, 116, 50, 24, 100, 111, 109, 97, 105, 110, 12, 120, 111, 114,
            24, 100, 111, 109, 97, 105, 110, 44, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111,
            110, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 248, 173, 149, 113, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 28, 97, 99, 99, 111, 117, 110, 116, 24, 100, 111, 109, 97, 105,
            110, 0,
        ];
        //let request_in_bytes = request.as_slice();
        std::thread::spawn(|| futures::executor::block_on(start_server()));
        std::thread::sleep(std::time::Duration::from_millis(1000));
        use futures::executor::ThreadPool;
        let pool = ThreadPool::new().unwrap();
        for _ in 0..1000 {
            pool.spawn_ok(async move {
                let transaction = Transaction::Requested {
                    instructions: vec!["1".to_string(), "22".to_string(), "333".to_string()],
                    creation_time: std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .expect("Failed to get System Time.")
                        .as_nanos(),
                    account_id: ("account".to_string(), "domain".to_string()),
                    signatures: vec!["1231312313".to_string(), "23912839123981".to_string()],
                };
                let tx = transaction.encode();
                let request_in_bytes = tx.as_slice();
                let mut stream = TcpStream::connect("127.0.0.1:8084")
                    .await
                    .expect("Failed to connect to server.");
                stream
                    .write_all(request_in_bytes)
                    .await
                    .expect("Failed to write all.");
                let mut buf = vec![0u8; 1024];
                let _n = stream.read(&mut buf).await.expect("Failed to read result");
                assert!(buf.starts_with(request_in_bytes));
                let mut buffer: &[u8] = buf.as_mut_slice();
                assert_eq!(
                    transaction.clone(),
                    Transaction::decode(&mut buffer).unwrap()
                );
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(20000));
    }

    async fn start_server() {
        let listener = TcpListener::bind("127.0.0.1:8084")
            .await
            .expect("Failed to bind server.");
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream.expect("Failed to open stream.");
            let (reader, writer) = &mut (&stream, &stream);
            io::copy(reader, writer)
                .await
                .expect("Failed to copy stream.");
        }
    }
}
