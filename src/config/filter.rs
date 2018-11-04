use super::*;

use std::fs::File;

use std::collections::BTreeMap;
use imap_extention::fetch::Mail;
use regex::RegexSet;
use regex::Error;

static CONFIG_FILE: &'static str = "filters.toml";

lazy_static! {
    pub static ref FILTER: Config = {
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    filter: BTreeMap<String, Filter>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Filter {
    pub subject_case_sensitive: bool,
    pub subject_contains: Option<Vec<String>>,
    pub subject_not_contains: Option<Vec<String>>,
    pub message_regex: Option<Vec<String>>,
    pub message_not_regex: Option<Vec<String>>,
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

impl Config {
    pub fn check(&self, mail: &Mail, filter: &String) -> bool {
        match self.filter.get(filter) {
            Some(f) => return f.check(mail, &filter),
            None => {
                error!("The filter '{}' was missing, but a empty filter has been add. However you still need to", filter);
                ::std::process::exit(1);
            }
        }
    }

    pub fn check_exist(&self, filter: &String) -> Option<&Filter> {
        match self.filter.get(filter) {
            Some(ref f) => Some(f),
            None => {
                let mut config = FILTER.clone();
                config.filter.insert(filter.to_string(), filter::Filter {
                    subject_case_sensitive: false,
                    subject_contains: Some(vec!["".to_string()]),
                    subject_not_contains: Some(vec!["".to_string()]),
                    message_regex: Some(vec!["".to_string()]),
                    message_not_regex: Some(vec!["".to_string()]),
                });

                error!("The filter '{}' was missing, but a empty filter has been add. However you still need to", filter);
                config.write();
                None
            }
        }
    }
}

impl Filter {
    fn check(&self, mail: &Mail, filter_name: &String) -> bool {
        let mut subject_contains = true;
        let mut subject_not_contains = true;

        match self.subject_contains {
            Some(ref filters) => {
                for filter in filters {
                    if self.subject_case_sensitive {
                        subject_contains = mail.subject.contains(filter) && subject_contains;
                    } else {
                        subject_contains = mail.subject.to_lowercase().contains(filter.to_lowercase().as_str()) && subject_contains;
                    }
                }
            }
            None => subject_contains = true,
        }

        match self.subject_not_contains {
            Some(ref filters) => {
                for filter in filters {
                    if self.subject_case_sensitive {
                        subject_not_contains = !mail.subject.contains(filter) && subject_not_contains;
                    } else {
                        subject_not_contains = !mail.subject.to_lowercase().contains(filter.to_lowercase().as_str()) && subject_not_contains;
                    }
                }
            }
            None => subject_not_contains = true,
        }


        return subject_contains && subject_not_contains &&
            self.message_regex(&mail.text, filter_name) &&
            self.message_not_regex(&mail.text, filter_name);
    }

    fn message_regex(&self, message: &String, filter: &String) -> bool {
        match self.message_regex {
            Some(ref r) => {
                let set = RegexSet::new(r);
                let set = error_handler_regexset(set, filter, "message_regex");

                let matches = set.matches(message);

                if matches.len() == r.len() {
                    return true;
                }

                false
            }
            None => true,
        }
    }

    fn message_not_regex(&self, message: &String, filter: &String) -> bool {
        match self.message_not_regex {
            Some(ref r) => {
                let set = RegexSet::new(r);
                let set = error_handler_regexset(set, filter, "message_not_regex");

                let matches = set.matches(message);

                if matches.len() == 0 {
                    return true;
                }

                false
            }
            None => true,
        }
    }
}

fn config_template() -> Config {
    Config {
        filter: {
            let mut t = BTreeMap::new();
            t.insert("Filter_1".to_string(), Filter {
                subject_case_sensitive: false,
                subject_contains: Some(vec!["[Something]".to_string()]),
                subject_not_contains: Some(vec!["TEST".to_string(), "REMINDER".to_string()]),
                message_regex: Some(vec!["WRITE A REGULAR EXPRESSION".to_owned()]),
                message_not_regex: Some(vec!["WRITE A REGULAR EXPRESSION".to_owned()]),
            });
            t
        },
    }
}

fn error_handler_regexset(set: Result<RegexSet, Error>, filter: &String, key: &str) -> RegexSet {
    if set.is_err() {
        match set.unwrap_err() {
            Error::Syntax(s) =>
                println!("The filter '{}' has a syntax error in '{}'\n{}",
                         filter,
                         key,
                         s),
            Error::CompiledTooBig(limit) =>
                println!("The filter '{}' with the key '{}' exceeded the set size limit. The size limit imposed is '{}'",
                         filter,
                         key,
                         limit),
            _ => println!("Unknown error. There is probable something wrong with the Regex Syntex belonging to '{}' key in filter '{}'",
                          key,
                          filter),
        }
        ::std::process::exit(1);
    } else {
        return set.unwrap();
    }
}