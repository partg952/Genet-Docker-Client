mod app;
mod core;
use eframe::egui::{self, response};
use std::sync::mpsc;
use std::{
    io::{BufRead, BufReader, Read},
    thread,
};

use crate::core::events::{DockerEvents, EventDetails, convert_to_event};
use crate::core::long_connection::EventsSocket;

fn main() -> eframe::Result {
    let (tx , rx) = mpsc::channel::<DockerEvents>();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    thread::spawn(move || {
        let mut event_socket = EventsSocket::connect().unwrap();
        let mut buffer = BufReader::new(event_socket.socket_connection);
        let mut ingest_data = false;
        for line in buffer.lines() {
            let line = line.unwrap();
            if !ingest_data && line == "" {
                ingest_data = !ingest_data;
                continue;
            }
            if ingest_data {
                let event = convert_to_event(&line);
                tx.send(event);
            }
        }
    });
    eframe::run_native(
        "Docker Client",
        native_options,
        Box::new(|cc| Ok(Box::new(app::DockerApp::new(cc,rx)))),
    )?;
    Ok(())
}
