use super::*;

static CONFIG_FILE: &'static str = "default.toml";

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
    pub service: bool,
    pub sleep_time: u64,
    mark_mail_as_seen: Option<bool>, // Should be true by default
    debug: Option<bool>, // Should be false default
    pub mail: Mail,
    pub slack: Slack,
    pub publish: Vec<Publish>,
}

impl Config {
    pub fn mark_mail_as_seen(&self) -> bool {
        self.mark_mail_as_seen.unwrap_or(true)
    }

    pub fn debug(&self) -> bool {
        self.debug.unwrap_or(false)
    }
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Mail {
    pub imap: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Slack {
    pub webhook: String,
    pub username: String,
    pub emoji: String,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Publish {
    pub mailbox: String,
    pub channel: Vec<String>,
    filter: Option<String>,
}

impl Publish {
    pub fn filter(&self) -> Option<&filter::Filter> {
        match &self.filter {
            &Some(ref f) => {
                match FILTER.filter.get(f) {
                    Some(filter) => return Some(filter),
                    None => {
                        let mut config = FILTER.clone();
                        config.filter.insert(f.to_string(), filter::Filter {
                            case_sensitive: false,
                            contains: Some(vec!["".to_string()]),
                            does_not_contains: Some(vec!["".to_string()]),
                        });

                        println!("The filter '{}' was missing, but a empty filter has been add. However you still need to", f);
                        config.write();
                        ::std::process::exit(1);
                    },
                }
            },
            &None => None
        }
    }
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

fn config_template() -> Config {
    Config {
        service: true,
        sleep_time: 5,
        mark_mail_as_seen: Some(true),
        debug: Some(false),
        mail: Mail {
            imap: "imap.domain.com".to_string(),
            port: 993,
            username: "my@mail.com".to_string(),
            password: "*******".to_string(),
        },
        slack: Slack {
            webhook: "https://hooks.slack.com/services/xxx/yyy/zzz".to_string(),
            username: "BOT".to_string(),
            emoji: "+1".to_string(),
        },
        publish: vec![
            Publish {
                mailbox: "Inbox".to_string(),
                channel: vec!["#testing_1".to_string(), "#testing_2".to_string()],
                filter: None,
            }, Publish {
                mailbox: "Archive".to_string(),
                channel: vec!["#general".to_string()],
                filter: Some("Filter_1".to_string()),
            }],
    }
}
