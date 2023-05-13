use directories::UserDirs;
use rfd::FileDialog;
use std::env;
use std::fs;
use std::fs::DirEntry;
use colored::Colorize;
use std::fs::File;
use std::io::Write;

mod tag_gui;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: rn <path> -> Renames all the files in the folder to the folder\n gui <path> -> Starts the GUI for tagging");
    }

    let command = &args[1];

    if command == "rn" {
        let file_path = &args[2];
        let name = file_path.split('\\').last().unwrap();
        println!("{} to {}", name, file_path);
        rename_file(file_path, name);
        return;
    }

    if command == "gui" {
        if args.len() < 3 {
            println!("Select file folder");
            let file_path = FileDialog::new()
                .set_directory(
                    UserDirs::new()
                        .unwrap()
                        .desktop_dir()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
                .pick_folder();
            if file_path.is_none() {
                println!("No folder selected");
                return;
            }
            start_tagging_gui(file_path.unwrap().to_str().unwrap());
        } else {
            let file_path = &args[2];
            start_tagging_gui(file_path);
        }
    }

    if command == "sub" {
        if args.len() < 4 {
            println!("{}","Usage: sub <path> <string to replace> <string to replace with>\nThis command will modify the tags file".yellow());
            return;
        }
        let path = &args[2];
        let old = &args[3];
        let new = &args[4];

        let filter = |filename: &str| {
            let ext = filename.split('.').last();
            if ext.is_none() {
                    return false;
            }else{
                return ext.unwrap() == "txt";
            }
        };

        let files = get_files_in_folder(path, Some(&filter));

        substitute(&files, old, new);
    }
}

fn rename_file(path: &str, new_name: &str) {
    for (cnt, file) in get_files_in_folder(path, None).into_iter().enumerate() {
        let f_path = file.path().display().to_string();
        let ext = if let Some(ext) = file.file_name().into_string().unwrap().split('.').last() {
            ext.to_string()
        } else {
            String::new()
        };

        let new_path = path.to_string() + "\\" + new_name + " (" + &cnt.to_string() + ")." + &ext;

        println!("OLD: {}\nNEW: {}\n", f_path.red(), new_path.green());
        if fs::rename(f_path, new_path).is_err() {
            println!("Failde to execute previous rename");
        }
    }
}

fn start_tagging_gui(path: &str) {
    // env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(840.0, 720.0)),
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

fn get_files_in_folder(path: &str, filter: Option<&dyn Fn(&str) -> bool>) -> Vec<DirEntry> {
    let files = fs::read_dir(path);
    if files.is_err() {
        println!("Failed to read directory");
        return Vec::new();
    }
    let files = files.unwrap();
    let mut good_files = Vec::new();

    if let Some(filter) = filter {
        for file in files {
            if let Ok(file) = file {
                if filter(file.path().to_str().unwrap()) {
                    good_files.push(file);
                }
            }
        }
    } else {
        for file in files {
            if let Ok(file) = file {
                good_files.push(file);
            }
        }
    }

    // if we do not save the state of the directory, the same file will be renamed multiple times
    good_files
}

fn substitute(files: &Vec<DirEntry>, old: &str, new: &str){
    for file in files {
        let caption = fs::read_to_string(file.path());
        if caption.is_err() {
            continue;
        }
        let caption = caption.unwrap();

        let new_caption = caption.replace(old, new);

        let file = File::create(file.path());
        if file.is_err() {
            println!("Impossible to save");
            continue;
        }

        if write!(file.unwrap(), "{}", new_caption).is_err() {
            println!("Error saving file");
        }

}
}
