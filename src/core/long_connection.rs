use std::{error::Error, io::Write, os::unix::net::UnixStream};
pub struct EventsSocket {
    pub socket_connection : UnixStream
}

impl EventsSocket {
    pub fn connect() -> Result<EventsSocket ,Box<dyn Error>> {
        let request_string = "GET /events HTTP/1.0\r\nHost:localhost\r\nConnection:keep-alive\r\n\r\n";
        let mut socket_connection = UnixStream::connect("/var/run/docker.sock")?;
        socket_connection.write_all(request_string.as_bytes())?;

        Ok(EventsSocket { socket_connection })
    }
}