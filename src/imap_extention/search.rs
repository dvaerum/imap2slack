use imap::client::Client;
use imap::error::Result;
use regex::Regex;
use std::vec::Vec;
use std::string::String;
use std::io::{Read,Write};
use std::str::FromStr;

pub enum SEARCH {
    SEEN,
    UNSEEN,
}

pub trait Search {
    fn search(&mut self, filter: Vec<SEARCH>) -> Result<Vec<usize>>;
}

impl<T: Read + Write> Search for Client<T> {
    fn search(&mut self, filter: Vec<SEARCH>) -> Result<Vec<usize>> {
        let criteria: String = filter.iter().map(|s| format!("{} ", search2str(s))).collect();
        match self.run_command_and_read_response(&format!("SEARCH {}", criteria.trim())) {
            Ok(response) => {
                let mut uids: Vec<usize> = Vec::new();

                for l in response {
                    if l.to_uppercase().contains("SEARCH") {
                        for uid in Regex::new(r"(?P<uid>\d+)").unwrap().captures_iter(&l) {
                            uids.push(usize::from_str(&uid["uid"]).unwrap());
                        }
                    }
                }

                Ok(uids)
            },
            Err(e) => Err(e)
        }
    }
}

fn search2str<'a>(s: &SEARCH) -> &'a str{
    match s {
        &SEARCH::SEEN => "SEEN",
        &SEARCH::UNSEEN => "UNSEEN"
    }
}