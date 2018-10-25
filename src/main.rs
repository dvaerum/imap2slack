extern crate imap;
extern crate native_tls;
extern crate regex;
extern crate quoted_printable;
#[macro_use]
extern crate lazy_static;

extern crate toml;
#[macro_use]
extern crate serde_derive;


use std::thread::sleep;
use std::time::Duration;

use native_tls::TlsConnector;
use imap::client;
use imap::error::Error;

mod imap_extention;
use imap_extention::search::*;
use imap_extention::fetch::*;
use imap_extention::path::{Path, PathFrom};

mod config;
use config::DEFAULT;

mod slack;
use slack::post_mails;

// To connect to the gmail IMAP server with this you will need to allow unsecure apps access.
// See: https://support.google.com/accounts/answer/6010255?hl=en
// Look at the gmail_oauth2.rs example on how to connect to a gmail server securely.
fn main() {
    for publish in &DEFAULT.publish {
        &publish.filter();
    }


    let domain: &str = &DEFAULT.mail.imap;
    let port: u16 = DEFAULT.mail.port.clone();
    let socket_addr = (domain, port);

    loop {
        let ssl_connector = TlsConnector::builder().build().unwrap();
        let mut imap_socket: client::Client<native_tls::TlsStream<std::net::TcpStream>>; // = Client::secure_connect(socket_addr, domain, ssl_connector).unwrap();
        match client::secure_connect(socket_addr, domain, &ssl_connector) {
            Ok(mut sock) => {
                sock.debug = DEFAULT.debug_imap();
                imap_socket = sock
            },
            Err(e) => {
                match e {
                    // An `io::Error` that occurred while trying to read or write to a network stream.
                    Error::Io(io_error) => {
                        println!("{:?}", io_error);
                        ::std::process::exit(1);
                    },
                    // An error from the `native_tls` library during the TLS handshake.
                    Error::TlsHandshake(tls_handshake_error) => {
                        println!("{:?}", tls_handshake_error);
                        ::std::process::exit(1);
                    },
                    // An error from the `native_tls` library while managing the socket.
                    Error::Tls(tls_error) => {
                        println!("{:?}", tls_error);
                        ::std::process::exit(1);
                    },
                    // A BAD response from the IMAP server.
                    Error::BadResponse(response) => {
                        println!("{:?}", response);
                        ::std::process::exit(1);
                    },
                    // A NO response from the IMAP server.
                    Error::NoResponse(response) => {
                        println!("{:?}", response);
                        ::std::process::exit(1);
                    },
                    // The connection was terminated unexpectedly.
                    Error::ConnectionLost => {
                        println!("Connection to the server has been lost");
                        ::std::process::exit(1);
                    },
                    // Error parsing a server response.
                    Error::Parse(parse_error) => {
                        println!("{:?}", parse_error);
                        ::std::process::exit(1);
                    },
                    // Error validating input data
                    Error::Validate(ValidateError) => {
                        println!("{:?}", ValidateError);
                        ::std::process::exit(1);
                    }
                    // Error appending a mail
                    Error::Append => {
                        println!("Error appending a mail");
                        ::std::process::exit(1);
                    },
                }
            }
        };

        let mut session = imap_socket.login(&DEFAULT.mail.username, &DEFAULT.mail.password).unwrap();

        for publish in &DEFAULT.publish {
            let path = Path::new(&publish.mailbox);
            let mut uids: Vec<usize> = Vec::new();

            println!("--- mailbox - {} ---", &path.as_str());
            match session.select_from(&path) {
//                Ok(mailbox) => println!("Selected mailbox - '{}'", mailbox),
                Ok(mailbox) => (),
                Err(e) => println!("Error selecting INBOX: {}", e),
            };

            match session.search2(vec![SEARCH::UNSEEN]) {
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
            println!("{:?}", &fetch);
            match fetch {
                Ok(mails) => {
                    for mail in &mails {
                        match &publish.filter() {
                            &Some(filter) => {
                                if filter.check(&mail.subject) {
                                    post_mails(mail, &publish.channel);
                                }
                            },
                            &None => {
                                post_mails(mail, &publish.channel);
                            }
                        }

                        if DEFAULT.mark_mail_as_seen() {
                            session.store(&mail.uid.to_string(), r"+FLAGS \Seen");
                        }
                    }
                },
                Err(e) => println!("Failed to fetch: {}", e),
            }
        }

        session.logout().unwrap();

        if DEFAULT.service {
            sleep(Duration::new(DEFAULT.sleep_time * 60, 0));
        } else {
            break;
        }
    };
}
