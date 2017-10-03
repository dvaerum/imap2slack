use imap::client::Client;
use imap::error::Result;
use regex::Regex;
use std::vec::Vec;
use std::string::String;
use std::str::FromStr;
use std::io::{Read,Write};
use quoted_printable::{decode, ParseMode};

#[derive(Debug)]
pub struct Mail {
    pub uid: usize,
    pub flags: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub reply_to: String,
    pub subject: String,
    pub date: String,
    pub text: String,
}

#[allow(dead_code)]
impl Mail {
    fn print(&self) {
        println!("uid: {}\nflags: {}\nfrom: {}\nto: {}\ncc: {}\nbcc: {}\nreply_to: {}\nsubject: {}\ndate: {}\ntext: {}\n",
                 self.uid, self.flags, self.from, self.to, self.cc, self.bcc, self.reply_to, self.subject, self.date, self.text);
    }

    fn print_debug(&self) {
        println!("uid: {:?}\nflags: {:?}\nfrom: {:?}\nto: {:?}\ncc: {:?}\nbcc: {:?}\nreply_to: {:?}\nsubject: {:?}\ndate: {:?}\ntext: {:?}\n",
                 self.uid, self.flags, self.from, self.to, self.cc, self.bcc, self.reply_to, self.subject, self.date, self.text);
    }
}

pub trait Folder {
    fn fetch_ext(&mut self, sequence_set: &Vec<usize>) -> Result<Vec<Mail>>;
}

impl<T: Read + Write> Folder for Client<T> {
    fn fetch_ext(&mut self, sequence_set: &Vec<usize>) -> Result<Vec<Mail>> {
        let mut r: Vec<Mail> = Vec::new();

        for sequence in sequence_set {

            match self.fetch(&sequence.to_string(), "(FLAGS BODY.PEEK[HEADER] BODY.PEEK[TEXT])") {
                Ok(responses) => {
                    let responses: String = responses.iter().map(|s| s.to_string()).collect();

                    let uid_and_flags = Regex::new(r#"[*][ ]+(?P<uid>\d+)[ ]+FETCH[ ]+\(FLAGS \((?P<flags>[^)]+)\)[ ]+(?s)(?P<next>.+)"#).unwrap().captures(&responses)
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let uid = usize::from_str(&uid_and_flags["uid"]).unwrap();
                    let flags: String = String::from(&uid_and_flags["flags"]);

    //                let header_fields = Regex::new(r#"BODY\[[^]]+[ ]+\{(?P<size>\d+)\}[^\n]+(?P<next>.+)"#).unwrap().captures(&uid_and_flags["next"])
                    let header_fields = Regex::new(r#"BODY\[[^\]]+\][ ]+\{(?P<size>\d+)\}[^\n]+\n(?s)(?P<next>.+)"#).unwrap().captures(&uid_and_flags["next"])
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let header_fields_size = usize::from_str(&header_fields["size"]).unwrap();
                    let header_fields_split = header_fields["next"].split_at(header_fields_size);

                    let from =     Regex::new(r"(?i)(^F|\nF)rom: (?P<from>.+?)\r?\n").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());
                    let to =       Regex::new(r"(?i)(^T|\nT)o: (?P<to>.+?)\r?\n").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());
                    let cc =       Regex::new(r"(?i)(^C|\nC)c: (?P<cc>.+?)\r?\n").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());
                    let bcc =      Regex::new(r"(?i)(^B|\nB)cc: (?P<bcc>.+?)\r?\n").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());
                    let reply_to = Regex::new(r"(?i)(^R|\nR)eply-To: (?P<reply_to>.+?)\r?\n").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());
                    let subject =  Regex::new(r"(?i)(^S|\nS)ubject:(?s) (?P<reply_to>(\r?\n\t?\s|.)+?)(\r?\n\w)").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().replace("\r\n ", " ").to_string());
                    let date =     Regex::new(r"(?i)(^D|\nD)ate: (?P<date>.+?)(\r?\n)").unwrap().captures(header_fields_split.0)
                        .map_or(String::new(), |s| s.get(2).unwrap().as_str().to_string());

                    let text_fields = Regex::new(r#"BODY\[TEXT\][ ]+\{(?P<size>\d+)\}[^\n]+\n(?s)(?P<next>.+)"#).unwrap().captures(&header_fields_split.1)
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let text_fields_size = usize::from_str(&text_fields["size"]).unwrap();
                    let text_fields_split = text_fields["next"].split_at(text_fields_size);

                    let text = String::from_utf8(decode(text_fields_split.0.replace("=\r\n", ""), ParseMode::Robust).unwrap()).unwrap();

                    r.push(Mail {
                        uid: uid,
                        flags: flags,
                        from: from,
                        to: to,
                        cc: cc,
                        bcc: bcc,
                        reply_to: reply_to,
                        subject: subject,
                        date: date,
                        text: text,
                    });
                },
                Err(e) => return Err(e)
            }
        }

        Ok(r)
    }
}