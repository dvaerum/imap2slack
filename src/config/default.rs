use super::*;
use chrono::{Utc, NaiveDateTime, DateTime, FixedOffset, offset};
use std::str::FromStr;
use std::fs;

static CONFIG_FILE: &'static str = "default.toml";
static TIMESTAMP_FILE: &'static str = "timestamp.txt";

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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub service: bool,
    pub sleep_time: u64,
    mark_mail_as_seen: Option<bool>, // Should be true by default
    debug: Option<bool>, // Should be false by default
    debug_imap: Option<bool>, // Should be false by default
    use_timestamp: Option<bool>, // Should be false by default
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

    pub fn debug_imap(&self) -> bool {
        self.debug_imap.unwrap_or(false)
    }

    pub fn use_timestamp(&self) -> bool {
        self.use_timestamp.unwrap_or(false)
    }

    pub fn get_timestamp(&self) -> DateTime<Utc> {
        let mut ts_file = path_config_dir();
        ts_file.push(TIMESTAMP_FILE);
        let ts_file = ts_file.as_path();

        if ts_file.exists() {
            println!("Open timestamp file - {}", format!("{}", Utc::now().timestamp()));

            let _trim: &[_] = &[' ', '\t', '\n'];
            let ts_str = fs::read_to_string(ts_file).
                expect(format!("Something went wrong with open the file '{}'", ts_file.to_str().unwrap()).as_ref()).
                trim_matches(_trim).to_owned();

            let mut timestamp;
            let tmp_timestamp = i64::from_str(ts_str.as_str());
            if tmp_timestamp.is_ok() {
                timestamp = tmp_timestamp.unwrap()
            } else {
                println!("Error: The timestamp file did not contain a timestamp");
                timestamp = Utc::now().timestamp();
            }

            fs::write(ts_file, format!("{}", Utc::now().timestamp()).as_bytes()).
                expect(format!("Something went wrong with writing the file '{}'", ts_file.to_str().unwrap()).as_ref());

            return DateTime::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
        } else {
            println!("Create timestamp file");
            let timestamp = Utc::now();
            fs::write(ts_file, format!("{}", timestamp.timestamp()).as_bytes()).
                expect(format!("Something went wrong with writing the file '{}'", ts_file.to_str().unwrap()).as_ref());
            return timestamp;
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Mail {
    pub imap: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Slack {
    pub webhook: String,
    pub username: String,
    pub emoji: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Publish {
    pub mailbox: String,
    pub channel: Vec<String>,
    filter: Option<String>,
}

impl Publish {
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
    pub fn filter_exist(&self) -> bool {
        match self.filter.as_ref() {
            Some(filter) => FILTER.check_exist(filter).is_some(),
            None => true,
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
        debug_imap: Some(false),
        use_timestamp: Some(false),
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
