
use std::{
    collections::HashMap,
    io::Stdout,
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::{self, available_parallelism},
};

use crate::render::renderer::{self};
use fs_extra::dir::get_size;
use rusty_pool::ThreadPool;
use std::fs;

use super::checker::{get_default_config_path, ConfigChecker};

#[derive(Clone, Debug)]
pub enum SelectStatus {
    Live,
    Deleting,
    Deleted,
    System,
    Searched,
    Calculating,
}
#[derive(Clone, Debug)]
pub struct SelectOptions {
    pub path: String,
    pub prefix: String,
    pub end: String,
    pub status: SelectStatus,
}

impl SelectOptions {
    pub fn new(path: String, prefix: String, end: String, status: SelectStatus) -> Self {
        SelectOptions { path, prefix, end, status }
    }
}
pub struct Selector {
    value: usize,
    pub options: Arc<Mutex<Vec<SelectOptions>>>,
    pub ex: HashMap<usize, usize>,
    pub need_refresh: Arc<AtomicBool>,
    pub remove_file_pool: ThreadPool,
    pub search_file_pool: ThreadPool,
}

impl Selector {
    pub fn new(value: usize) -> Self {
        Selector { value, options: Arc::new(Mutex::new(Vec::<SelectOptions>::new())), ex: HashMap::new(), need_refresh: Arc::new(AtomicBool::new(false)),  remove_file_pool: ThreadPool::default(), search_file_pool: ThreadPool::default() }
    }
    pub fn init(&mut self, path: std::path::PathBuf) -> bool {
        if self.search(path) == false {
            return false;
        };
        return true;
    }

    pub fn search(&mut self, path: std::path::PathBuf) -> bool {
        let cl: ConfigChecker;
        if let Some(_cl) = ConfigChecker::load_init() {
            cl = _cl
        } else {
            return false;
        };
        let config_loader = Arc::new(cl);
        let pools = Arc::new(self.search_file_pool.clone());
        let nr = self.need_refresh.clone();
        search_files(path.to_str().unwrap(), config_loader, self.options.clone(), pools, nr);
        return true;
    }
    pub fn add_exit(&mut self) {
        let mut guard = self.options.lock().unwrap();
        self.ex.insert(guard.len(), usize::MAX);
        self.ex.insert(usize::MAX, usize::MAX);
        guard.push(SelectOptions::new("Exit".to_owned(), "".to_owned(), "".to_owned(), SelectStatus::System));
    }
    pub fn value(&self) -> usize {
        self.value
    }
    pub fn render(&mut self, stdout: &mut Stdout) -> usize {
        let (res, operation) = renderer::selector(stdout, self.options.clone(), &mut self.value, self.need_refresh.clone());
        match operation {
            RendererOperation::SYSTEM => {
                if self.ex.contains_key(&res) {
                    self.remove_file_pool.clone().shutdown();
                    self.search_file_pool.clone().shutdown();
                    return *self.ex.get(&res).unwrap();
                }
            }
            RendererOperation::REMOVE => {
                self.remove_file(res);
            }
            _ => {}
        }
        return res;
    }
    fn remove_file(&mut self, res: usize) {
        let options_status: SelectStatus;
        let options_len: usize;
        {
            let guard = self.options.lock().unwrap();
            options_len = guard.len();
            if options_len < res {
                return;
            }
            options_status = guard[res].status.clone();
        }
        if let SelectStatus::Live = options_status {
            let path: String;
            {
                let mut guard = self.options.lock().unwrap();
                guard[res].status = SelectStatus::Deleting;
                guard[res].end = "[Removing]".to_string();
                self.need_refresh.swap(true, std::sync::atomic::Ordering::Relaxed);
                path = guard[res].path.clone()
            };
            let _options = self.options.clone();
            let _need_refresh = self.need_refresh.clone();
            self.remove_file_pool.evaluate(move || {
                if let Err(e) = fs::remove_dir_all(path) {
                    let mut guard = _options.lock().unwrap();
                    guard[res].status = SelectStatus::Deleted;
                    guard[res].end = format!("[Err{}]", e.to_string());
                    _need_refresh.swap(true, std::sync::atomic::Ordering::Relaxed);
                } else {
                    let mut guard = _options.lock().unwrap();
                    guard[res].status = SelectStatus::Deleted;
                    guard[res].end = "[Removed]".to_string();
                    _need_refresh.swap(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
        }
    }
}

#[derive(Clone, Debug)]
pub enum RendererOperation {
    SYSTEM,
    REMOVE,
    ANOTHER,
    NONE
}

fn search_files(path: &str, config_loader: Arc<ConfigChecker>, options: Arc<Mutex<Vec<SelectOptions>>>, pools: Arc<ThreadPool>, need_refresh: Arc<AtomicBool>) {
    // let mut input_template_all: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        let mut dirs: Vec<String> = vec![];
        let mut files: Vec<String> = vec![];
        let mut dirs_path: Vec<String> = vec![];
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(file_name) = file_path.file_name() {
                        if let Some(name) = file_name.to_str() {
                            files.push(name.to_owned());
                        }
                    }
                } else if file_path.is_dir() {
                    if let Some(file_name) = file_path.file_name() {
                        if let Some(name) = file_name.to_str() {
                            dirs.push(name.to_owned());
                            dirs_path.push(file_path.to_str().unwrap().to_owned());
                            if config_loader.is_shallow(&name.to_owned()) == true {
                                continue;
                            }
                        }
                    }
                    let _pools = pools.clone();
                    let _options = options.clone();
                    let _config_loader = config_loader.clone();
                    let _nr = need_refresh.clone();
                    pools.evaluate(move || search_files(file_path.clone().to_str().unwrap(), _config_loader, _options, _pools, _nr));
                    // let sub_directory = search_files(file_path.to_str().unwrap(), config_loader);
                    // input_template_all.extend(sub_directory);
                }
            }
        }
        if let Some((prefix, target)) = config_loader.check(&dirs, &files) {
            let mut index = dirs.iter().position(|r| r == &target.to_owned()).unwrap();
            {
                let mut guard = options.lock().unwrap();
                let r = SelectOptions::new(dirs_path[index].clone(), prefix, "[wait]".to_owned(), SelectStatus::Searched);
                index = guard.len();
                guard.push(r);
            }
            let _path = path.to_owned().clone();
            let _opt = options.clone();
            let _nr = need_refresh.clone();
            thread::spawn(move || {
                if let Ok(size) = get_size(&_path) {
                    let mut guard = _opt.lock().unwrap();
                    let _size= "[".to_owned() + &(((size as f64 / 1024_f64 / 1024_f64) * 100_f64).round() / 100_f64).to_string() + "Mb]";
                    guard[index].end = _size.clone();
                    // msgbox::create("GOT",&_size,IconType::Info).unwrap();
                    guard[index].status = SelectStatus::Live;
                    // println!("{},{}",index,_size);
                    _nr.swap(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
            // input_template_all.push((dirs_path[index].clone(), prefix))
        }
    }
}
