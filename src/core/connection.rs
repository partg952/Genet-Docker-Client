use std::borrow::Cow;
use std::error::Error;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
pub struct Socket {
    pub socket_connection: UnixStream,
}
impl Socket {
    pub fn connect_to_socket() -> Result<Socket, Box<dyn Error>> {
        let mut socket_connection = UnixStream::connect("/var/run/docker.sock")?;
        Ok(Socket { socket_connection })
    }

    pub fn write_request(&mut self, request: &[u8]) -> Result<(), Box<dyn Error>> {
        self.socket_connection.write_all(request)?;
        Ok(())
    }

    pub fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut response_string = String::new();
        self.socket_connection
            .read_to_string(&mut response_string)?;
        Ok(response_string)
    }
    //for reading container logs
    pub fn read_response_utf8(&mut self) -> Result<String, Box<dyn Error>> {
        let mut utf8_buffer = Vec::new();
        self.socket_connection.read_to_end(&mut utf8_buffer)?;
        let mut response_string = String::from_utf8_lossy(&utf8_buffer).to_string();
        Ok(response_string)
    }
}
