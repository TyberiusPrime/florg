#![allow(dead_code)]
use serde::{ser::Serializer, Deserialize, Serialize};
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Header {
    pub title: String,
    pub first_paragraph: String,
}

#[derive(Debug, Clone, Eq, Serialize)]
pub(crate) struct Node {
    pub path: String,
    pub header: Header,
    pub raw: String,
    //children: Vec<Node>,
}

#[derive(Debug)]
pub(crate) struct Storage {
    data_path: PathBuf,
    git_binary: String,
    nodes: Vec<Node>,
    pub settings: HashMap<String, String>,
}

impl Storage {
    pub(crate) fn new(data_path: PathBuf, git_binary: String) -> Storage {
        let settings = Self::load_settings(&data_path).unwrap_or_else(|_| HashMap::new());
        let mut s = Storage {
            data_path,
            nodes: Vec::new(),
            git_binary,
            settings,
        };
        s.reload();
        s
    }

    pub fn reload(&mut self) {
        let nodes = Self::parse_path(&self.data_path, None, None);
        self.nodes = nodes;
    }

    fn load_settings(data_path: &PathBuf) -> Result<HashMap<String, String>> {
        use toml::Table;
        let raw = std::fs::read_to_string(data_path.join("settings.toml"))?;
        let parsed = raw.parse::<Table>()?;
        let mut res = HashMap::new();
        for (key, value) in parsed.iter() {
            if let Some(v) = value.as_str() {
                res.insert(key.to_string(), v.to_string());
            }
        }
        Ok(res)
    }

    pub fn store_settings(&self) {
        let out = toml::to_string(&self.settings).unwrap();
        std::fs::write(self.data_path.join("settings.toml"), out).expect("saving settings failed");
    }

    fn parse_path(
        data_path: &PathBuf,
        old_root: Option<Node>,
        update_nodes: Option<Vec<String>>,
    ) -> Vec<Node> {
        let mut nodes = Vec::new();
        for entry in WalkDir::new(data_path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_type().is_dir()
                    || entry
                        .file_name()
                        .to_str()
                        .map(|s| s == "node.florg")
                        .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir() {
//                dbg!(data_path);
 //               dbg!(entry.path());
                let path = entry
                    .path()
                    .parent()
                    .unwrap()
                    .strip_prefix(data_path)
                    .unwrap()
                    .to_string_lossy();
                let path = path.replace("/", "");
                nodes.push(Node::parse(path, entry.path()));
            }
        }
        nodes
    }

    pub(crate) fn get_node(&self, path: &str) -> Option<&Node> {
        self.nodes.iter().filter(|n| n.path == path).next()
    }

    pub(crate) fn levels(&self, path: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let mut p = path.to_owned();
        while !p.is_empty() {
            let entry = self.get_node(&p);
            let letter = p.pop().unwrap().to_string();
            res.push((
                letter,
                entry.map(|x| &x.header.title[..]).unwrap_or("").to_string(),
            ));
        }
        res.reverse();
        res
    }

    pub(crate) fn children_for(&self, path: &str) -> Vec<&Node> {
        let lp = path.len() + 1;
        let mut res: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| n.path.len() == lp && n.path.starts_with(path))
            .collect();
        res.sort();
        res
    }

    pub(crate) fn replace_node(&mut self, node: Node) {
        self.nodes.retain(|x| x.path != node.path);

        let mut filename = self.data_path.clone();
        for p in node.path.chars() {
            filename.push(p.to_string());
        }
        std::fs::create_dir_all(&filename).expect("failed to create directory");
        filename.push("node.florg");

        let existed = filename.exists();
        let msg = if existed {
            format!("Changed node {} '{}'", node.path, node.header.title)
        } else {
            format!("Added node {} '{}'", node.path, node.header.title)
        };

        std::fs::write(filename, node.raw.trim()).expect("Failed to write file");
        self.add_and_commit(&msg);
        self.nodes.push(node);
    }

    fn add_and_commit(&self, msg: &str) {
        std::process::Command::new(&self.git_binary)
            .arg("add")
            .arg(".")
            .current_dir(&self.data_path)
            .status()
            .expect("git add failed");
        std::process::Command::new(&self.git_binary)
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(&self.data_path)
            .status()
            .expect("git add failed");
    }
}

impl Node {
    pub fn new(path: &str, raw: &str) -> Node {
        let header = Self::extract_header(raw);
        Node {
            path: path.to_string(),
            header,
            raw: raw.to_string(),
        }
    }

    fn parse(path: String, file_path: &Path) -> Node {
        let raw = std::fs::read_to_string(file_path).unwrap();
        let header = Self::extract_header(&raw);
        Node { path, header, raw }
    }
    fn extract_header(contents: &str) -> Header {
        let title = match contents.split_once("\n") {
            Some((first_line, _)) => first_line,
            _ => contents,
        };
        let first_para = match contents.split_once("\n\n") {
            Some((first_para, _)) => {
                if first_para.contains('\n') {
                    first_para.strip_prefix(title).unwrap().trim()
                } else {
                    let mut it = contents.split("\n\n");
                    it.next();
                    it.next().unwrap().trim()
                }
            }
            _ => contents,
        };
        Header {
            title: title.to_string(),
            first_paragraph: first_para.to_string(),
        }
    }

    pub fn content(&self) -> &str {
        &self.raw
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
