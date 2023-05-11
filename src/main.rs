use std::env;
use std::fs;

mod tag_gui;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = &args[1];
    let file_path = &args[2];

    if command == "rn" {
        let name = file_path.split('\\').last().unwrap();
        println!("{} to {}", name, file_path);
        rename_file(file_path, name);
        return;
    }
    if command == "gui" {
        start_tagging_gui(file_path);
    }
}

fn rename_file(path: &str, new_name: &str) {
    let files = fs::read_dir(path).unwrap();

    // if we do not save the state of the directory, the same file will be renamed multiple times
    let files: Vec<_> = files.collect();

    for (cnt, file) in files.into_iter().enumerate() {
        let f_path = file.unwrap().path();

        let new_path = if cnt != 0 {
            path.to_string() + "\\" + new_name + " (" + &cnt.to_string() + ").jpg"
        } else {
            path.to_string() + "\\" + new_name + ".jpg"
        };
        println!("{}  {}", f_path.display(), new_path);
        if fs::rename(f_path, new_path).is_err() {
            println!("Failde to execute previous rename");
        }
    }
}

fn start_tagging_gui(path: &str) {
    // env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 720.0)),
        ..Default::default()
    };

    let gui = tag_gui::TagGui::new(path);
    eframe::run_native(
        "Dataset images tagging util",
        options,
        Box::new(|_cc| Box::<tag_gui::TagGui>::new(gui)),
    )
    .unwrap();
}
