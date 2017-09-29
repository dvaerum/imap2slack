use std::env::home_dir;
use std::fs::{create_dir_all,File};
use std::path::Path;
use std::io::{Read,Write};

use toml;
use serde_derive;
use lazy_static;


static CONFIG_FOLDER: &'static str = ".config/imap2slack";
static CONFIG_FILE: &'static str = "config.toml";

fn init() -> File {
    let mut path_config_dir = home_dir().unwrap();
    path_config_dir.push(CONFIG_FOLDER);
    let mut path_config_file = path_config_dir.clone();
    path_config_file.push(CONFIG_FILE);
    let path_config_file = path_config_file.as_path();
    let mut config_file: File;


    create_dir_all(path_config_dir.as_path()).expect(&format!("Failed to create the directory '{}'", path_config_dir.to_str().unwrap()));

    if path_config_file.exists() {
        if path_config_file.is_file() {
            config_file = File::open(path_config_file).expect(&format!("unable to open the config file '{}'", path_config_file.to_str().unwrap()));
        } else {
            panic!("Cannot access the config file '{}'", path_config_file.to_str().unwrap());
        }
    } else {
        config_file = File::create(path_config_file).expect(&format!("Failed at creating a template config file '{}'", path_config_file.to_str().unwrap()));
        write_config_template(&mut config_file);
        println!("Edit the config file '{}'", path_config_file.to_str().unwrap());
        ::std::process::exit(1);
    }

    config_file
}

fn write_config_template(f: &mut File) {
    f.write_all(b"\
    service = true\n\
    sleep_time = 5   # unit is minutes
    \n\
    [mail]\n\
    imap = 'imap.domain.com'\n\
    port = 993\n\
    username = 'my@mail.com'\n\
    password = '*******'\n\
    mailbox = 'Inbox'\n\
    \n\
    [slack]\n\
    webhook = 'https://hooks.slack.com/services/xxx/yyy/zzz'\n\
    username = 'BOT'\n\
    channel = '#testing'\n\
    emoji = '+1'\n\
    ").expect(&format!("Failed to create a config file"))
}

#[derive(Deserialize)]
pub struct Config {
    pub service: bool,
    pub sleep_time: u64,
    pub mail: Mail,
    pub slack: Slack,
}

#[derive(Deserialize)]
pub struct Mail {
    pub imap: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub mailbox: String
}

#[derive(Deserialize)]
pub struct Slack {
    pub webhook: String,
    pub username: String,
    pub channel: String,
    pub emoji: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        read_config()
    };
}

fn read_config() -> Config {
    let mut config_file = init();
    let mut data = String::new();

    config_file.read_to_string(&mut data);

    toml::from_str(&data).unwrap()
}