use imap::error::Result;
use std::vec::Vec;
use std::string::String;
use std::io::{Read,Write};
use super::mailparse::{self, ParsedContentType, ParsedMail};
use std::collections::HashMap;
use config::DEFAULT;
use imap::client::Session;

#[derive(Debug)]
pub struct Mail {
    pub uid: u32,
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

impl<T: Read + Write> Folder for Session<T> {
    fn fetch_mail(&mut self, sequence_set: &Vec<usize>) -> Result<Vec<Mail>> {
        let mut r: Vec<Mail> = Vec::new();

        for sequence in sequence_set {

//            let fetch = self.fetch(&sequence.to_string(), "(FLAGS BODY.PEEK[HEADER] BODY.PEEK[TEXT])");
            let fetch = self.fetch(&sequence.to_string(), "(FLAGS BODY.PEEK[])");
//            println!("test_1: {:?}", fetch);
            match fetch {
                Ok(mut responses) => {
                    if (*responses).len() > 1 {
                        println!("WARNING response size is bigger then 1 and you need it find a way to deal with it");
                    }

                    let response = &(*responses)[0];

                    let mut uid = response.message;
                    let mut flags = String::new();
                    let mut from = String::new();
                    let mut to = String::new();
                    let mut cc = String::new();
                    let mut bcc = String::new();
                    let mut reply_to = String::new();
                    let mut subject = String::new();
                    let mut date = String::new();
                    let mut text: String = String::new();


//                    println!("====================================================");
//                    println!("test_2: {:?}", &response);
//                    println!("====================================================");
//                    println!("====================================================");
//                    println!("test_2: {}", String::from_utf8_lossy((*responses)[0].body().unwrap()));
//                    println!("====================================================");

                    let mut mail_buffer: Vec<u8> = response.body().unwrap().to_vec();

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