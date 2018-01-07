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

                            let uid = usize::from_str(&uid_and_flags["uid"]).unwrap();
                            let flags: String = String::from(&uid_and_flags["flags"]);

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

//                    println!("---===( fetched mail )===---\n{}", String::from_utf8_lossy(&mail_buffer));

                    let mail = mailparse::parse_mail(&mail_buffer).unwrap();
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