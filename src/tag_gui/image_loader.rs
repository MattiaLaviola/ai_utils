use egui;
use egui::Vec2;
use egui_extras::image::RetainedImage;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;

use std::thread;
use std::thread::JoinHandle;

#[allow(dead_code)]
enum BufferCommand {
    LoadNext,
    LoadPrevious,
    Stop,
    Save(String, String),
}

enum BufferResult {
    None,
    Next(CaptionedImg),
    Previous(CaptionedImg),
}

impl BufferResult {
    fn unwrap(self) -> CaptionedImg {
        match self {
            BufferResult::Next(img) => img,
            BufferResult::Previous(img) => img,
            BufferResult::None => panic!("BufferResult::None"),
        }
    }

    fn is_none(&self) -> bool {
        matches!(*self, BufferResult::None)
    }

    fn is_next(&self) -> bool {
        matches!(*self, BufferResult::Next(_))
    }

    fn is_previous(&self) -> bool {
        matches!(*self, BufferResult::Previous(_))
    }
}

//#[derive(Clone)]
pub struct CaptionedImg {
    name: String,
    pub caption: String,
    img: Vec<u8>,
    //Retained images are not clonable
    cache: RetainedImage,
    wrong_size: bool,
}

impl CaptionedImg {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn caption(&self) -> String {
        self.caption.clone()
    }
    pub fn img(&self) -> Vec<u8> {
        self.img.clone()
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.cache.show_size(ui, Vec2::from((512.0, 512.0)));
    }

    pub fn is_wrong_size(&self) -> bool {
        self.wrong_size
    }

    pub fn new(name: &str, caption: &str, img: &[u8]) -> Option<Self> {
        // TODO fix program crashing when the image is not loading
        let cache = egui_extras::RetainedImage::from_image_bytes(name, img);
        if cache.is_err() {
            return None;
        }
        let cache = cache.unwrap();
        let mut w_size = false;
        if cache.width() != 512 || cache.height() != 512 {
            println!(
                "Imge {} has wrong size w:{} h:{}",
                &name,
                cache.width(),
                cache.height()
            );
            w_size = true;
        }
        Some(Self {
            name: name.to_string(),
            caption: caption.to_string(),
            img: img.to_vec(),
            cache,
            wrong_size: w_size,
        })
    }
}

impl Clone for CaptionedImg {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            caption: self.caption.clone(),
            img: self.img.clone(),
            cache: egui_extras::RetainedImage::from_image_bytes(&self.name, &self.img).unwrap(),
            wrong_size: self.wrong_size,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.name = source.name();
        self.caption = source.caption();
        self.img = source.img();
        self.cache = egui_extras::RetainedImage::from_image_bytes(&self.name, &self.img).unwrap();
        self.wrong_size = source.is_wrong_size();
    }
}

struct WorkerThreadData {
    t_files: Vec<String>,
    t_dir: String,
    send_channel: mpsc::Sender<BufferResult>,
    recv_channel: mpsc::Receiver<BufferCommand>,
}

// This struct is used as a buffer for preloading the images,to speed up the loading
pub struct ImageLoader {
    _thread_handle: std::thread::JoinHandle<()>,
    send_channel: std::sync::mpsc::Sender<BufferCommand>,
    recv_channel: std::sync::mpsc::Receiver<BufferResult>,
    // TODO: Implement buffer to allow faster scrolling
    // buffer: Vec<CaptionedImg>,
    // buffer_pos: usize,
    //buffer_size: usize,
}

impl ImageLoader {
    pub fn new(root_dir: String, file_list: Vec<String>) -> Self {
        // Maybe change this behavour in the future, at the moment is useful for testing
        if file_list.is_empty() {
            panic!("File list empty");
        }

        let (to_thread, recv_thread) = mpsc::channel();
        let (to_gui, recv_gui) = mpsc::channel();

        let thread_data = WorkerThreadData {
            t_files: file_list,
            t_dir: root_dir,
            send_channel: to_gui,
            recv_channel: recv_thread,
        };

        let thread_handle = ImageLoader::start_thread(thread_data);

        Self {
            send_channel: to_thread,
            recv_channel: recv_gui,
            _thread_handle: thread_handle,
        }
    }

    pub fn get_next(&mut self) -> Option<CaptionedImg> {
        self.get_img(true)
    }

    pub fn get_previous(&mut self) -> Option<CaptionedImg> {
        self.get_img(false)
    }

    pub fn save(&mut self, img: &CaptionedImg) {
        self.send_channel
            .send(BufferCommand::Save(img.name(), img.caption()))
            .unwrap();
    }

    pub fn save_caption(&mut self, name: &str, caption: &str) {
        self.send_channel
            .send(BufferCommand::Save(name.to_string(), caption.to_string()))
            .unwrap();
    }

    fn get_img(&mut self, forward: bool) -> Option<CaptionedImg> {
        let gen_request = || {
            if forward {
                BufferCommand::LoadNext
            } else {
                BufferCommand::LoadPrevious
            }
        };

        let is_correct = |img: &BufferResult| {
            if forward {
                img.is_next()
            } else {
                img.is_previous()
            }
        };

        // We ask the thread for the next image, it should already be in ram by now
        self.send_channel.send(gen_request()).unwrap();

        let img = self.recv_channel.recv().expect("Worker thread closed");

        // Just to be sure
        if !is_correct(&img) {
            println!("Image loader got wrong image");
        }

        if img.is_none() {
            None
        } else {
            Some(img.unwrap())
        }
    }

    fn start_thread(data: WorkerThreadData) -> JoinHandle<()> {
        thread::spawn(move || {
            let data = data;
            // Data unwarap----------------------
            let t_dir = data.t_dir;
            let mut t_files = data.t_files;
            let to_gui = data.send_channel;
            let recv_channel = data.recv_channel;
            //-----------------------------------

            let mut pos = 0;
            const FORWARD: bool = false;
            const BACKWARD: bool = true;
            let mut loading_direction = FORWARD;
            let mut next_img = ImageLoader::load_valid_image(&t_dir, &mut t_files, pos, false);
            let mut second_img = true;

            //Main loop
            loop {
                let command = recv_channel.recv().expect("Main thread shut down");
                match command {
                    BufferCommand::LoadNext => {
                        if t_files.is_empty() {
                            to_gui
                                .send(BufferResult::None)
                                .expect("Main therad shut down");
                            continue;
                        }
                        
                        // This is needed because when the thread has just started and is executing ths code for the first time
                        // if now we do pos +=1 (pos = 1) and the we load pos+1 (pos = 2) we are skipping the pos=1 image
                        // the solutions are either init pos to -1 and make it a signed int on this
                        if !second_img {
                            pos += 1;
                        } else {
                            second_img = false;
                        }

                        // If the next pos is out of bounds we return None
                        if pos >= t_files.len() {
                            pos -= 1;
                            to_gui
                                .send(BufferResult::None)
                                .expect("Main therad shut down");
                            continue;
                        }

                        
                        // If the previous command was also a LoadNext then we already have the next image loaded
                        let to_send = if loading_direction == FORWARD {
                            next_img
                        } else {
                            // If the previous command was LoadPrevious we need to load the image from disk
                            // because inside next_image there is the image before the one beeing shown now
                            ImageLoader::load_valid_image(&t_dir, &mut t_files, pos, FORWARD)
                        };

                        to_gui
                            .send(BufferResult::Next(to_send))
                            .expect("Main therad shut down");

                        // We alreay served the image to the user and now we stat to preload the next image to cut down on loading time
                        if pos + 1 < t_files.len() {
                            next_img =
                                ImageLoader::load_valid_image(&t_dir, &mut t_files, pos + 1, false);
                        }else{
                            // In theory wher pos -1 == t_files.len() we just start to send None as a response
                            // so the image that we are loading now should never be used, but if we do not put anything in next_img
                            // the compiler gets mad so we do this just to make sure that next_img does not point to memory
                            // that belongs by now to some other variable
                            next_img =
                            ImageLoader::load_valid_image(&t_dir, &mut t_files, pos, false);
                        }

                        loading_direction = FORWARD;
                    }

                    BufferCommand::LoadPrevious => {
                        // If the next pos is out of bounds we return None
                        if pos == 0 {
                            to_gui
                                .send(BufferResult::None)
                                .expect("Main therad shut down");
                            continue;
                        }

                        if t_files.is_empty() {
                            to_gui
                                .send(BufferResult::None)
                                .expect("Main therad shut down");
                            continue;
                        }

                        pos -= 1;

                        /*
                        // the ownership of next_img is going to be transfered, so if needed we clone it here
                        let next_next_img = if pos > 0 {
                            ImageLoader::load_valid_image(&t_dir, &mut t_files, pos - 1, false)
                        } else {
                            next_img.clone()
                        };
                        */

                        let to_send = if loading_direction == BACKWARD {
                            next_img
                        } else {
                            ImageLoader::load_valid_image(&t_dir, &mut t_files, pos, BACKWARD)
                        };

                        to_gui
                            .send(BufferResult::Previous(to_send))
                            .expect("Main therad shut down");

                        if pos -1 > 0{
                            next_img = ImageLoader::load_valid_image(&t_dir, &mut t_files, pos - 1, false);
                        }else{
                            next_img = ImageLoader::load_valid_image(&t_dir, &mut t_files, pos, false);
                        }

                        loading_direction = BACKWARD;
                    }

                    BufferCommand::Save(file, tags) => {
                        ImageLoader::save_image(&file, &tags, &t_dir);
                    }

                    BufferCommand::Stop => {
                        return;
                    }
                }
            }
        })
    }

    fn try_load_image(root_dir: &String, file_name: &String) -> Option<CaptionedImg> {
        if root_dir.is_empty() || file_name.is_empty() {
            println!(
                "Error trying to load image\nDir: {}\nFile: {}",
                root_dir, file_name
            );
            return None;
        }

        let base = if root_dir.ends_with('\\') {
            root_dir.to_owned() + file_name
        } else {
            root_dir.to_owned() + "\\" + file_name
        };

        let img_path = base.clone() + ".png";
        let tags_path = base + ".txt";

        let mut buffer = vec![];
        if let Ok(mut file) = File::open(img_path) {
            if let Err(e) = file.read_to_end(&mut buffer) {
                println!("Error reading file: {}", e);
                return None;
            }
        }

        let read_op = fs::read_to_string(tags_path);
        let caption = if let Ok(caption) = read_op {
            caption
        } else {
            String::new()
        };
        CaptionedImg::new(file_name, &caption, &buffer)
    }

    // This function returns an image if a valid one is found, otherwise it returns None
    // also, invalid images are removed from the list
    fn load_valid_image(
        root_dir: &String,
        files: &mut Vec<String>,
        starting_pos: usize,
        load_previous: bool,
    ) -> CaptionedImg {
        let mut pos = starting_pos;
        let mut img = ImageLoader::try_load_image(root_dir, &files[pos]);
        while img.is_none() {
            files.remove(pos);
            if files.is_empty() {
                return ImageLoader::get_std_img();
            }

            if load_previous && pos > 0 {
                pos -= 1;
            }

            img = ImageLoader::try_load_image(root_dir, &files[pos]);
        }
        img.unwrap()
    }

    fn save_image(file_name: &str, caption: &str, root_dir: &str) {
        let tags_path = root_dir.to_owned() + "\\" + file_name + ".txt";

        let file = File::create(tags_path);
        if file.is_err() {
            println!("Impossible to save");
            return;
        }

        if write!(file.unwrap(), "{}", caption).is_err() {
            println!("Error saving file");
        }
    }

    pub fn get_std_img() -> CaptionedImg {
        let bytes = include_bytes!("../../assets/no_img.png");
        CaptionedImg::new("no image", ".\\", bytes).unwrap()
    }
}
