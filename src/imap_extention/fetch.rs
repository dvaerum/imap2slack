use imap::client::Client;
use imap::error::Result;
use regex::Regex;
use std::vec::Vec;
use std::string::String;
use std::str::FromStr;
use std::io::{Read,Write};
use quoted_printable::{decode, ParseMode};
use super::mailparse;

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
    pub fn print(&self) {
        println!("uid: {}\nflags: {}\nfrom: {}\nto: {}\ncc: {}\nbcc: {}\nreply_to: {}\nsubject: {}\ndate: {}\ntext: {}\n",
                 self.uid, self.flags, self.from, self.to, self.cc, self.bcc, self.reply_to, self.subject, self.date, self.text);
    }

    pub fn print_debug(&self) {
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
                    let mut from = String::new();
                    let mut to = String::new();
                    let mut cc = String::new();
                    let mut bcc = String::new();
                    let mut reply_to = String::new();
                    let mut subject = String::new();
                    let mut date = String::new();
                    let mut text = String::new();

                    let responses: String = responses.iter().map(|s| s.to_string()).collect();

                    let uid_and_flags = Regex::new(r#"[*][ ]+(?P<uid>\d+)[ ]+FETCH[ ]+\(FLAGS \((?P<flags>[^)]+)\)[ ]+(?s)(?P<next>.+)"#).unwrap().captures(&responses)
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let uid = usize::from_str(&uid_and_flags["uid"]).unwrap();
                    let flags: String = String::from(&uid_and_flags["flags"]);

//                    print!("\n\n\n---===( uid )===---\n{}", uid);

//                    print!("\n\n\n---===( flags )===---\n{}", flags);

                    let header_fields = Regex::new(r#"BODY\[[^\]]+\][ ]+\{(?P<size>\d+)\}[^\n]+\n(?s)(?P<next>.+)"#).unwrap().captures(&uid_and_flags["next"])
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let header_fields_size = usize::from_str(&header_fields["size"]).unwrap();
                    let header_fields_split = header_fields["next"].split_at(header_fields_size);

//                    println!("\n\n\n---===( header_fields_split.0 )===---\n{}---------------------------", header_fields_split.0);

                    let text_fields = Regex::new(r#"BODY\[TEXT\][ ]+\{(?P<size>\d+)\}[^\n]+\n(?s)(?P<next>.+)"#).unwrap().captures(&header_fields_split.1)
                        .expect("Something is wrong with the passing (regex) of the data from the fetch mail");
                    let text_fields_size = usize::from_str(&text_fields["size"]).unwrap();
                    let text_fields_split = text_fields["next"].split_at(text_fields_size);

                    let tmp_text = String::from_utf8(decode(text_fields_split.0.replace("=\r\n", ""), ParseMode::Robust).unwrap()).unwrap();


//                    print!("\n================================================\n{}{}===================================\n\n", header_fields_split.0, tmp_text);

                    let mail_as_string: String =  format!("{}{}", header_fields_split.0, tmp_text);
                    let mail = mailparse::parse_mail(mail_as_string.as_bytes()).unwrap();
//                    let (subject_tmp, _) = mailparse::parse_headers(header_fields_split.0.as_bytes()).unwrap();
                    //                    let subject = subject_tmp.get_value().unwrap();

                    for header in &mail.headers {
//                        println!("HEADER -> {}: {}", header.get_key().unwrap(), header.get_value().unwrap());
                        //                        if mailheader.get_key().unwrap().to_lowercase() ==  "subject" {
                        //                            subject = mailheader.get_value().unwrap();
                        //                        }

                        match header.get_key().unwrap().to_lowercase().as_ref() {
                            "from" => from = header.get_value().unwrap(),
                            "to" => to = header.get_value().unwrap(),
                            "cc" => cc = header.get_value().unwrap(),
                            "bcc" => bcc = header.get_value().unwrap(),
                            "reply_to" => reply_to = header.get_value().unwrap(),
                            "subject" => subject = header.get_value().unwrap(),
                            "date" => date = header.get_value().unwrap(),
                            _ => (),
                        }
                    }

                    // This piece of code is used for debugging
//                    let tmp = mail.get_body().expect("Fail to pass the BODY of the mail and error handling is needed");
//                    println!("\n==============( mail.get_body() - size: {} )===================\n{}\n\n", tmp.len(), tmp);
//                    for i in 0..mail.subparts.len() {
//                        println!("\n==============( subpart {} out of {} )===================", i+1, mail.subparts.len());
//                        for header in &mail.subparts[i].headers {
//                            println!("HEADER -> {}: {}", header.get_key().unwrap(), header.get_value().unwrap());
//                        }
//                        println!("--==( BODY )==--");
//                        println!("{}", mail.subparts[i].get_body().expect("Fail to pass the BODY of the mail and error handling is needed"));
//                    }

                    text = mail.get_body().expect("Fail to pass the BODY of the mail and error handling is needed");
                    if text.len() == 0 {
                        for i in 0..mail.subparts.len() {
                            let mut content_type = String::new();
                            for header in &mail.subparts[i].headers {
                                if header.get_key().unwrap().to_lowercase() == "content-type" {
                                    content_type = header.get_value().unwrap().to_lowercase();
                                }
                            }

                            if content_type.len() == 0 {
                                panic!("This mail don't have the Content-Type header, figure out have to handle it");
                            } else if content_type.contains("text/plain") {
                                text = mail.subparts[i].get_body().expect("Fail to pass the BODY of the mail and error handling is needed");
                            }
                        }
                    }

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