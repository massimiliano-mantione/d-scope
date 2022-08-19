#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod errors;
mod photo_set;

use std::path::PathBuf;

use eframe::egui::{self, ImageButton};
use egui_extras::RetainedImage;
use errors::{DScopeError, DScopeResult};
use photo_set::{photo_file_name, PhotoSet};

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
                    if ui.button("Load").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.status.load = Some(path);
                        }
                    }
                });
            }
            DScopeUi::Show {
                photos,
                current_photo_index,
                current_photo,
            } => {
                egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                    if ui.button("Load").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.status.load = Some(path);
                        }
                    }
                    if ui.button("Save").clicked() {
                        if let Err(error) = photos.save() {
                            self.status.error = Some(error);
                        }
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
            }
        }
    }
}
