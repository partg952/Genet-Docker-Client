use std::{sync::mpsc, time::Duration};

use crate::core::events::DockerEvents;
use crate::core::parser::ContainerInfo;
use crate::core::requests;
use eframe::egui::panel::Side;
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Id, Layout, Margin, RichText, Stroke, Ui, vec2,
};

const SPACE_XS: f32 = 4.0;
const SPACE_SM: f32 = 8.0;
const SPACE_MD: f32 = 12.0;

pub struct DockerApp {
    containers: Vec<ContainerInfo>,
    selected_container: Option<String>,
    rx: mpsc::Receiver<DockerEvents>,
    logs: Vec<String>,
    logs_rx: Option<mpsc::Receiver<String>>,
    theme_initialized: bool,
}

impl DockerApp {
    pub fn new(_cc: &eframe::CreationContext, rx: mpsc::Receiver<DockerEvents>) -> Self {
        let containers_list = requests::get_containers().unwrap();
        Self {
            containers: containers_list,
            selected_container: None,
            rx,
            logs: Vec::new(),
            logs_rx: None,
            theme_initialized: false,
        }
    }

    fn init_theme(&mut self, ctx: &egui::Context) {
        if self.theme_initialized {
            return;
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = vec2(SPACE_SM, SPACE_SM);
        style.spacing.button_padding = vec2(10.0, 6.0);
        style.spacing.window_margin = Margin::same(SPACE_MD as i8);
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(32, 35, 41);
        style.visuals.extreme_bg_color = Color32::from_rgb(38, 42, 49);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(53, 58, 66);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(58, 63, 72);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(63, 69, 79);
        style.visuals.window_corner_radius = CornerRadius::same(8);

        ctx.set_style(style);
        self.theme_initialized = true;
    }

    fn container_name(container: &ContainerInfo) -> String {
        container
            .names
            .first()
            .map(|name| name.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "Unnamed".to_string())
    }

    fn filter_with_status_and_render(&mut self, status: &str, ui: &mut Ui) {
        let mut clicked_container = None;

        for container in self
            .containers
            .iter()
            .filter(|container| container.state == status)
        {
            let is_selected = self
                .selected_container
                .as_ref()
                .is_some_and(|id| id == &container.id);

            let text = if is_selected {
                RichText::new(Self::container_name(container))
                    .size(14.0)
                    .strong()
                    .color(Color32::from_rgb(226, 231, 240))
            } else {
                RichText::new(Self::container_name(container))
                    .size(14.0)
                    .color(Color32::from_rgb(204, 210, 223))
            };

            let button = if is_selected {
                egui::Button::new(text)
                    .fill(Color32::from_rgb(68, 76, 90))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(92, 102, 120)))
                    .corner_radius(CornerRadius::same(8))
            } else {
                egui::Button::new(text)
                    .fill(Color32::from_rgb(49, 54, 63))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(61, 67, 78)))
                    .corner_radius(CornerRadius::same(8))
            };

            let button_width = (ui.available_width() - SPACE_XS).max(180.0);
            let response = ui.add_sized(
                [button_width, 34.0],
                button.wrap_mode(egui::TextWrapMode::Truncate),
            );

            if response.hovered() {
                response
                    .clone()
                    .on_hover_text(Self::container_name(container));
            }
            if response.clicked() {
                clicked_container = Some(container.id.clone());
            }

            ui.add_space(SPACE_XS);
        }

        if let Some(container_id) = clicked_container {
            if self
                .selected_container
                .as_ref()
                .is_none_or(|selected_id| selected_id != &container_id)
            {
                self.selected_container = Some(container_id.clone());
                self.logs.clear();
                self.logs_rx = Some(requests::get_logs(container_id));
            }
        }
    }

    fn render_row_in_container_info(&self, ui: &mut Ui, field_name: &str, field_value: &str) {
        ui.add(
            egui::Label::new(
                RichText::new(field_name)
                    .color(Color32::from_rgb(149, 158, 176))
                    .size(13.0),
            )
            .wrap_mode(egui::TextWrapMode::Truncate),
        );
        ui.add(
            egui::Label::new(
                RichText::new(field_value)
                    .strong()
                    .size(13.0)
                    .color(Color32::from_rgb(218, 224, 236)),
            )
            .wrap_mode(egui::TextWrapMode::Truncate),
        );
        ui.end_row();
    }

    fn refresh_containers(&mut self) {
        self.containers = requests::get_containers().unwrap();
    }

    fn state_color(state: &str) -> Color32 {
        match state {
            "running" => Color32::from_rgb(151, 176, 159),
            "exited" => Color32::from_rgb(177, 155, 155),
            _ => Color32::from_rgb(166, 173, 188),
        }
    }

    fn render_state_badge(ui: &mut Ui, state: &str) {
        Frame::new()
            .fill(Color32::from_rgb(55, 60, 70))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(state.to_ascii_uppercase())
                        .size(11.0)
                        .strong()
                        .color(Self::state_color(state)),
                );
            });
    }
}

impl eframe::App for DockerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.init_theme(ctx);

        if let Some(logs_rx) = self.logs_rx.as_ref() {
            while let Ok(line) = logs_rx.try_recv() {
                self.logs.push(line);
                if self.logs.len() > 1000 {
                    self.logs.drain(0..100);
                }
            }
        }

        if self.logs_rx.is_some() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        while let Ok(event) = self.rx.try_recv() {
            match event {
                DockerEvents::StartContainer(details) => {
                    if let Some(targeted_container) = self
                        .containers
                        .iter_mut()
                        .find(|container| container.id == details.container_id)
                    {
                        targeted_container.state = "running".to_string();
                    }
                }
                DockerEvents::StopContainer(details) => {
                    if let Some(targeted_container) = self
                        .containers
                        .iter_mut()
                        .find(|container| container.id == details.container_id)
                    {
                        targeted_container.state = "exited".to_string();
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
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(SPACE_SM);
                        Frame::new()
                            .fill(Color32::from_rgb(40, 44, 52))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Containers")
                                            .strong()
                                            .size(18.0)
                                            .color(Color32::from_rgb(224, 230, 242)),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format!("{}", self.containers.len()))
                                                    .size(12.0)
                                                    .strong()
                                                    .color(Color32::from_rgb(189, 197, 212)),
                                            );
                                        },
                                    );
                                });
                            });

                        ui.add_space(SPACE_MD);

                        let running_count = self
                            .containers
                            .iter()
                            .filter(|c| c.state == "running")
                            .count();
                        ui.add(egui::Label::new(
                            RichText::new(format!("Running ({running_count})"))
                                .size(13.0)
                                .strong()
                                .color(Self::state_color("running")),
                        ));
                        ui.add_space(SPACE_SM);
                        self.filter_with_status_and_render("running", ui);

                        ui.add_space(SPACE_SM);
                        ui.separator();
                        ui.add_space(SPACE_MD);

                        let exited_count = self
                            .containers
                            .iter()
                            .filter(|c| c.state == "exited")
                            .count();
                        ui.add(egui::Label::new(
                            RichText::new(format!("Exited ({exited_count})"))
                                .size(13.0)
                                .strong()
                                .color(Self::state_color("exited")),
                        ));
                        ui.add_space(SPACE_SM);
                        self.filter_with_status_and_render("exited", ui);
                    });
            });

        if let Some(selected_id) = self.selected_container.as_ref() {
            let selected_id = selected_id.clone();
            let container_snapshot = self
                .containers
                .iter()
                .find(|item| item.id == selected_id)
                .map(|c| {
                    (
                        Self::container_name(c),
                        c.image.clone(),
                        c.state.clone(),
                        c.id.clone(),
                        c.command.clone(),
                    )
                });

            egui::SidePanel::new(Side::Right, "container_info")
                .min_width(300.0)
                .max_width(460.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(SPACE_SM);
                            ui.heading(RichText::new("Container Details").size(20.0));
                            ui.add_space(SPACE_XS);
                            ui.separator();
                            ui.add_space(SPACE_MD);

                            if let Some((name, image, state, id, command)) = &container_snapshot {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(name)
                                                .size(20.0)
                                                .strong()
                                                .color(Color32::from_rgb(223, 229, 240)),
                                        )
                                        .wrap(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            Self::render_state_badge(ui, state);
                                        },
                                    );
                                });

                                ui.add_space(SPACE_MD);
                                ui.label(
                                    RichText::new("Information")
                                        .strong()
                                        .size(14.0)
                                        .color(Color32::from_rgb(187, 194, 208)),
                                );
                                ui.add_space(SPACE_SM);

                                Frame::new()
                                    .fill(Color32::from_rgb(41, 45, 53))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::same(10))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                                            egui::Grid::new("container_info_grid")
                                                .num_columns(2)
                                                .spacing([12.0, 10.0])
                                                .min_col_width(96.0)
                                                .show(ui, |ui| {
                                                    self.render_row_in_container_info(
                                                        ui, "Image", image,
                                                    );
                                                    self.render_row_in_container_info(
                                                        ui, "State", state,
                                                    );
                                                    self.render_row_in_container_info(
                                                        ui,
                                                        "ID",
                                                        &id[..12],
                                                    );
                                                    self.render_row_in_container_info(
                                                        ui, "Command", command,
                                                    );
                                                });
                                        });
                                    });

                                ui.add_space(SPACE_MD);
                                ui.separator();
                                ui.add_space(SPACE_MD);

                                ui.horizontal(|ui| {
                                    let start_button = ui.add(
                                        egui::Button::new(RichText::new("Start").strong())
                                            .min_size(vec2(90.0, 34.0))
                                            .fill(Color32::from_rgb(88, 103, 95)),
                                    );
                                    let stop_button = ui.add(
                                        egui::Button::new(RichText::new("Stop").strong())
                                            .min_size(vec2(90.0, 34.0))
                                            .fill(Color32::from_rgb(112, 97, 97)),
                                    );

                                    if start_button.clicked() {
                                        requests::start_container(&selected_id).unwrap();
                                    }
                                    if stop_button.clicked() {
                                        requests::stop_container(&selected_id).unwrap();
                                    }
                                });

                                ui.add_space(SPACE_SM);

                                ui.horizontal(|ui| {
                                    if ui.button("Copy ID").clicked() {
                                        ui.ctx().copy_text(id.clone());
                                    }
                                    if ui.button("Refresh").clicked() {
                                        self.refresh_containers();
                                    }
                                });
                            }
                        });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Logs").strong().size(18.0));
            ui.add_space(SPACE_SM);

            if self.selected_container.is_none() {
                ui.label(
                    RichText::new("Select a container").color(Color32::from_rgb(166, 173, 188)),
                );
                return;
            }

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.logs {
                        ui.label(RichText::new(line).monospace().size(12.0));
                    }
                });
        });
    }
}
