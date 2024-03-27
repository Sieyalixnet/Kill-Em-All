use std::{env, fs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Checker {
    pub target: String,
    contains_files_dirs: Vec<String>,
    excludes_files_dirs: Vec<String>,
    pub prefix: String,
}
impl Checker {
    pub fn check(&self, dirs: &Vec<String>, files: &Vec<String>) -> bool {
        if dirs.contains(&self.target) == false {
            return false;
        }
        //There is any file of contains_files_dirs NOT in the dir, return false.
        for file in &self.contains_files_dirs {
            let mut res = false;
            if dirs.contains(file) == true {
                res = true
            } else if files.contains(file) == true {
                res = true
            }
            if res == false {
                return false;
            }
        }
        //There is any file of excludes_files_dirs in the dir, return false.
        for file in &self.excludes_files_dirs {
            let mut res = true;
            if dirs.contains(file) == true {
                res = false
            } else if files.contains(file) == true {
                res = false
            }
            if res == false {
                return false;
            }
        }
        true
    }
}

pub fn get_default_config_path() -> Option<String> {
    if let Ok(path) = env::current_exe() {
        if let Some(dir) = path.parent() {
            if let Some(dir_path) = dir.to_str() {
                return Some(dir_path.to_owned() + "/config.yaml");
            }
        }
    }
    None
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ConfigChecker {
    includes: Vec<Checker>,
    shallow: Vec<String>,
}

impl ConfigChecker {
    /**
     with Emoji will make some errors when calculating the width. So do not use any Emoji.
     */
    pub fn init(with_emoji: bool) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut checker_npm = Checker {
            target: "node_modules".to_owned(),
            contains_files_dirs: vec!["package.json".to_owned()],
            excludes_files_dirs: vec![],
            prefix: "[Node]".to_owned(),
        };
        let mut checker_rust = Checker {
            target: "target".to_owned(),
            contains_files_dirs: vec!["Cargo.toml".to_owned()],
            excludes_files_dirs: vec![],
            prefix: "[Rust]".to_owned(),
        };
        if with_emoji {
            checker_npm.prefix = "[🟩Node]".to_owned();
            checker_rust.prefix = "[🦀️Rust]".to_owned();
        }
        let config = ConfigChecker {
            includes: vec![checker_npm, checker_rust],
            shallow: vec!["node_modules".to_owned()],
        };
        let config_str = serde_yaml::to_string(&config)?;
        if let Some(path) = get_default_config_path() {
            fs::write(path, config_str)?;
        }
        Ok(())
    }
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let _s: String = String::from_utf8_lossy(&fs::read(path)?).parse()?;
        let loader: ConfigChecker = serde_yaml::from_str(&_s)?;
        Ok(loader)
    }
    pub fn load_init()->Option<Self>{
        //try to load config, if None, create a default one and load it
        if let Some(path) = get_default_config_path() {
            if let Ok(_cl) = ConfigChecker::load(&path) {
                return Some(_cl);
            } else {
                ConfigChecker::init(false).unwrap();
                return Some(ConfigChecker::load(&path).unwrap())
            }
        } else {
            return None;
        }
    }
    
    pub fn is_shallow(&self, file: &String) -> bool {
        self.shallow.contains(file)
    }
    pub fn check(&self, dirs: &Vec<String>, files: &Vec<String>) -> Option<(String, String)> {
        for checker in &self.includes {
            if checker.check(dirs, files) == true {
                return Some((checker.prefix.to_owned(), checker.target.to_owned()));
            }
        }
        None
    }
}
