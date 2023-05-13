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
        // println!("{:?}", container); let bytes = include_bytes!("../assets/no_img.png");

        TagGui {
            img_loader: image_loader::ImageLoader::new(path.to_string(), container),
            current_image: image_loader::ImageLoader::get_std_img(),
            persistent_txt: String::new(),
            desired_rows: 35,
            loaded_first_img: false,
            can_open_warinig: true,
        }
    }

    fn setup_file_list(container: &mut Vec<String>, dir: &str) {
        let files = fs::read_dir(dir).unwrap();

        for file in files {
            let file = file.unwrap();

            // In the directory we expect 2 files, an image, and a txt file containing the tags
            let name = file.file_name().to_str().unwrap().to_string();
            if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") {
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
            //ui.heading("Tagging Tool");

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
                let std_button_size = egui::vec2(90.0, 30.0);

                ui.label(format!("{}", self.current_image.name()));

                let available_width = ui.available_width() - std_button_size.x * 3.0 - 30.0;

                ui.add_space(available_width);

                let button = egui::Button::new("Previous").min_size(std_button_size);
                if ui.add(button).clicked() {
                    self.can_open_warinig = true;

                    let img_name = self.current_image.name();
                    let img_caption = self.current_image.caption();

                    let img = self.img_loader.get_previous();
                    if let Some(img) = img {
                        self.current_image = img;
                    }

                    self.img_loader.save_caption(&img_name, &img_caption);
                }

                let button = egui::Button::new("Next").min_size(std_button_size);
                if ui.add(button).clicked() {
                    self.can_open_warinig = true;
                    let img = self.img_loader.get_next();

                    let img_name = self.current_image.name();
                    let img_caption = self.current_image.caption();

                    if let Some(img) = img {
                        self.current_image = img;
                    }

                    self.img_loader.save_caption(&img_name, &img_caption);
                }

                let button = egui::Button::new("Save").min_size(std_button_size);
                if ui.add(button).clicked() {
                    self.img_loader.save(&self.current_image);
                }
            });

            ui.add_space(10.0);
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
        });
    }
}
