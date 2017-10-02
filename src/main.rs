extern crate imap;
extern crate openssl;
extern crate regex;
extern crate quoted_printable;
#[macro_use]
extern crate lazy_static;

extern crate toml;
#[macro_use]
extern crate serde_derive;

use std::string::String;
use std::thread::sleep;
use std::time::Duration;

use openssl::ssl::{SslConnectorBuilder, SslMethod};
use imap::client::Client;

mod imap_extention;
use imap_extention::search::*;
use imap_extention::fetch::*;
use imap_extention::path::{Path, PathFrom};

mod config;
use config::*;

mod slack;
use slack::post_mails;

// To connect to the gmail IMAP server with this you will need to allow unsecure apps access.
// See: https://support.google.com/accounts/answer/6010255?hl=en
// Look at the gmail_oauth2.rs example on how to connect to a gmail server securely.
fn main() {
    let domain: &str = &CONFIG.mail.imap;
    let port: u16 = CONFIG.mail.port.clone();
    let socket_addr = (domain, port);

    loop {
        let ssl_connector = SslConnectorBuilder::new(SslMethod::tls()).unwrap().build();
        let mut imap_socket = Client::secure_connect(socket_addr, domain, ssl_connector).unwrap();
        imap_socket.login(&CONFIG.mail.username, &CONFIG.mail.password).unwrap();

        for p in &CONFIG.publish {
            let path = Path::new(&p.mailbox);
            let mut uids: Vec<usize> = Vec::new();

            println!("--- mailbox - {} ---", &path.as_str());
            match imap_socket.select_from(&path) {
                Ok(mailbox) => println!("Selected mailbox"),
                Err(e) => println!("Error selecting INBOX: {}", e),
            };

            println!("--- Search ---");
            match imap_socket.search(vec![SEARCH::UNSEEN]) {
                Ok(u) => {
                    println!("* SEARCH: {:?}", u);
                    uids = u
                }
                Err(e) => println!("Failed in searching for mail: {}", e),
            };

            println!("--- Fetch ---");

            let fetch = imap_socket.fetch_ext(&uids);
            match fetch {
                Ok(mails) => {
                    for mail in &mails {

                        post_mails(mail, &p.channel);
                    }
                    for mail in mails {
                        imap_socket.store(&mail.uid.to_string(), r"+FLAGS \Seen");
                    }
                },
                Err(e) => println!("Failed to fetch: {}", e),
            }
        }

        imap_socket.logout().unwrap();

        if CONFIG.service {
            sleep(Duration::new(CONFIG.sleep_time * 60, 0));
        } else {
            break;
        }
    };
}