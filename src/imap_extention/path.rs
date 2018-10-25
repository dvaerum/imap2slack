use std::string::String;
use imap::error::Result;
use regex::Regex;
use imap::client::Client;
use std::io::{Read,Write};
use std::str::FromStr;
use imap::Mailbox;
use imap::client::Session;

#[allow(dead_code)]#[derive(Debug)]
pub struct Status {
    pub messages: u32,
    pub recent: u32,
    pub unseen: u32,
}

pub struct Path {
    inner: String,
}

impl Path {
    pub fn new(path: &str) -> Path {
        Path {inner: String::from(path)}
    }

//    #[allow(dead_code)]
//    pub fn status<T: Read + Write>(&self, client: &mut Client<T>) -> Result<Status> {
//        match client.run_command_and_read_response(&format!("STATUS \"{}\" (MESSAGES UNSEEN RECENT)", self.inner)) {
//            Ok(response) => {
//                let response: String = response.into_iter().collect();
//
//                let messages = Regex::new(r"MESSAGES (\d+)").unwrap().captures(&response);
//                let recent   = Regex::new(r"RECENT (\d+)").unwrap().captures(&response);
//                let unseen   = Regex::new(r"UNSEEN (\d+)").unwrap().captures(&response);
//
//                Ok(Status {
//                    messages: messages.map_or(0, |s| s.get(1).map_or(0, |ss| u32::from_str(ss.as_str()).unwrap())),
//                    recent:     recent.map_or(0, |s| s.get(1).map_or(0, |ss| u32::from_str(ss.as_str()).unwrap())),
//                    unseen:     unseen.map_or(0, |s| s.get(1).map_or(0, |ss| u32::from_str(ss.as_str()).unwrap()))
//                })
//            },
//            Err(e) => return Err(e)
//        }
//    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

pub trait PathFrom {
    fn select_from(&mut self, path: &Path) -> Result<Mailbox>;
}

impl<T: Read + Write> PathFrom for Session<T>{
    fn select_from(&mut self, path: &Path) -> Result<Mailbox> {
        self.select(path.as_str())
    }
}