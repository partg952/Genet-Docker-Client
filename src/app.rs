use std::sync::mpsc;

use crate::core::events::DockerEvents;
use crate::core::parser::ContainerInfo;
use crate::core::requests;
use eframe::egui::panel::Side;
use eframe::egui::{self, Id, Ui, vec2};

pub struct DockerApp {
    containers: Vec<ContainerInfo>,
    selected_container: Option<String>,
    rx: mpsc::Receiver<DockerEvents>,
}

impl DockerApp {
    pub fn new(_cc: &eframe::CreationContext, rx: mpsc::Receiver<DockerEvents>) -> Self {
        let containers_list = requests::get_containers().unwrap();
        Self {
            containers: containers_list,
            selected_container: None,
            rx,
        }
    }
    fn filter_with_status_and_render(&mut self, status: &str, ui: &mut Ui) {
        self.containers
            .iter()
            .filter(|container| container.state == status)
            .for_each(|container| {
                let is_selected = self.selected_container == Some(container.id.clone());
                let text = if is_selected {
                    egui::RichText::new(container.names[0][1..].to_string())
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::WHITE)
                } else {
                    egui::RichText::new(container.names[0][1..].to_string()).size(16.0)
                };

                let button_style = if is_selected {
                    egui::Button::new(text).fill(egui::Color32::from_rgb(70, 130, 180))
                } else {
                    egui::Button::new(text)
                };

                let container_button = ui.add(
                    button_style
                        .wrap_mode(egui::TextWrapMode::Truncate)
                        .min_size(vec2(200.0, 0.0)),
                );
                ui.add_space(8.0);

                if container_button.hovered() {
                    container_button.show_tooltip_text(container.names[0][1..].to_string());
                }
                if container_button.clicked() {
                    self.selected_container = Some(container.id.clone());
                }
            });
    }
    fn render_row_in_container_info(&self, ui: &mut Ui, field_name: &str, field_value: &str) {
        ui.label(
            egui::RichText::new(field_name)
                .color(egui::Color32::LIGHT_GRAY)
                .size(13.0),
        );
        ui.label(egui::RichText::new(field_value).strong().size(13.0));
        ui.end_row();
    }
}

impl eframe::App for DockerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.rx.try_recv() {
            println!("{:?}", event);
            match event {
                DockerEvents::StartContainer(details) => {
                    let targeted_container_option = self
                        .containers
                        .iter_mut()
                        .find(|container| container.id == details.container_id);
                    if let Some(targeted_container) = targeted_container_option {
                        targeted_container.state = "running".to_string()
                    }
                }
                DockerEvents::StopContainer(details) => {
                    let targeted_container_option = self
                        .containers
                        .iter_mut()
                        .find(|container| container.id == details.container_id);
                    if let Some(targeted_container) = targeted_container_option {
                        targeted_container.state = "exited".to_string()
                    }
                }
                _ => {}
            }
        }
        egui::SidePanel::new(Side::Left, Id::new("containers_list"))
            .exact_width(280.0)
            .resizable(false)
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("📦 Containers ({})", self.containers.len()))
                        .strong()
                        .size(18.0),
                ));
                ui.separator();
                ui.add_space(12.0);

                let running_count = self
                    .containers
                    .iter()
                    .filter(|c| c.state == "running")
                    .count();
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("▶ Running ({})", running_count))
                        .size(13.0)
                        .strong(),
                ));
                ui.add_space(8.0);
                self.filter_with_status_and_render("running", ui);
                ui.separator();
                ui.add_space(12.0);

                let exited_count = self
                    .containers
                    .iter()
                    .filter(|c| c.state == "exited")
                    .count();
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("⏹ Exited ({})", exited_count))
                        .size(13.0)
                        .strong(),
                ));
                ui.add_space(8.0);
                self.filter_with_status_and_render("exited", ui);
            });

        if self.selected_container != None {
            egui::SidePanel::new(Side::Right, "container_info")
                .min_width(300.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    let container_info_option = self
                        .containers
                        .iter()
                        .find(|item| Some(item.id.clone()) == self.selected_container);

                    ui.heading(egui::RichText::new("📦 Container Details").size(20.0));
                    ui.separator();
                    ui.add_space(12.0);

                    if let Some(container_info) = container_info_option {
                        let container_id = container_info.id.clone();

                        ui.vertical_centered(|ui| {
                            let container_name = &container_info.names[0][1..];

                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(container_name)
                                        .size(20.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                )
                                .wrap(),
                            );
                        });

                        ui.add_space(16.0);

                        ui.add(egui::Label::new(
                            egui::RichText::new("Information")
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::LIGHT_YELLOW),
                        ));
                        ui.add_space(10.0);

                        egui::Grid::new("container_info_grid")
                            .num_columns(2)
                            .spacing([12.0, 12.0])
                            .min_col_width(80.0)
                            .show(ui, |ui| {
                                self.render_row_in_container_info(
                                    ui,
                                    "Image",
                                    &container_info.image,
                                );
                                self.render_row_in_container_info(
                                    ui,
                                    "State",
                                    &container_info.state,
                                );

                                let short_id = &container_info.id[0..12];
                                self.render_row_in_container_info(ui, "ID", short_id);

                                self.render_row_in_container_info(
                                    ui,
                                    "Command",
                                    &container_info.command,
                                );
                            });

                        ui.add_space(18.0);
                        ui.separator();
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.horizontal(|ui| {
                                let start_button = ui.add(
                                    egui::Button::new(egui::RichText::new("Start").strong())
                                        .min_size(vec2(90.0, 35.0))
                                        .fill(egui::Color32::from_rgb(50, 150, 80)),
                                );
                                let stop_button = ui.add(
                                    egui::Button::new(egui::RichText::new("Stop").strong())
                                        .min_size(vec2(90.0, 35.0))
                                        .fill(egui::Color32::from_rgb(180, 70, 70)),
                                );
                                if start_button.clicked() {
                                    if let Some(selected_container_info) = &self.selected_container
                                    {
                                        requests::start_container(&selected_container_info)
                                            .unwrap();
                                    }
                                }
                                if stop_button.clicked() {
                                    if let Some(selected_container_info) = &self.selected_container
                                    {
                                        requests::stop_container(&selected_container_info).unwrap();
                                    }
                                }
                            });
                        });

                        ui.add_space(12.0);

                        let container_id_clone = container_id.clone();
                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("📋 Copy ID")).clicked() {
                                ui.ctx().copy_text(container_id_clone.clone());
                            }
                            if ui.button(egui::RichText::new("🔄 Refresh")).clicked() {
                                self.containers = requests::get_containers().unwrap();
                            }
                        });
                    }
                });
        }
    }
}
