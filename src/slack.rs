extern crate slack_hook;

use self::slack_hook::{Slack, PayloadBuilder, AttachmentBuilder, Result};

use imap_extention::fetch::*;
use config::CONFIG;

pub fn post_mails(mails: &Vec<Mail>, username: &str, channel: &str, emoji: &str) -> Result<()> {
    let slack = Slack::new(CONFIG.slack.webhook.as_str()).expect("Failed at connecting to the Slack Webhook");

    for mail in mails {
        let p = PayloadBuilder::new()
            .attachments(
                vec![AttachmentBuilder::new("")
                    .pretext(format!("From:\t\t{}\nTo:\t\t\t{}", mail.from, mail.to))
                    .title(mail.subject.clone())
                    .text(mail.text.clone())
                    .build().unwrap()])
            .channel(CONFIG.slack.channel.clone())
            .username(CONFIG.slack.username.clone())
            .icon_emoji(format!(":{}:", &CONFIG.slack.emoji))
            .build()
            .unwrap();

        match slack.send(&p) {
            Ok(()) => println!("ok"),
            Err(x) => return Err(x)
        }
    }
    Ok(())
}