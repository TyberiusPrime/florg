#![allow(dead_code)]
#![allow(unused_imports)]
use crate::openai;
use anyhow::{bail, Context, Result};
use once_cell::unsync::Lazy;
use regex::Regex;
use serde::{ser::Serializer, Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Header {
    pub title: String,
    pub first_paragraph: String,
    pub has_more_content: bool,
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
    pub data_path: PathBuf,
    git_binary: String,
    nodes: Vec<Node>,
    pub settings: toml_edit::Document,

    pub(crate) chatgpt: Option<openai::ChatGPT>,
}

pub struct MailAccount {
    pub sender: String,
    pub addresses: Vec<String>,
}

pub const FLORG_FILENAME: &'static str = "node.adoc";
pub const FLORG_CACHE_FILENAME: &'static str = "node.cache";
pub const FLORG_SUFFIX: &'static str = ".adoc";

impl Storage {
    fn get_chatgtp_key(settings: &toml_edit::Document) -> Option<String> {
        settings
            .get("chatgpt")?
            .get("api_key")?
            .as_str()
            .map(|x| x.to_string())
    }
    pub(crate) fn new(data_path: PathBuf, git_binary: String) -> Storage {
        let settings =
            Self::load_settings(&data_path, None).unwrap_or_else(|_| toml_edit::Document::new());
        //todo: make this robust
        let chatgpt = Storage::get_chatgtp_key(&settings)
            .map(|s| openai::ChatGPT::new(s.to_string(), data_path.clone()));

        let mut s = Storage {
            data_path,
            nodes: Vec::new(),
            git_binary,
            settings,
            chatgpt,
        };
        s.reload();
        s
    }

    pub fn reload(&mut self) {
        let nodes = Self::parse_path(&self.data_path);
        self.nodes = nodes;
        //print a sorted list of the nodes path...
        /* let mut paths: Vec<_> = self.nodes.iter().map(|n| n.path.clone()).collect();
        paths.sort();
        dbg!(paths); */
        let settings = Self::load_settings(&self.data_path, None)
            .unwrap_or_else(|_| toml_edit::Document::new());

        let chatgpt = Storage::get_chatgtp_key(&settings)
            .map(|s| openai::ChatGPT::new(s.to_string(), self.data_path.clone()));
        self.settings = settings;
        self.chatgpt = chatgpt;
    }

    pub fn settings_filename(data_path: &PathBuf) -> PathBuf {
        data_path.join("settings.toml")
    }

    pub fn load_settings(data_path: &PathBuf, raw: Option<String>) -> Result<toml_edit::Document> {
        let raw = match raw {
            Some(raw) => raw,
            None => std::fs::read_to_string(Self::settings_filename(data_path))?,
        };
        let parsed = raw.parse::<toml_edit::Document>()?;
        Ok(parsed)
    }

    pub fn store_settings(&self) {
        let out = self.settings.to_string();
        std::fs::write(self.data_path.join("settings.toml"), out).expect("saving settings failed");
    }

    fn parse_path(
        data_path: &PathBuf,
        //old_root: Option<Node>,
        //update_nodes: Option<Vec<String>>,
    ) -> Vec<Node> {
        let mut nodes = Vec::new();
        for entry in WalkDir::new(data_path)
            .into_iter()
            .filter_entry(|entry| {
                entry.file_type().is_dir()
                    || entry
                        .file_name()
                        .to_str()
                        .map(|s| s == FLORG_FILENAME)
                        .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir() {
                //                dbg!(data_path);
                let path = entry
                    .path()
                    .parent()
                    .unwrap()
                    .strip_prefix(data_path)
                    .unwrap()
                    .to_string_lossy();
                let path = path.replace("/", "");
                //dbg!(entry.path(), &path);
                nodes.push(Node::parse(path, entry.path()));
            }
        }
        //nodes that have no FLORG_FILENAME but child nodes need to be patched in.
        let mut paths: HashSet<_> = nodes.iter().map(|n| n.path.clone()).collect();
        let paths2 = paths.clone();
        for p in paths2.iter() {
            if p.len() > 0 {
                if !paths.contains(&p[..p.len() - 1]) {
                    let path = &p[..p.len() - 1];
                    let node = Node {
                        path: path.to_string(),
                        header: Header {
                            title: "(empty node)".to_string(),
                            first_paragraph: "".to_string(),
                            has_more_content: false,
                        },
                        raw: "".to_string(),
                    };
                    nodes.push(node);
                    paths.insert(path.to_string());
                }
            }
        }
        nodes.sort_by(|a, b| a.path.cmp(&b.path));
        nodes
    }

    pub(crate) fn get_node(&self, path: &str) -> Option<&Node> {
        //todo: replace with binary search
        self.nodes.iter().filter(|n| n.path == path).next()
    }

    pub(crate) fn get_node_mut(&mut self, path: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().filter(|n| n.path == path).next()
    }

    pub(crate) fn delete_node(&mut self, path: &str) -> Result<(), String> {
        let node = self.get_node(path).ok_or("node not found")?;
        let file_path = node.dirname(&self.data_path);
        std::fs::remove_dir_all(file_path).map_err(|e| e.to_string())?;
        self.add_and_commit(&format!("Deleted node {path} and children"));
        self.nodes.retain(|n| !n.path.starts_with(path));
        Ok(())
    }

    pub(crate) fn move_node(&mut self, org_path: &str, new_path: &str, commit: bool) -> Result<()> {
        if self.get_node(new_path).is_some() {
            bail!("new path already exists".to_string());
        }
        let new_file_path = Node::dirname_from_path(&self.data_path, new_path);
        let node = self.get_node(org_path).context("node not found")?;
        let file_path = node.dirname(&self.data_path);
        println!("moving {:?} {:?}", &file_path, &new_file_path);
        std::fs::rename(file_path, new_file_path)?;
        self.rename_all_children(org_path, new_path);
        if commit {
            self.add_and_commit(&format!("moved node {org_path} to {new_path}"));
        }
        Ok(())
    }

    pub(crate) fn swap_node_with_previous(&mut self, path: &str) -> Result<()> {
        if path.ends_with("A") {
            bail!("Can not swap first node");
        }
        if self.get_node("!").is_some() {
            bail!("Can not swap, tempmove node ('!') already exists. Manually clean up the storage directory.");
        }
        let prev = self.find_previous_sibling(path).context("No prev node")?;
        self.move_node(&prev, "!", false)
            .context("failed to move prev to tempmove")?;
        self.move_node(path, &prev, false)
            .context("failed to move path to prev")?;
        self.move_node("!", &path, false)
            .context("failed to move tempmove into prev")?;
        self.make_nodes_sorted();
        self.add_and_commit(&format!("Swapped nodes up: {} and {}", path, prev));
        Ok(())
    }

    pub(crate) fn swap_node_with_next(&mut self, path: &str) -> Result<()> {
        if path.ends_with("Z") {
            bail!("Can not swap last node");
        }
        if self.get_node("!").is_some() {
            bail!("Can not swap, tempmove node ('!') already exists. Manually clean up the storage directory.");
        }
        let next = self.find_next_sibling(path).context("No next node")?;
        self.move_node(&next, "!", false)
            .context("failed to move next to tempmove")?;
        self.move_node(path, &next, false)
            .context("failed to move path to next")?;
        self.move_node("!", &path, false)
            .context("failed to move tempmove into next")?;
        self.make_nodes_sorted();
        self.add_and_commit(&format!("Swapped nodes down: {} and {}", path, next));
        Ok(())
    }

    fn make_nodes_sorted(&mut self) {
        self.nodes.sort_by(|a, b| a.path.cmp(&b.path));
    }

    fn find_previous_sibling(&self, path: &str) -> Option<String> {
        let node_idx = self
            .nodes
            .binary_search_by(|node| node.path.as_str().cmp(path))
            .ok()?;
        //find the first previous node that is at the same level (same path.len)
        //if they share their -1 prefix, return it
        for node in self.nodes[..node_idx].iter().rev() {
            println!(
                "Looking at {} {:?}",
                node.path,
                node.path.len().cmp(&path.len())
            );
            match node.path.len().cmp(&path.len()) {
                Ordering::Less => return None,
                Ordering::Equal => {
                    if node.path[..path.len() - 1] == path[..path.len() - 1] {
                        return Some(node.path.clone());
                    } else {
                        return None;
                    }
                }
                Ordering::Greater => {}
            }
        }
        None
    }

    fn find_next_sibling(&self, path: &str) -> Option<String> {
        let node_idx = self
            .nodes
            .binary_search_by(|node| node.path.as_str().cmp(path))
            .ok()?;
        //find the first previous node that is at the same level (same path.len)
        //if they share their -1 prefix, return it
        for node in self.nodes[node_idx + 1..].iter() {
            println!(
                "Looking at {} {:?}",
                node.path,
                node.path.len().cmp(&path.len())
            );
            match node.path.len().cmp(&path.len()) {
                Ordering::Less => return None,
                Ordering::Equal => {
                    if node.path[..path.len() - 1] == path[..path.len() - 1] {
                        return Some(node.path.clone());
                    } else {
                        return None;
                    }
                }
                Ordering::Greater => {}
            }
        }
        None
    }

    fn rename_all_children(&mut self, org_path: &str, new_path: &str) {
        //todo: fix all the links
        for node in self.nodes.iter_mut() {
            if node.path.starts_with(org_path) {
                let suffix = &node.path[org_path.len()..];
                println!(
                    "Renaming {} to {}",
                    node.path,
                    format!("{}{}", new_path, suffix)
                );
                node.path = format!("{}{}", new_path, suffix);
            }
        }
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

    pub(crate) fn children_paths_for(&self, path: &str) -> Vec<String> {
        let lp = path.len() + 1;
        let mut res: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| n.path.len() == lp && n.path.starts_with(path))
            .map(|n| n.path.clone())
            .collect();
        res.sort();
        res
    }

    fn get_letter_set(start: char, stop: char) -> HashSet<char> {
        let mut res = HashSet::new();
        for c in start..=stop {
            res.insert(c);
        }
        res
    }

    pub(crate) fn find_next_empty_child(&self, path: &str) -> String {
        let letters = Self::get_letter_set('A', 'Y');
        let used: HashSet<char> = self
            .children_for(path)
            .iter()
            .map(|node| node.path.chars().last().unwrap())
            .collect();
        let remaining = letters.difference(&used);
        match remaining.min() {
            Some(lowest) => format!("{path}{lowest}"),
            None => self.find_next_empty_child(&format!("{path}Z")),
        }
    }

    pub(crate) fn replace_node(&mut self, node: Node, commit: bool) {
        self.nodes.retain(|x| x.path != node.path);

        let mut filename = node.dirname(&self.data_path);
        std::fs::create_dir_all(&filename).expect("failed to create directory");
        filename.push(FLORG_FILENAME);

        let existed = filename.exists()
            && (std::fs::read_to_string(&filename).unwrap_or("".to_string()) != "(placeholder)");
        let msg = if existed {
            format!("Changed node {} '{}'", node.path, node.header.title)
        } else {
            format!("Added node {} '{}'", node.path, node.header.title)
        };

        std::fs::write(filename, node.raw.trim()).expect("Failed to write file");
        if commit {
            self.add_and_commit(&msg);
        }
        self.nodes.push(node);
    }

    pub(crate) fn remove_placeholder(&mut self, path: &str) {
        let node = self.get_node(path);
        let mut remove_path = None;
        if let Some(node) = node {
            if node.raw == "(placeholder)" {
                let rd =
                    std::fs::read_dir(&node.dirname(&self.data_path)).expect("Failed to read dir");
                if rd
                    .filter_map(|x| x.ok())
                    .filter(|entry| {
                        entry
                            .path()
                            .file_name()
                            .unwrap_or_else(|| OsStr::new(""))
                            .to_string_lossy()
                            != FLORG_FILENAME
                    })
                    .next()
                    .is_none()
                {
                    remove_path = Some(node.path.to_string())
                }
            }
        }
        if let Some(remove_path) = remove_path {
            self.remove_node(&remove_path);
        }
    }

    pub fn remove_node(&mut self, path: &str) {
        let filename: PathBuf = Node::dirname_from_path(&self.data_path, path);
        std::fs::remove_dir_all(filename).expect("Failed to unlink file");
        self.nodes.retain(|x| x.path != path);
        //copilot: unlink  filename
    }

    pub fn add_and_commit(&self, msg: &str) {
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

    pub(crate) fn set_cached_node(&mut self, path: &str, raw: &str, to_cache: &str) -> Result<()> {
        let node_path = Node::dirname_from_path(&self.data_path, path).join(FLORG_CACHE_FILENAME);
        let hash = sha256::digest(raw.as_bytes()).to_string();
        let output = format!("{}\n{}", hash, to_cache);
        Ok(std::fs::write(node_path, output)?)
    }

    pub(crate) fn get_cached_node(&mut self, path: &str) -> Option<String> {
        let node = self.get_node(path)?;
        let hash = sha256::digest(node.raw.as_bytes()).to_string();
        let input = std::fs::read_to_string(
            Node::dirname_from_path(&self.data_path, path).join(FLORG_CACHE_FILENAME),
        )
        .ok()?;
        //dbg!("Read cache");
        let (stored_hash, content) = input.split_once("\n")?;
        if stored_hash == hash {
            Some(content.to_string())
        } else {
            //dbg!("hash mismatch", &hash, &stored_hash);
            None
        }
    }

    pub(crate) fn history_get(&self, name: &str) -> Result<Vec<String>> {
        let dir = self.data_path.join("history");
        std::fs::create_dir_all(&dir)?;
        let filename = dir.join(format!("{name}.json"));
        let raw = std::fs::read_to_string(&filename)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub(crate) fn history_store(&self, name: &str, entries: &Vec<String>) -> Result<()> {
        let dir = self.data_path.join("history");
        std::fs::create_dir_all(&dir)?;
        let filename = dir.join(format!("{name}.json"));
        let raw = serde_json::to_string(&entries)?;
        std::fs::write(&filename, raw)?;
        Ok(())
    }

    pub fn get_mail_accounts(&self) -> Vec<MailAccount> {
        let inner = || -> Option<Vec<MailAccount>> {
            let accounts = Vec::new();
            for acc in (&self.settings).get("mail.accounts")?.as_table()?.iter() {
                let name = acc.0;
                let sender = acc.1.get("sender")?.as_str()?;
                //let addresses: Vec<String> = acc.1.get("sender")?.as_array()?.collect();
                ////todo
                //let addresses = Vec::new();
            }
            Some(accounts)
        };
        let res = inner();
        res.unwrap_or_else(|| Vec::new())
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

    pub fn dirname(&self, data_path: &PathBuf) -> PathBuf {
        Node::dirname_from_path(data_path, &self.path[..])
    }

    pub fn dirname_from_path(data_path: &PathBuf, path: &str) -> PathBuf {
        let mut filename = data_path.clone();
        for p in path.chars() {
            // it's a d/i/r/e/c/t/o/r/y path
            filename.push(p.to_string());
        }
        filename
    }
    fn parse(path: String, file_path: &Path) -> Node {
        let raw = std::fs::read_to_string(file_path).unwrap();
        let header = Self::extract_header(&raw);
        Node { path, header, raw }
    }

    fn extract_header(contents: &str) -> Header {
        let title = match contents.split_once("\n") {
            Some((first_line, _)) => first_line.trim_start_matches("= "),
            _ => contents,
        };
        let (first_para, has_more) = match contents.split_once("\n\n") {
            Some((first_para, _)) => (
                if first_para.contains('\n') {
                    first_para.strip_prefix(title).unwrap().trim()
                } else {
                    let mut it = contents.split("\n\n");
                    it.next();
                    it.next().unwrap().trim()
                },
                false,
            ),
            _ => (contents, true),
        };
        Header {
            title: title.to_string(),
            first_paragraph: first_para.to_string(),
            has_more_content: has_more,
        }
    }

    pub fn content(&self) -> &str {
        &self.raw
    }

    pub fn extract_tags(text: &str) -> HashSet<String> {
        let re = Lazy::new(|| Regex::new(r"#[A-Za-z][A-Za-z0-9]+").unwrap());
        let res = re.find_iter(text).map(|m| m.as_str().to_string()).collect();
        res
    }

    pub fn get_tags(&self) -> Vec<String> {
        return Node::extract_tags(&self.raw).into_iter().collect();
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
