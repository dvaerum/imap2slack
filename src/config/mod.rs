extern crate serde;

pub mod default;
pub mod filter;
pub use self::default::CONFIG as DEFAULT;
pub use self::filter::CONFIG as FILTER;

use std::env::home_dir;
use std::fs::{create_dir_all,File};
use std::io::{Read,Write};
use std::path::{Path,PathBuf};

use toml;

use self::serde::Deserialize;
use self::serde::Serialize;

static CONFIG_FOLDER: &'static str = ".config/imap2slack";

fn init<'de, T>(filename: &str, config: T) -> File where T: Deserialize<'de> + Serialize + WriteConfig {
    let path_config_file = path_config_file(filename);
    let mut config_file: File;

    let path_config_dir = path_config_dir();
    create_dir_all(path_config_dir.as_path()).expect(&format!("Failed to create the directory '{}'", path_config_file.to_str().unwrap()));

    if path_config_file.exists() {
        if path_config_file.is_file() {
            config_file = File::open(&path_config_file).expect(&format!("unable to open the config file '{}'", path_config_file.to_str().unwrap()));
        } else {
            panic!("Cannot access the config file '{}'", path_config_file.to_str().unwrap());
        }
    } else {
        config.write();
        ::std::process::exit(1);
    }

    config_file
}

fn error_handler<T>(config: serde::export::Result<T, toml::de::Error>) -> T where T: self::serde::export::fmt::Debug {
    if config.is_err() {
        let error = &config.unwrap_err().inner;
        match error.kind {
            toml::de::ErrorKind::Custom => {
                println!("You have to add the {}, in the section [{}] of the config file", error.message, error.key.first().unwrap());
            }
            _ => {
                println!("-=( Un-handled error )=-");
                println!("{:?}", error);
            }
        }

        ::std::process::exit(1);
    }

    config.unwrap()
}

fn path_config_dir() -> PathBuf {
    let mut path_config_dir = home_dir().unwrap();
    path_config_dir.push(CONFIG_FOLDER);
    path_config_dir
}

fn path_config_file(filename: &str) -> PathBuf {
    let mut path_config_file = path_config_dir();
    path_config_file.push(filename);
    path_config_file
}

trait WriteConfig {
    fn write(&self);
}

fn write_config<T>(filename: &str, config: &T) where T: Serialize {
    let path_config_file = path_config_file(filename);

    let mut config_file = File::create(&path_config_file).expect(&format!("Failed at creating a template config file '{}'", &path_config_file.to_str().unwrap()));

    let toml = toml::to_string(&config).unwrap();
    config_file.write_all(toml.as_bytes()).expect(&format!("Failed to create a config file"));

    println!("Edit the config file '{}'", &path_config_file.to_str().unwrap());
}