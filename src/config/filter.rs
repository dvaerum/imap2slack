use super::*;

use std::fs::{File};

use std::collections::BTreeMap;

static CONFIG_FILE: &'static str = "filters.toml";

lazy_static! {
    pub static ref CONFIG: Config = {
        read_config(CONFIG_FILE, config_template())
    };
}

fn read_config(config_file: &str, config: Config) -> Config {
    use std::io::Read;

    let mut config_file = init(config_file, config);

    let mut data = String::new();
    config_file.read_to_string(&mut data);
    error_handler(toml::from_str(&data))
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Config {
    pub filter: BTreeMap<String, Filter>,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Filter {
    pub case_sensitive: bool,
    pub contains: Option<Vec<String>>,
    pub does_not_contains: Option<Vec<String>>,
}

impl WriteConfig for Config {
    fn write(&self) {
        let path_config_file = path_config_file(CONFIG_FILE);

        let mut config_file = File::create(&path_config_file).expect(&format!("Failed at creating a template config file '{}'", &path_config_file.to_str().unwrap()));

        let toml = toml::to_string(self).unwrap();
        config_file.write_all(toml.as_bytes()).expect(&format!("Failed to create a config file"));

        println!("Edit the config file '{}'", &path_config_file.to_str().unwrap());
    }
}

impl Filter {
    pub fn check(&self, text: &String) -> bool {
        let mut r_contains: bool = true;
        let mut r_does_not_contains = true;

        match self.contains {
            Some(ref filters) => {
                for filter in filters {
                    if self.case_sensitive {
                        r_contains = text.contains(filter.as_str()) && r_contains;
                    } else {
                        r_contains = text.to_lowercase().as_str().contains(filter.to_lowercase().as_str()) && r_contains;
                    }
                }
            }
            None => r_contains = true,
        }

        match self.does_not_contains {
            Some(ref filters) => {
                for filter in filters {
                    if self.case_sensitive {
                        r_does_not_contains = !text.as_str().contains(filter.as_str()) && r_does_not_contains;
                    } else {
                        r_does_not_contains = !text.to_lowercase().as_str().contains(filter.to_lowercase().as_str()) && r_does_not_contains;
                    }
                }
            }
            None => r_does_not_contains = true,
        }

        if r_contains && r_does_not_contains {
            return true;
        } else {
            return false;
        }
    }
}

fn config_template() -> Config {
    Config {
        filter: { let mut t = BTreeMap::new();
            t.insert("Filter_1".to_string(),Filter {
                case_sensitive: false,
                contains: Some(vec!["[Something]".to_string()]),
                does_not_contains: Some(vec!["TEST".to_string(), "REMINDER".to_string()]),
            });
            t
        },
    }
}
