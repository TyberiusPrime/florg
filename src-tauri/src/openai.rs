use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml_edit;

#[derive(Debug)]
pub struct ChatGPT {
    api_key: String,
    data_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub title: Option<String>,
    pub date: chrono::NaiveDateTime,
    pub prompt: String,
    pub messages: Vec<(String, serde_json::Value)>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationListEntry {
    pub title: Option<String>,
    pub date: chrono::NaiveDateTime,
    pub filename: String,
}

const STORAGE_PREFIX: &'static str = "chatgpt";

impl ChatGPT {
    pub fn new(api_key: String, data_path: PathBuf) -> Self {
        Self { api_key, data_path }
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }
    fn get_toml(&self) -> Result<toml_edit::Document, anyhow::Error> {
        let filename = self.data_path.join(STORAGE_PREFIX).join("prompts.toml");
        let raw = std::fs::read_to_string(filename)?;
        let parsed = raw.parse::<toml_edit::Document>()?;
        Ok(parsed)
    }
    pub fn get_all_prompts(
        &self,
    ) -> Result<HashMap<String, HashMap<String, String>>, anyhow::Error> {
        let parsed = self.get_toml()?;
        let mut res = HashMap::new();
        for (k, v) in parsed.iter() {
            if let Some(table) = v.as_table() {
                res.insert(
                    k.to_string(),
                    table
                        .iter()
                        .map(|(k, v)| (Some((k.to_string(), v.as_str()?.to_string()))))
                        .filter_map(|x| x)
                        .collect(),
                );
            }
        }
        Ok(res)
    }

    pub fn update_prompts(&self, key: &str, prompts: HashMap<&str, &str>) -> Result<()> {
        let mut parsed = self.get_toml().unwrap_or_default();
        let mut t = toml_edit::Table::new();
        for (k, v) in prompts.iter() {
            t[k] = toml_edit::value(*v);
        }
        parsed[key] = toml_edit::Item::Table(t);
        let out = parsed.to_string();
        std::fs::create_dir_all(self.data_path.join(STORAGE_PREFIX))?;
        std::fs::write(
            self.data_path.join(STORAGE_PREFIX).join("prompts.toml"),
            out,
        )?;
        Ok(())
    }

    pub fn list_conversations(&self) -> Result<Vec<ConversationListEntry>> {
        let path = self.data_path.join(STORAGE_PREFIX).join("conversations");
        std::fs::create_dir_all(&path)?;
        let mut res = Vec::new();
        for entry in glob::glob(&path.join("*.json").to_string_lossy())? {
            let entry = entry?;
            let filename = entry
                .file_name()
                .context("no filename on .json file?")?
                .to_string_lossy();
            let convo = self.get_conversation(&filename)?;
            res.push(ConversationListEntry {
                title: convo.title,
                date: convo.date,
                filename: filename.to_string(),
            });
        }
        Ok(res)
    }

    pub fn get_conversation(&self, filename: &str) -> Result<Conversation> {
        let path = self
            .data_path
            .join(STORAGE_PREFIX)
            .join("conversations")
            .join(filename);
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save_conversation(&self, filename: &str, conversation: &Conversation) -> Result<()> {
        let path = self
            .data_path
            .join(STORAGE_PREFIX)
            .join("conversations")
            .join(filename);
        std::fs::create_dir_all(&path.parent().unwrap())?;
        let out = serde_json::to_string_pretty(&conversation)?;
        std::fs::write(path, out)?;
        Ok(())
    }
}
