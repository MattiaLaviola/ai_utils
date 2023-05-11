use eframe::egui;

use std::fs;

pub mod image_loader;
use image_loader::CaptionedImg;

pub struct TagGui {
    img_loader: image_loader::ImageLoader,
    current_image: CaptionedImg,
    persistent_txt: String,
    desired_rows: usize,
    loaded_first_img: bool,
    can_open_warinig: bool,
}

impl TagGui {
    pub fn new(path: &str) -> Self {
        let mut container = Vec::new();
        TagGui::setup_file_list(&mut container, path);
        if container.is_empty() {
            panic!("No files found in directory");
        }
        let bytes = include_bytes!("../assets/no_img.png");

        TagGui {
            img_loader: image_loader::ImageLoader::new(path.to_string(), container),
            current_image: image_loader::CaptionedImg::new("no image", ".\\", bytes),
            persistent_txt: String::new(),
            desired_rows: 35,
            loaded_first_img: false,
            can_open_warinig: true,
        }
    }

    fn setup_file_list(container: &mut Vec<String>, dir: &str) {
        println!("Note: only files ending in .png are supported");

        let files = fs::read_dir(dir).unwrap();

        for file in files {
            let file = file.unwrap();

            // In the directory we expect 2 files, an image, and a txt file containing the tags
            if file.file_name().to_str().unwrap().ends_with(".png") {
                let name = file.file_name().to_str().unwrap().to_string();

                // We remove the .png extension
                let name = name[..name.len() - 4].to_string();
                container.push(name);
            }
        }
    }
}

impl eframe::App for TagGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tagging Tool");

            if !self.loaded_first_img {
                self.loaded_first_img = true;
                let img = self.img_loader.get_next();
                if img.is_none() {
                    panic!("No valid images found images");
                }
                self.current_image = img.unwrap();
            }

            if self.current_image.is_wrong_size() && self.can_open_warinig {
                egui::Window::new("My Window").show(ctx, |ui| {
                    ui.label("This image is the not the right size");
                    if ui.button("Close").clicked() {
                        self.can_open_warinig = false;
                    }
                });
            }

            ui.horizontal(|ui| {
                //Main pic
                self.current_image.show(ui);

                // If the tag is very long, I dont want the textbox take alla the space
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Tags textbox
                    let text_edit_multiline =
                        egui::TextEdit::multiline(&mut self.current_image.caption)
                            .desired_width(f32::INFINITY)
                            .desired_rows(self.desired_rows);

                    ui.add(text_edit_multiline);
                });
            });

            // Persistent textarea to store stuff
            let persistent_txt = egui::TextEdit::multiline(&mut self.persistent_txt)
                .desired_width(f32::INFINITY)
                .desired_rows(5);
            ui.add(persistent_txt);

            ui.horizontal_centered(|ui| {
                if ui.button("Previous").clicked() {
                    self.can_open_warinig = true;
                    self.img_loader.save(&self.current_image);
                    let img = self.img_loader.get_previous();
                    if let Some(img) = img {
                        self.current_image = img;
                    }
                }

                ui.label(format!(" Selected file: {}", self.current_image.name()));

                if ui.button("Next").clicked() {
                    self.can_open_warinig = true;
                    self.img_loader.save(&self.current_image);
                    let img = self.img_loader.get_next();
                    if let Some(img) = img {
                        self.current_image = img;
                    }
                }

                if ui.button("Save").clicked() {
                    self.img_loader.save(&self.current_image);
                }
            });
        });
    }
}
