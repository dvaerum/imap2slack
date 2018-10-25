use imap::client::Client;
use imap::error::Result;
use regex::Regex;
use std::vec::Vec;
use std::string::String;
use std::io::{Read,Write};
use std::str::FromStr;
use imap::client::Session;

#[allow(dead_code)]
pub enum SEARCH {
    SEEN,
    UNSEEN,
}

pub trait Search {
    fn search2(&mut self, filter: Vec<SEARCH>) -> Result<Vec<usize>>;
}

impl<T: Read + Write> Search for Session<T> {
    fn search2(&mut self, filter: Vec<SEARCH>) -> Result<Vec<usize>> {
        let criteria: String = filter.iter().map(|s| format!("{} ", search2str(s))).collect();
        let search_result = self.run_command_and_read_response(&format!("SEARCH {}", criteria.trim()));
        println!("Search; {:?}", &search_result);
        match search_result {
            Ok(response) => {
                let mut uids: Vec<usize> = Vec::new();

                let response = String::from_utf8(response).expect("Failed to convert 'Vec<u8>' to 'String', but this should not happen");
                println!("{}", &response);
                if response.to_uppercase().contains("SEARCH") {
                    for uid in Regex::new(r"(?P<uid>\d+)").unwrap().captures_iter(&response) {
                        uids.push(usize::from_str(&uid["uid"]).unwrap());
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