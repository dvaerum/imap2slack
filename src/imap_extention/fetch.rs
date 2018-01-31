use imap::client::Client;
use imap::error::Result;
use regex::Regex;
use std::vec::Vec;
use std::string::String;
use std::str::FromStr;
use std::io::{Read,Write};
use quoted_printable::{decode, ParseMode};
use super::mailparse::{self, MailHeader, ParsedContentType, ParsedMail};
use std::collections::HashMap;
use config::DEFAULT;

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

struct MailBodyPart {
    headers: HashMap<String, String>,
    ctype: ParsedContentType,
    body: String,
}

fn find_mail_body_parts(mbp: &mut Vec<MailBodyPart>, mail_body: ParsedMail) {
    let ctype = mail_body.ctype.mimetype.to_lowercase();

    if ctype.contains("multipart/") {
        for subpart in mail_body.subparts {
            find_mail_body_parts(mbp, subpart);
        }
    } else if ctype.contains("text/plain") {
        let mut headers = HashMap::new();
        for header in &mail_body.headers {
            headers.insert(header.get_key().unwrap(), header.get_value().unwrap());
        }

        mbp.push(MailBodyPart{
            headers: headers,
            body: mail_body.get_body().unwrap(),
            ctype: mail_body.ctype,
        });
    }
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
    fn fetch_mail(&mut self, sequence_set: &Vec<usize>) -> Result<Vec<Mail>>;
}

impl<T: Read + Write> Folder for Client<T> {
    fn fetch_mail(&mut self, sequence_set: &Vec<usize>) -> Result<Vec<Mail>> {
        let mut r: Vec<Mail> = Vec::new();

        for sequence in sequence_set {

            match self.fetch_raw(&sequence.to_string(), "(FLAGS BODY.PEEK[HEADER] BODY.PEEK[TEXT])") {
                Ok(mut responses) => {
                    let mut uid = 0;
                    let mut flags = String::new();
                    let mut from = String::new();
                    let mut to = String::new();
                    let mut cc = String::new();
                    let mut bcc = String::new();
                    let mut reply_to = String::new();
                    let mut subject = String::new();
                    let mut date = String::new();
                    let mut text: String = String::new();


                    let mut mail_buffer: Vec<u8> = Vec::new();
                    let mut counter: usize = 0;

                    while counter < responses.len() {
                        let string = String::from_utf8(responses[counter].clone()).unwrap();
//                        print!("{}", String::from_utf8_lossy(&responses[counter]));

                        if string.starts_with("*") {
                            let uid_and_flags = Regex::new(r#"[*][ ]+(?P<uid>\d+)[ ]+FETCH[ ]+\(FLAGS \((?P<flags>[^)]+)\)"#)
                                .unwrap()
                                .captures(&string)
                                .expect("Something went wrong with finding UID and FLAGS from the fetched mail");

                            uid = usize::from_str(&uid_and_flags["uid"]).unwrap();
                            flags = String::from(&uid_and_flags["flags"]);

//                            println!("---===( uid: {} )===---", uid);
//                            println!("---===( flags: {} )===---", flags);
                        }

                        let body_size = usize::from_str( Regex::new(r#"BODY\[[^\]]+\][ ]+\{(?P<size>\d+)\}"#)
                            .unwrap()
                            .captures(&string)
//                            .expect("Something went wrong with finding the size of BODY in the fetched mail");
                            .map_or( "0", |s| s.name("size").map_or("0", |s| s.as_str()))).unwrap();
//                        println!("---===( size: {} )===---", body_size);

                        counter += 1;

                        if body_size > 0 {
                            let mut body_counter = 0;
                            while body_counter < body_size {
                                if body_counter + responses[counter].len() <= body_size {
                                    body_counter += responses[counter].len();
//                                    print!("{}", String::from_utf8_lossy(&responses[counter]));
                                    mail_buffer.append(responses[counter].as_mut());
                                } else {
                                    body_counter += responses[counter].len();
                                    let (left, right) = responses[counter].split_at(body_size - body_counter);
//                                    println!("{}", String::from_utf8_lossy(&left));
                                    mail_buffer.extend_from_slice(left);
                                }
                                counter += 1;
                            }
                        }
                    }

                    let mail = mailparse::parse_mail(&mail_buffer).unwrap();

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

                    let mut mail_body_parts: Vec<MailBodyPart> = Vec::new();
                    find_mail_body_parts(&mut mail_body_parts, mail);

                    if DEFAULT.debug() {
                        for i in 0..mail_body_parts.len() {
                            println!("---===( subpart {} )===---", i);
                            println!("headers: {:?}", &mail_body_parts[i].headers);
                            println!("ctype: {:?}", &mail_body_parts[i].ctype);
                            println!("body.len: {}", &mail_body_parts[i].body.len());
                            println!("");
                        }
                    }

                    for mail_body_part in mail_body_parts {
                        if mail_body_part.ctype.mimetype.contains("text/plain") {
                            text = mail_body_part.body;
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