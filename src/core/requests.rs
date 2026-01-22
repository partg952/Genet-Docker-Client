use std::error::Error;

use crate::core::connection;
use crate::core::parser;
fn generate_raw_http_request(path: String) -> Vec<u8> {
    let request = format!(
        "{} HTTP/1.0\r\nHost:localhost\r\nConnection:close\r\n\r\n",
        path
    );
    return request.into_bytes();
}

pub fn get_containers() -> Result<Vec<parser::ContainerInfo>, Box<dyn Error>> {
    let get_containers_request =
        generate_raw_http_request("GET /containers/json?all=1".to_string());
    let mut socket_instance = connection::Socket::connect_to_socket()?;
    socket_instance.write_request(&get_containers_request)?;

    let mut response = socket_instance.read_response()?;
    let parsed_array = parser::parse_array(response)?;

    Ok(parsed_array)
}

pub fn start_container(id: &String) -> Result<(), Box<dyn Error>> {
    let request_path = format!("POST /containers/{}/start", id);
    let start_container_request = generate_raw_http_request(request_path);
    let mut socket_instance = connection::Socket::connect_to_socket()?;
    socket_instance.write_request(&start_container_request)?;
    let response = socket_instance.read_response()?;
    Ok(())
}
pub fn stop_container(id: &String) -> Result<(), Box<dyn Error>> {
    let request_path = format!("POST /containers/{}/stop", id);
    let stop_container_request = generate_raw_http_request(request_path);
    let mut socket_instance = connection::Socket::connect_to_socket()?;
    socket_instance.write_request(&stop_container_request)?;
    Ok(())
}
pub fn restart_container(id: &String) -> Result<(), Box<dyn Error>> {
    stop_container(id)?;
    start_container(id)?;
    Ok(())
}
pub fn get_logs(id: &String) -> Result<(), Box<dyn Error>> {
    let request_path = format!("GET /containers/{}/logs?stdout=true&stderr=true", id);
    let logs_container_request = generate_raw_http_request(request_path);
    let mut socket_instance = connection::Socket::connect_to_socket()?;
    socket_instance.write_request(&logs_container_request)?;
    let response = socket_instance.read_response_utf8()?;

    Ok(())
}
