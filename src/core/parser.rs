use eframe::egui::Event;
use serde::Deserialize;
use serde_json;
use std::error::Error;
use struct_iterable::Iterable;
#[derive(Debug, Deserialize, Iterable)]
pub struct ContainerInfo {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Command")]
    pub command: String,
    #[serde(rename = "Names")]
    pub names: Vec<String>,
    #[serde(rename = "Created")]
    pub created: u64,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "State")]
    pub state: String,
}
#[derive(Debug, Deserialize)]
pub struct Events {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub id: String,
}
pub fn parse_event(event_string: String) -> Result<Events, Box<dyn Error>> {
    let events_json: Events = serde_json::from_str(&event_string)?;
    Ok(events_json)
}

fn remove_response_headers(response: String) -> String {
    let response_split: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();
    if response_split.len() > 1 {
        return response_split[1].to_string();
    }
    return response;
}

pub fn parse_array(response: String) -> Result<Vec<ContainerInfo>, Box<dyn Error>> {
    let response_body = remove_response_headers(response);
    let response_array: Vec<serde_json::Value> = serde_json::from_str(&response_body)?;
    let parsed_array: Vec<ContainerInfo> = response_array
        .iter()
        .map(|item| {
            let parsed_item: ContainerInfo = serde_json::from_value(item.clone()).unwrap();
            parsed_item
        })
        .collect();
    Ok(parsed_array)
}
