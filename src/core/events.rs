use crate::core::parser::Events;
#[derive(Debug)]
pub struct EventDetails {
    pub container_id : String
}
#[derive(Debug)]
pub enum DockerEvents {
    StartContainer(EventDetails),
    StopContainer(EventDetails),
    OtherEvent
}
pub fn convert_to_event(event_string:&String) -> DockerEvents {
    let event = crate::core::parser::parse_event(event_string.to_string()).unwrap();
    match event.status.as_str() {
        "start" => {
            DockerEvents::StartContainer(EventDetails { container_id: event.id })
        },
        "stop" => {
            DockerEvents::StopContainer(EventDetails { container_id: event.id })
        },
        _=> {
            DockerEvents::OtherEvent
        }
    }
}