use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::NaiveDateTime;
use notmuch;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Message {
    id: String,
    from: Option<String>,
    to: Option<String>,
    subject: Option<String>,
    date: Option<NaiveDateTime>,
    tags: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Thread {
    id: String,
    subject: String,
    authors: Vec<String>,
    tags: Vec<String>,
    messages: Vec<Message>,
    unread: bool,
}

#[derive(Debug)]
pub struct MailStore {
    database_path: PathBuf,
    config_path: PathBuf,
}

#[derive(Serialize, Debug)]
pub struct SingleMessage {
    id: String,
    tags: Vec<String>,
    filename: String,
    json: String,
}

#[derive(Serialize, Debug)]
pub struct SingleMessageBrief {
    from: String,
    subject: String,
}

impl MailStore {
    pub fn new(path: impl AsRef<Path>, config_path: impl AsRef<Path>) -> MailStore {
        MailStore {
            database_path: path.as_ref().to_path_buf(),
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    pub fn query(&self, query: &str, filtered_authors: &HashSet<String>) -> (Vec<Thread>, bool) {
        let database = self.open_db();
        let query = database.create_query(query).unwrap();
        query.set_sort(notmuch::Sort::NewestFirst);
        let mut result = Vec::new();
        let mut count = 0;
        let mut more = false;
        for thread in query.search_threads().unwrap() {
            let mut t = Vec::new();
            for message in thread.messages() {
                t.push(Message {
                    id: message.id().to_string(),
                    from: message
                        .header("from")
                        .unwrap_or(None)
                        .map(|x| x.to_string()),
                    to: message.header("to").unwrap_or(None).map(|x| x.to_string()),
                    subject: message
                        .header("subject")
                        .unwrap_or(None)
                        .map(|x| x.to_string()),
                    date: NaiveDateTime::from_timestamp_opt(message.date(), 0),
                    tags: message.tags().collect(),
                });
                count += 1;
            }
            let tags: Vec<String> = thread.tags().collect();
            let unread = tags.contains(&"unread".to_string());
            result.push(Thread {
                id: thread.id().to_string(),
                subject: thread.subject().to_string(),
                authors: thread
                    .authors()
                    .iter()
                    .filter(|x| !filtered_authors.contains(*x))
                    .map(|x| x.to_string())
                    .collect(),
                tags,
                messages: t,
                unread,
            });
            if count > 100 {
                more = true;
                break;
            }
        }
        (result, more)
    }

    fn open_db(&self) -> notmuch::Database {
        notmuch::Database::open_with_config(
            Some(&self.database_path),
            notmuch::DatabaseMode::ReadWrite,
            Some(&self.config_path),
            None,
        )
        .expect("failed to open notmuch database")
    }

    pub fn get_message(&self, msg_id: &str) -> anyhow::Result<SingleMessage> {
        let database = self.open_db();
        let message = database.find_message(msg_id)?.context("not found")?;
        let raw = std::fs::read_to_string(message.filename())?;
        let parsed = mail_parser::Message::parse(raw.as_bytes()).unwrap();

        let json: String = serde_json::to_string_pretty(&parsed).unwrap();

        Ok(SingleMessage {
            id: msg_id.to_string(),
            json,
            tags: message.tags().collect(),
            filename: message.filename().to_string_lossy().to_string(),
        })
    }

    pub fn get_message_brief(&self, msg_id: &str) -> anyhow::Result<SingleMessageBrief> {
        let database = self.open_db();
        let message = database.find_message(msg_id)?.context("not found")?;
        return Ok(SingleMessageBrief {
            from: message
                .header("from")
                .unwrap_or(None)
                .map(|x| x.to_string())
                .unwrap_or_else(|| "".to_string()),
            subject: message
                .header("subject")
                .unwrap_or(None)
                .map(|x| x.to_string())
                .unwrap_or_else(|| "".to_string()),
        });
    }

    pub fn add_tags(&self, msg_id: &str, tags: &Vec<String>) -> anyhow::Result<()> {
        let database = self.open_db();

        let message = database.find_message(msg_id)?.context("not found")?;
        message.freeze()?;
        for tag in tags {
            message.add_tag(tag)?;
        }
        message.thaw()?;
        Ok(())
    }
    pub fn remove_tags(&self, msg_id: &str, tags: &Vec<String>) -> anyhow::Result<()> {
        let database = self.open_db();

        let message = database.find_message(msg_id)?.context("not found")?;
        message.freeze()?;
        for tag in tags {
            message.remove_tag(tag)?;
        }
        message.thaw()?;
        Ok(())
    }

    pub fn toggle_tag(&self, msg_id: &str, tag: &String) -> anyhow::Result<()> {
        use itertools::Itertools;
        let database = self.open_db();

        let message = database.find_message(msg_id)?.context("not found")?;
        message.freeze()?;
        if message.tags().contains(tag) {
            message.remove_tag(tag)?;
        } else {
            message.add_tag(tag)?;
        }
        message.thaw()?;
        Ok(())
    }

    pub fn store_attachments(&self, msg_id: &str, path: &PathBuf) -> anyhow::Result<()> {
        use mail_parser::MimeHeaders;
        let database = self.open_db();
        let message = database.find_message(msg_id)?.context("not found")?;
        let raw = std::fs::read(message.filename())?;
        let message = mail_parser::Message::parse(&raw[..]).unwrap();
        for (ii, attachment) in message.attachments().enumerate() {
            let name = attachment
                .attachment_name()
                .map(|x| x.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "attachment-{}.{}",
                        ii,
                        if attachment.is_text() {
                            "txt"
                        } else if attachment.is_text_html() {
                            "html"
                        } else {
                            "bin"
                        }
                    )
                });
            if name.contains("/") {
                println!("skipping attachment {}, contained /", name);
            }
            let save_name = path.join(name);
            std::fs::write(save_name, attachment.contents())?;
        }

        Ok(())
    }

    pub fn new_mail(
        &mut self,
        prev: Option<String>,
        accounts: Vec<crate::storage::MailAccount>,
    ) -> (PathBuf, String) {
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        //create a maildir filename from unix time, current process identifier, hostename,
        //concactenated with a .
        let maildir_mail_filename = format!(
            "{}.{}.florg.{}.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            std::process::id(),
            hostname,
            "mail"
        );
        let from = format!(
            "From: {}\n",
            accounts
                .get(0)
                .map(|x| &x.sender[..])
                .unwrap_or("Configure mail accounts in settings!")
        );
        let mut content = match prev {
            Some(_) => from, //todo
            None => from,
        };
        content.push_str("\n");
        let maildir_file_path = self
            .database_path
            .join(".Drafts")
            .join("cur")
            .join(maildir_mail_filename);
        std::fs::write(&maildir_file_path, &content).expect("Failed to write mail draft file");
        (maildir_file_path, content)
    }
}
