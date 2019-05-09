extern crate imap;
extern crate native_tls;
extern crate regex;
extern crate quoted_printable;
#[macro_use]
extern crate lazy_static;

extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate chrono;
extern crate core;


use std::thread::sleep;
use std::time;
use std::convert::TryFrom;

use native_tls::TlsConnector;
use imap::Client;
use imap::error::Error;

mod imap_extention;

use imap_extention::search::*;
use imap_extention::fetch::*;
use imap_extention::path::{Path, PathFrom};

mod config;

use config::DEFAULT;
use config::FILTER;

mod slack;

use slack::post_mails;
use chrono::{DateTime, Utc, NaiveDateTime, FixedOffset, Duration};

use regex::Regex;


// To connect to the gmail IMAP server with this you will need to allow unsecure apps access.
// See: https://support.google.com/accounts/answer/6010255?hl=en
// Look at the gmail_oauth2.rs example on how to connect to a gmail server securely.
fn main() {
    let re_timezone_name: Regex = Regex::new(r"\([A-Za-z0-9]+\)").unwrap();

    simple_logger::init_with_level(log::Level::Info).unwrap();

    for publish in &DEFAULT.publish {
        if !&publish.filter_exist() {
            ::std::process::exit(1);
        }
    }

    let domain: &str = &DEFAULT.mail.imap;
    let port: u16 = DEFAULT.mail.port.clone();
    let socket_addr = (domain, port);

    let tls = TlsConnector::builder().build().unwrap();

    loop {
        let client: Client<native_tls::TlsStream<std::net::TcpStream>>;
        match imap::connect(socket_addr, domain, &tls) {
            Ok(mut _client) => {
                _client.debug = DEFAULT.debug_imap();
                client = _client
            }
            Err(e) => {
                match e {
                    // An `io::Error` that occurred while trying to read or write to a network stream.
                    Error::Io(io_error) => {
                        error!("{:?}", io_error);
                        ::std::process::exit(1);
                    }
                    // An error from the `native_tls` library during the TLS handshake.
                    Error::TlsHandshake(tls_handshake_error) => {
                        error!("{:?}", tls_handshake_error);
                        ::std::process::exit(1);
                    }
                    // An error from the `native_tls` library while managing the socket.
                    Error::Tls(tls_error) => {
                        error!("{:?}", tls_error);
                        ::std::process::exit(1);
                    }
                    // A BAD response from the IMAP server.
                    Error::Bad(response) => {
                        error!("{:?}", response);
                        ::std::process::exit(1);
                    }
                    // A NO response from the IMAP server.
                    Error::No(response) => {
                        error!("{:?}", response);
                        ::std::process::exit(1);
                    }
                    // The connection was terminated unexpectedly.
                    Error::ConnectionLost => {
                        error!("Connection to the server has been lost");
                        ::std::process::exit(1);
                    }
                    // Error parsing a server response.
                    Error::Parse(parse_error) => {
                        error!("{:?}", parse_error);
                        ::std::process::exit(1);
                    }
                    // Error validating input data
                    Error::Validate(validate_error) => {
                        error!("{:?}", validate_error);
                        ::std::process::exit(1);
                    }
                    // Error appending a mail
                    Error::Append => {
                        error!("Error appending a mail");
                        ::std::process::exit(1);
                    }
                }
            }
        };

        let mut session = client.login(&DEFAULT.mail.username, &DEFAULT.mail.password).unwrap();

        let mut search_args: Vec<SEARCH>;
        let mut timestamp: Option<DateTime<Utc>> = None;
        if DEFAULT.use_timestamp() {
            timestamp = Some(DEFAULT.get_timestamp());
            search_args = vec![SEARCH::SINCE(timestamp.clone().unwrap() - Duration::days(1))];
        } else {
            search_args = vec![SEARCH::UNSEEN];
        }

        for publish in &DEFAULT.publish {
            let path = Path::new(&publish.mailbox);
            let mut uids: Vec<usize> = Vec::new();

            info!("--- mailbox - {} ---", &path.as_str());
            match session.select_from(&path) {
                Ok(mailbox) => debug!("Selected mailbox - '{}'", mailbox),
                Err(e) => error!("Error selecting INBOX: {}", e),
            };

            match session.search2(&search_args) {
                Ok(u) => {
                    if DEFAULT.debug() {
                        println!("---===( Search )===---\n{:?}", &u);
                    }
                    uids = u
                }
                Err(e) => println!("Failed in searching for mail: {}", e),
            };

            if DEFAULT.debug() {
                println!("---===( Fetch )===---");
            }
            let fetch = session.fetch_mail(&uids);
            println!("The number of email return from fetch {}", &fetch.as_ref().map(|x| i32::try_from(x.len()).unwrap()).unwrap_or(-1));
            match fetch {
                Ok(mails) => {
                    for mail in &mails {
                        if DEFAULT.debug() {
                            println!("uid: {}, flags: '{}', from: '{}', to: '{}', cc: '{}', bcc: '{}', reply_to: '{}', subject: '{}', date: '{}'",
                                     &mail.uid, &mail.flags, &mail.from, &mail.to, &mail.cc, &mail.bcc, &mail.reply_to, &mail.subject, &mail.date);
                        }

                        if DEFAULT.use_timestamp() {
                            let datetime = timestamp.unwrap();
                            let mut date = re_timezone_name.replace_all(&mail.date, "").to_string().trim().to_owned();

                            let mail_ts_fixed = DateTime::parse_from_str(&date, "%a, %d %b %Y %H:%M:%S %z").
                                expect("There is a format in the Date header of the email there is not handled correct");
                            let mail_ts_utc = mail_ts_fixed.clone().with_timezone(&Utc);


                            println!("---===( Compare Timestamp )===---\nLocal: {} - Mail: {}\n{}\n\n", datetime, &mail_ts_utc,
                                     if mail_ts_utc < datetime {"Local DT the biggest"} else {"Mail DT the biggest"});
                            if mail_ts_utc < datetime {
                                continue;
                            }
                        }
                        match publish.filter() {
                            Some(filter) => {
                                if FILTER.check(&mail, filter) {
                                    post_mails(mail, &publish.channel);
                                }
                            }
                            None => {
                                post_mails(mail, &publish.channel);
                            }
                        }

                        if DEFAULT.mark_mail_as_seen() {
                            println!("mark mail as see: {}", &mail.uid);
                            session.store(&mail.uid.to_string(), r"+FLAGS \Seen");
                        }
                    }
                }
                Err(e) => println!("Failed to fetch: {}", e),
            }
        }

        session.logout().unwrap();

        if DEFAULT.service {
            sleep(time::Duration::new(DEFAULT.sleep_time * 60, 0));
        } else {
            break;
        }
    };
}
