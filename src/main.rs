#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod errors;
mod photo_set;

use std::{f32::consts::PI, path::PathBuf};

use eframe::{
    egui::{
        self,
        plot::{Line, Plot, PlotImage, Value, Values},
        Button, ImageButton, Slider,
    },
    epaint::{Color32, Stroke},
};
use egui_extras::RetainedImage;
use errors::DScopeError;
use photo_set::{
    photo_file_name, DisplayTime, PhotoSet, MOLE_CENTER_DISTANCE_MAX, MOLE_SIZE_MAX,
    PHOTO_PX_PER_MM,
};

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

enum DScopeUi {
    Empty,
    Show {
        photos: PhotoSet,
        current_photo_index: usize,
        current_photo: RetainedImage,
        show_measures: bool,
        edit_measures: bool,
        edit_data: bool,
        save: bool,
    },
}

struct DScopeStatus {
    pub error: Option<DScopeError>,
    pub load: Option<PathBuf>,
    pub ui: DScopeUi,
}

struct MyApp {
    status: DScopeStatus,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            status: DScopeStatus {
                error: None,
                load: None,
                ui: DScopeUi::Empty,
            },
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(error) = self.status.error.take() {
            error.show();
        }
        if let Some(path) = self.status.load.take() {
            match PhotoSet::from_path(path) {
                Ok(photos) => {
                    match RetainedImage::from_image_bytes("selected-photo", &photos.photos[0].bytes)
                    {
                        Ok(current_photo) => {
                            self.status.ui = DScopeUi::Show {
                                photos,
                                current_photo_index: 0,
                                current_photo,
                                show_measures: false,
                                edit_measures: false,
                                edit_data: false,
                                save: false,
                            };
                        }
                        Err(error) => {
                            self.status.error =
                                Some(DScopeError::cannot_create_image(error, photo_file_name(0)))
                        }
                    }
                }
                Err(error) => self.status.error = Some(error),
            }
        }
        match &mut self.status.ui {
            DScopeUi::Empty => {
                egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.status.load = Some(path);
                            }
                        }
                    })
                });
            }
            DScopeUi::Show {
                photos,
                current_photo_index,
                current_photo,
                show_measures,
                edit_measures,
                edit_data,
                save,
            } => {
                if *save {
                    *save = false;
                    if let Err(error) = photos.save() {
                        self.status.error = Some(error);
                    } else {
                        *edit_measures = false;
                        *edit_data = false;
                    }
                }

                egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Load").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.status.load = Some(path);
                                }
                            }
                            if ui.button("Save").clicked() {
                                *save = true;
                            }
                            if ui.button("Save as").clicked() {
                                if let Some(new_path) = rfd::FileDialog::new().pick_folder() {
                                    let old_path = photos.path.clone();
                                    photos.path = new_path;
                                    match photos.save() {
                                        Ok(_) => {}
                                        Err(error) => {
                                            photos.path = old_path;
                                            self.status.error = Some(error);
                                        }
                                    }
                                }
                            }

                            ui.separator();

                            ui.checkbox(show_measures, "Metrics");
                            if *show_measures {
                                if ui
                                    .add_enabled(!*edit_measures, Button::new("Edit measures"))
                                    .clicked()
                                {
                                    *edit_measures = true;
                                }
                            }
                            if ui
                                .add_enabled(!*edit_data, Button::new("Edit data"))
                                .clicked()
                            {
                                *edit_data = true;
                            }

                            ui.separator();

                            let current_photo_info = &photos.photos[*current_photo_index];
                            if photos.info.surname.len() > 0 {
                                ui.label(&photos.info.surname);
                            }
                            if photos.info.name.len() > 0 {
                                ui.label(&photos.info.name);
                            }
                            ui.label(format!("[{}]", current_photo_info.id));
                            ui.label(format!(
                                "{}",
                                DisplayTime::new(current_photo_info.info.time)
                            ));
                            if let Some(size) = current_photo_info.info.mole_metrics.size() {
                                ui.label(format!("(size {} mm)", size));
                            }
                        });

                        if *edit_data || *edit_measures {
                            let current_photo_info = &mut photos.photos[*current_photo_index];

                            if *edit_data {
                                ui.label("Visit");
                                ui.horizontal(|ui| {
                                    ui.label("Surname");
                                    ui.text_edit_singleline(&mut photos.info.surname);
                                    ui.separator();
                                    ui.label("Name");
                                    ui.text_edit_singleline(&mut photos.info.name);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Notes");
                                    ui.text_edit_multiline(&mut photos.info.notes);
                                });
                                if !*edit_measures {
                                    if ui.button("Save").clicked() {
                                        *save = true;
                                    }
                                }
                            }

                            if *edit_measures {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("X");
                                    ui.add(
                                        Slider::new(
                                            &mut current_photo_info.info.mole_metrics.center_x,
                                            -MOLE_CENTER_DISTANCE_MAX..=MOLE_CENTER_DISTANCE_MAX,
                                        )
                                        .clamp_to_range(true),
                                    );
                                    ui.label("Y");
                                    ui.separator();
                                    ui.add(
                                        Slider::new(
                                            &mut current_photo_info.info.mole_metrics.center_y,
                                            -MOLE_CENTER_DISTANCE_MAX..=MOLE_CENTER_DISTANCE_MAX,
                                        )
                                        .clamp_to_range(true),
                                    );
                                    ui.label("Size");
                                    ui.separator();
                                    ui.add(
                                        Slider::new(
                                            &mut current_photo_info.info.mole_metrics.diameter,
                                            0.0..=MOLE_SIZE_MAX,
                                        )
                                        .clamp_to_range(true),
                                    );
                                    ui.separator();
                                    if ui.button("Save").clicked() {
                                        *save = true;
                                    }
                                });
                            }
                        }
                    })
                });

                egui::SidePanel::left("photo-list").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        for (index, photo) in photos.photos.iter().enumerate() {
                            let size = photo.preview.size();
                            let button = ImageButton::new(
                                photo.preview.texture_id(ctx),
                                [size[0] as f32, size[1] as f32],
                            )
                            .selected(index == *current_photo_index);
                            if ui.add(button).clicked() {
                                if *current_photo_index != index {
                                    match RetainedImage::from_image_bytes(
                                        "selected-photo",
                                        &photos.photos[index].bytes,
                                    ) {
                                        Ok(new_photo) => {
                                            *current_photo_index = index;
                                            *current_photo = new_photo;
                                        }
                                        Err(error) => {
                                            self.status.error =
                                                Some(DScopeError::cannot_create_image(
                                                    error,
                                                    photo_file_name(index),
                                                ))
                                        }
                                    }
                                }
                            }
                        }
                    });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    let unlock_movement = !*show_measures;

                    Plot::new("main-panel")
                        .data_aspect(1.0)
                        .allow_zoom(unlock_movement)
                        .allow_scroll(unlock_movement)
                        .allow_drag(unlock_movement)
                        .show_axes([false, false])
                        .show(ui, |plot| {
                            let size = current_photo.size();
                            let image = PlotImage::new(
                                current_photo.texture_id(ctx),
                                Value::new(0.0, 0.0),
                                [
                                    size[0] as f32 / PHOTO_PX_PER_MM,
                                    size[1] as f32 / PHOTO_PX_PER_MM,
                                ],
                            );
                            plot.image(image);

                            if *show_measures {
                                let current_photo_info = &mut photos.photos[*current_photo_index];

                                plot.line(
                                    Line::new(Values::from_values_iter(circle(
                                        current_photo_info.info.mole_metrics.center_x,
                                        current_photo_info.info.mole_metrics.center_y,
                                        current_photo_info.info.mole_metrics.diameter / 2.0,
                                        64,
                                    )))
                                    .stroke(Stroke::new(3.0, Color32::WHITE)),
                                );

                                if let Some(point) = plot.pointer_coordinate() {
                                    let px = point.x as f32;
                                    let py = point.y as f32;

                                    if plot.plot_clicked() {
                                        current_photo_info.info.mole_metrics.center_x = px;
                                        current_photo_info.info.mole_metrics.center_y = py;
                                    }

                                    let drag = plot.pointer_coordinate_drag_delta();
                                    if drag[0] != 0.0 || drag[1] != 0.0 {
                                        let p2_x = px;
                                        let p2_y = py;
                                        let p1_x = px - drag[0];
                                        let p1_y = py - drag[1];
                                        let cx = current_photo_info.info.mole_metrics.center_x;
                                        let cy = current_photo_info.info.mole_metrics.center_y;

                                        let r2 =
                                            ((p2_x - cx).powf(2.0) + (p2_y - cy).powf(2.0)).sqrt();
                                        let r1 =
                                            ((p1_x - cx).powf(2.0) + (p1_y - cy).powf(2.0)).sqrt();

                                        current_photo_info.info.mole_metrics.diameter +=
                                            2.0 * (r2 - r1);
                                    }
                                }
                            }
                        });
                });
            }
        }
    }
}

fn circle(x: f32, y: f32, r: f32, n: usize) -> impl Iterator<Item = Value> {
    let arc = if n == 0 { PI } else { 2.0 * PI / (n as f32) };
    (0..=n)
        .into_iter()
        .map(move |i| i as f32 * arc)
        .map(move |arc| Value {
            x: (x + (r * arc.cos())) as f64,
            y: (y + (r * arc.sin())) as f64,
        })
}
