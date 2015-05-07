use std::io::{self, Read, Write, Cursor};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use hyper::net::{NetworkStream, NetworkConnector};

use net::{self, Url};
use replayer::HttpReplayer;

struct MockConnector {
    replayer: Arc<Mutex<HttpReplayer>>,
}

impl MockConnector {
    fn new(context: &'static str) -> MockConnector {
        let replayer = HttpReplayer::new(context);
        let replayer = Arc::new(Mutex::new(replayer));

        MockConnector { replayer: replayer }
    }
}

impl NetworkConnector for MockConnector {
    type Stream = MockStream;

    fn connect(&mut self, host: &str, port: u16, scheme: &str) -> io::Result<MockStream> {
        Ok(MockStream {
            url: Url { host: host.to_string(), port: port, scheme: scheme.to_string() },
            replayer: self.replayer.clone(),

            read: None,
            write: vec![]
        })
    }
}

#[derive(Clone)]
struct MockStream {
    url: Url,
    replayer: Arc<Mutex<HttpReplayer>>,

    read: Option<Cursor<Vec<u8>>>,
    write: Vec<u8>
}

impl MockStream {
    fn load_stream(&mut self) {
        let mut replayer = self.replayer.lock().unwrap();
        let stream = replayer.load_stream(self.url.clone(), self.write.clone())
            .expect("Failed to load HTTP response").clone();

        self.read = Some(Cursor::new(stream))
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref mut read) = self.read {
            read.read(buf)
        } else {
            self.load_stream();
            self.read(buf)
        }
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write.flush()
    }
}

impl NetworkStream for MockStream {
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        Ok("127.0.0.1:1337".parse().unwrap())
    }
}

#[test]
fn test_normal_usage() {
    // TODO: This works, but it should be testing against a local server instead
    // of example.com.

    use hyper::Client;

    let connector = MockConnector::new("test");

    // Create a client.
    let mut client = Client::with_connector(connector);

    // Creating an outgoing request.
    let mut res = client.get("http://www.example.com/")
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);

    // Creating an outgoing request.
    let mut res = client.get("http://www.ulysse.io/")
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);

    panic!();
}
