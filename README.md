This program was create to post mails received from different mailing list into different Slack channels, so that our members could read and comment on the mails with in our internal communication platform. This program is make to run on linux, don't know if it runs on other operation systems.

## Config
The first time you run the program, it will create a config example in the location `~/.config/imap2slack/default.toml`.
Edit the `default.toml` config file. Config the following information

### defualt.toml
- `service` If `false` it only checks for mails ones. If `true` it continues to check for mails.
- `sleep` number of minutes to wait before checking for new mail again.
- `mark_mail_as_seen` If `false` the mails will not be marked as read. If `true` the mails will be marked as read.

#### [mail]
- `ìmap` The url for the imap server
- `port` The port no. for the imap server
- `username` The username
- `password` The password

#### [slack]
- `webhook` Enter the url for the Slack inbound hook
- `username` What should the username be?
- `emoji` Select a default or custom emoji

#### [[publish]]
- `channel` The name of the channel that you want to post the mail in 
- `mailbox` The dir to the mail box (no spaces)
- `filter` The name of the filter (optional)

### filters.toml
Both `contains` and `does_not_contains` have to be satisfied before a mail is posted.
If a filter is mentioned in `default.toml`, but does not exist in `filters.toml` a empty instance are created in the config file.

#### [filter.SOME_NAME]
- `case_sensitive` if case sensitive  `true` / `false`
- `contains` a toml array of words that the subject of the mail should contain
- `does_not_contains` a toml array of words that the subject of the mail should not contain
