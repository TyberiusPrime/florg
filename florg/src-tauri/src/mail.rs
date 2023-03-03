use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use notmuch;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Message {
    from: Option<String>,
    to: Option<String>,
    subject: Option<String>,
    date: Option<NaiveDateTime>,
    tags: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Thread {
    subject: String,
    authors: Vec<String>,
    tags: Vec<String>,
    messages: Vec<Message>,
}

#[derive(Debug)]
pub struct MailStore {
    database_path: PathBuf,
    config_path: PathBuf,
}

impl MailStore {
    pub fn new(path: impl AsRef<Path>, config_path: impl AsRef<Path>) -> MailStore {
        MailStore {
            database_path: path.as_ref().to_path_buf(),
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    pub fn query(&self, query: &str) -> Vec<Thread> {
        let database = notmuch::Database::open_with_config(
            Some(&self.database_path),
            notmuch::DatabaseMode::ReadOnly,
            Some(&self.config_path),
            None,
        )
        .expect("failed to open notmuch database");
        let query = database.create_query(query).unwrap();
        query.set_sort(notmuch::Sort::NewestFirst);
        let mut result = Vec::new();
        let mut count = 0;
        for thread in query.search_threads().unwrap() {
            let mut t = Vec::new();
            for message in thread.messages() {
                t.push(Message {
                    from: message
                        .header("from").unwrap_or(None).map(|x| x.to_string()),
                    to: message
                        .header("to").unwrap_or(None).map(|x| x.to_string()),
                    subject: message
                        .header("subject").unwrap_or(None).map(|x| x.to_string()),
                    date: NaiveDateTime::from_timestamp_opt(message.date(), 0),
                    tags: message.tags().collect(),
                });
                count += 1;
            }
            result.push(Thread {
                subject: thread.subject().to_string(),
                authors: thread.authors(),
                tags: thread.tags().collect(),
                messages: t,
            });
            if count > 100 {
                break;
            }
        }
        result
    }
}
