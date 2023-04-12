#![allow(dead_code)]
#![allow(unused_imports)]
use crate::openai;
use anyhow::{anyhow, bail, Context, Result};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreePath(Vec<u32>);

impl TreePath {
    pub fn new() -> TreePath {
        TreePath(Vec::new())
    }

    fn temp_path() -> TreePath {
        let mut res = TreePath::new();
        res.0.push(u32::MAX);
        res
    }

    pub fn concat(&self, other: &TreePath) -> TreePath {
        let mut res = self.clone();
        res.0.extend(other.0.iter());
        res
    }

    pub fn push(&mut self, other: u32) {
        self.0.push(other);
    }

    pub fn pop(&mut self) -> Option<u32> {
        self.0.pop()
    }

    pub fn append(&self, other: u32) -> TreePath {
        let mut res = self.clone();
        res.0.push(other);
        res
    }

    fn from_path(path: &Path) -> Result<TreePath> {
        let mut res = TreePath::new();
        for p in path.iter() {
            //parse OSStr to int
            let parsed = p.to_str().ok_or(anyhow!("no filename?"))?.parse::<u32>()?;
            res.0.push(parsed)
        }
        Ok(res)
    }

    pub fn from_file_path(file_path: &str) -> Result<TreePath> {
        {
            let mut res = TreePath::new();
            for p in file_path.split("/") {
                if p.trim().len() == 0 {
                    continue;
                }
                //parse OSStr to int
                let parsed = p.parse::<u32>()?;
                res.0.push(parsed)
            }
            Ok(res)
        }
    }

    pub fn to_human(self: &TreePath) -> String {
        let mut r = String::new();
        let mut last_was_number = false;
        for block in self.iter() {
            if *block < 26 {
                r.push((b'A' + *block as u8) as char);
                last_was_number = false;
            } else {
                if last_was_number {
                    r.push('/');
                }
                r.push_str(&block.to_string());
                last_was_number = true;
            }
        }
        r
    }
    pub fn from_human(path: &str) -> Result<TreePath> {
        //each block is either
        // one letter A-Z
        // only digits
        // if two digit blocks follow each other, they're seperated with /
        // the letters get turned into 0..26
        // the digits are numbers > 26.
        //
        //
        let regex = Regex::new(r"([A-Z]|[0-9]+|/[0-9]+)/?").unwrap();
        let mut res: Vec<u32> = Vec::new();

        for cap in regex.captures_iter(path) {
            let group = cap.get(1).unwrap().as_str();
            match group.chars().next().unwrap() {
                'A'..='Z' => {
                    let idx = group.chars().next().unwrap() as u32 - b'A' as u32;
                    res.push(idx);
                }
                '/' => {
                    let num = group[1..].parse::<u32>().unwrap();
                    res.push(num);
                }
                _ => {
                    let num = group.parse::<u32>().unwrap();
                    res.push(num);
                }
            }
        }

        Ok(TreePath::from(res))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<u32> {
        self.0.iter()
    }

    pub fn starts_with(&self, other: &TreePath) -> bool {
        self.0.starts_with(&other.0)
    }
    pub fn ends_with(&self, other: &TreePath) -> bool {
        self.0.ends_with(&other.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn from_str(input: &str) -> Result<Self> {
        let mut result = Vec::new();
        for s in input.split("/") {
            result.push(s.parse()?);
        }
        Ok(TreePath(result))
    }

    pub fn parent(&self) -> TreePath {
        let mut parent = self.clone();
        parent.0.pop();
        parent
    }

    fn to_filepath(&self, prefix: &Path) -> PathBuf {
        let mut path = prefix.to_path_buf();
        for p in &self.0 {
            path.push(p.to_string());
        }
        path
    }
}

impl From<&[u32]> for TreePath {
    fn from(input: &[u32]) -> Self {
        let mut res = TreePath::new();
        for x in input {
            res.0.push(*x);
        }
        res
    }
}

impl From<Vec<u32>> for TreePath {
    fn from(input: Vec<u32>) -> Self {
        TreePath(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Header {
    pub title: String,
    pub first_paragraph: String,
    pub has_more_content: bool,
}

#[derive(Debug, Clone, Eq, Serialize)]
pub(crate) struct Node {
    pub path: TreePath,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GitHistoryEntry {
    hash: String,
    date: String,
    message: String,
}

impl core::fmt::Display for TreePath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("/")
        )
    }
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

    pub fn store_settings(&self) -> Result<()> {
        let out = self.settings.to_string();
        std::fs::write(self.data_path.join("settings.toml"), out).context("saving settings failed")
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
                    .unwrap();
                let tree_path = TreePath::from_path(path);
                match tree_path {
                    Ok(tree_path) => nodes.push(Node::parse(tree_path, entry.path())),
                    Err(_) => continue,
                }
            }
        }
        //nodes that have no FLORG_FILENAME but child nodes need to be patched in.
        let mut paths: HashSet<_> = nodes.iter().map(|n| n.path.clone()).collect();
        let paths2 = paths.clone();
        for p in paths2.iter() {
            if p.len() > 0 {
                let parent = p.parent();
                if !paths.contains(&parent) {
                    let path = parent;
                    let node = Node {
                        path: path.clone(),
                        header: Header {
                            title: "(empty node)".to_string(),
                            first_paragraph: "".to_string(),
                            has_more_content: false,
                        },
                        raw: "".to_string(),
                    };
                    nodes.push(node);
                    paths.insert(path.clone());
                }
            }
        }
        nodes.sort_by(|a, b| a.path.cmp(&b.path));
        nodes
    }

    pub(crate) fn get_node(&self, path: &TreePath) -> Option<&Node> {
        //todo: replace with binary search
        self.nodes.iter().filter(|n| &n.path == path).next()
    }

    pub(crate) fn get_node_mut(&mut self, path: &TreePath) -> Option<&mut Node> {
        self.nodes.iter_mut().filter(|n| &n.path == path).next()
    }

    pub(crate) fn delete_node(&mut self, path: &TreePath, commit: bool) -> Result<()> {
        let node = self.get_node(path).context("node not found")?;
        let file_path = node.dirname(&self.data_path);
        std::fs::remove_dir_all(file_path).context("failed to remove_dir_all")?;
        if commit {
            self.add_and_commit(&format!("Deleted node {path} and children"))?;
        }
        self.nodes.retain(|n| !n.path.starts_with(path));
        Ok(())
    }

    pub(crate) fn move_node(
        &mut self,
        org_path: &TreePath,
        new_path: &TreePath,
        commit: bool,
    ) -> Result<()> {
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
            self.add_and_commit(&format!("moved node {org_path} to {new_path}"))?;
        }
        Ok(())
    }

    pub(crate) fn swap_node_with_previous(&mut self, path: &TreePath) -> Result<()> {
        if path.ends_with(&TreePath::from(&[0u32][..])) {
            bail!("Can not swap first node");
        }
        let temp_path = TreePath::temp_path();
        if self.get_node(&temp_path).is_some() {
            bail!("Can not swap, tempmove node ('!') already exists. Manually clean up the storage directory.");
        }
        let prev = self.find_previous_sibling(path).context("No prev node")?;
        self.move_node(&prev, &temp_path, false)
            .context("failed to move prev to tempmove")?;
        self.move_node(path, &prev, false)
            .context("failed to move path to prev")?;
        self.move_node(&temp_path, path, false)
            .context("failed to move tempmove into prev")?;
        self.make_nodes_sorted();
        self.add_and_commit(&format!("Swapped nodes up: {} and {}", path, prev))?;
        Ok(())
    }

    pub(crate) fn swap_node_with_next(&mut self, path: &TreePath) -> Result<()> {
        let temp_path = TreePath::temp_path();
        if self.get_node(&temp_path).is_some() {
            bail!("Can not swap, tempmove node ('!') already exists. Manually clean up the storage directory.");
        }
        let next = self.find_next_sibling(path).context("No next node")?;
        self.move_node(&next, &temp_path, false)
            .context("failed to move next to tempmove")?;
        self.move_node(path, &next, false)
            .context("failed to move path to next")?;
        self.move_node(&temp_path, path, false)
            .context("failed to move tempmove into next")?;
        self.make_nodes_sorted();
        self.add_and_commit(&format!("Swapped nodes down: {} and {}", path, next))?;
        Ok(())
    }

    fn make_nodes_sorted(&mut self) {
        self.nodes.sort_by(|a, b| a.path.cmp(&b.path));
    }

    fn find_previous_sibling(&self, path: &TreePath) -> Option<TreePath> {
        let node_idx = self
            .nodes
            .binary_search_by(|node| node.path.cmp(path))
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
                    if node.path.0[..path.len() - 1] == path.0[..path.len() - 1] {
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

    fn find_next_sibling(&self, path: &TreePath) -> Option<TreePath> {
        let node_idx = self
            .nodes
            .binary_search_by(|node| node.path.cmp(path))
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
                    if node.path.0[..path.len() - 1] == path.0[..path.len() - 1] {
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

    fn rename_all_children(&mut self, org_path: &TreePath, new_path: &TreePath) {
        //todo: fix all the links
        for node in self.nodes.iter_mut() {
            if node.path.starts_with(org_path) {
                let suffix = TreePath::from(&node.path.0[org_path.len()..]);
                println!(
                    "Renaming {} to {}",
                    node.path,
                    format!("{}{}", new_path, suffix)
                );
                node.path = new_path.concat(&suffix);
            }
        }
    }

    pub(crate) fn levels(&self, path: &TreePath) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let mut p = path.to_owned();
        while !p.is_empty() {
            let entry = self.get_node(&p);
            let letter = p.0.pop().unwrap().to_string();
            res.push((
                letter,
                entry.map(|x| &x.header.title[..]).unwrap_or("").to_string(),
            ));
        }
        res.reverse();
        res
    }

    pub(crate) fn children_for(&self, path: &TreePath) -> Vec<&Node> {
        let lp = path.len() + 1;
        let mut res: Vec<_> = self
            .nodes
            .iter()
            .filter(|n| n.path.len() == lp && n.path.starts_with(path))
            .collect();
        res.sort();
        res
    }

    pub(crate) fn children_paths_for(&self, path: &TreePath) -> Vec<TreePath> {
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

    //perform a depth first iteration of the tree
    //abort if the callback returns false
    pub(crate) fn depth_first_search(
        &self,
        path: &TreePath,
        callback: &Box<&dyn Fn(&Node) -> bool>,
    ) -> Option<&Node> {
        for child in self.children_for(path) {
            let r = callback(child);
            if r {
                return Some(child);
            }
            let r = self.depth_first_search(&child.path, callback);
            match r {
                Some(node) => return Some(node),
                None => {}
            }
        }
        None
    }

    fn get_letter_set(start: char, stop: char) -> HashSet<char> {
        let mut res = HashSet::new();
        for c in start..=stop {
            res.insert(c);
        }
        res
    }

    pub(crate) fn find_next_empty_child(&self, path: &TreePath) -> TreePath {
        let used: HashSet<u32> = self
            .children_for(path)
            .iter()
            .map(|node| node.path.0.iter().last().map(|x| *x).unwrap())
            .collect();
        for ii in 0..u32::MAX {
            if !used.contains(&ii) {
                return path.concat(&TreePath::from(&[ii][..]));
            }
        }
        panic!("could not find an empty child. Exceeded u32::MAX");
    }

    pub(crate) fn replace_node(&mut self, node: Node, commit: bool) -> Result<()> {
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
            self.add_and_commit(&msg)?;
        }
        self.nodes.push(node);
        Ok(())
    }

    pub(crate) fn remove_placeholder(&mut self, path: &TreePath) {
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
                    remove_path = Some(node.path.clone())
                }
            }
        }
        if let Some(remove_path) = remove_path {
            self.remove_node(&remove_path);
        }
    }

    pub fn remove_node(&mut self, path: &TreePath) {
        let filename: PathBuf = Node::dirname_from_path(&self.data_path, path);
        std::fs::remove_dir_all(filename).expect("Failed to unlink file");
        self.nodes.retain(|x| &x.path != path);
        //copilot: unlink  filename
    }

    pub fn add_and_commit(&self, msg: &str) -> Result<()> {
        std::process::Command::new(&self.git_binary)
            .arg("add")
            .arg(".")
            .current_dir(&self.data_path)
            .status()
            .context("git add failed")?;
        std::process::Command::new(&self.git_binary)
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(&self.data_path)
            .status()
            .context("git add failed")?;
        Ok(())
    }

    pub fn get_git_history(&self, limit: u32) -> Result<Vec<GitHistoryEntry>> {
        let raw = std::process::Command::new(&self.git_binary)
            .arg("log")
            .arg("--date=iso")
            .arg("--pretty=%ah%n%H%n%s%n")
            .arg(format!("-n{limit}"))
            .current_dir(&self.data_path)
            .output()
            .context("git log failed")?
            .stdout;
        let raw = String::from_utf8(raw).context("git log output not utf8")?;
        let mut res: Vec<GitHistoryEntry> = Vec::new();
        //every line is valid json, but it's not a json array...
        for block in raw.split("\n\n") {
            let block = block.trim();
            if !block.is_empty() {
                let lines: Vec<_> = block.split("\n").collect();
                let parsed: GitHistoryEntry = GitHistoryEntry {
                    hash: lines[1].to_string(),
                    date: lines[0].to_string(),
                    message: lines[2].to_string(),
                };
                res.push(parsed)
            }
        }
        Ok(res)
    }

    pub fn git_undo(&mut self, hash: &str) -> Result<()> {
        std::process::Command::new(&self.git_binary)
            .arg("reset")
            .arg("--hard")
            .arg(hash)
            .current_dir(&self.data_path)
            .status()
            .context("git reset hard hash failed")?;
        std::process::Command::new(&self.git_binary)
            .arg("reset")
            .arg("HEAD@{1}")
            .current_dir(&self.data_path)
            .status()
            .context("git reset to head failed")?;

        self.add_and_commit(&format!("Undo to state of {}", hash))?;
        self.reload();
        Ok(())
    }

    pub(crate) fn set_cached_node(
        &mut self,
        path: &TreePath,
        raw: &str,
        to_cache: &str,
    ) -> Result<()> {
        let node_path = Node::dirname_from_path(&self.data_path, path).join(FLORG_CACHE_FILENAME);
        let hash = sha256::digest(raw.as_bytes()).to_string();
        let output = format!("{}\n{}", hash, to_cache);
        Ok(std::fs::write(node_path, output)?)
    }

    pub(crate) fn get_cached_node(&mut self, path: &TreePath) -> Option<String> {
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

    pub fn sort_children(&mut self, path: &TreePath) -> Result<()> {
        fn natural_sort(vec: &mut Vec<(String, TreePath)>) {
            vec.sort_by(|a, b| {
                let split_a = a.0.split(|c: char| !c.is_ascii_digit());
                let split_b = b.0.split(|c: char| !c.is_ascii_digit());

                for (part_a, part_b) in split_a.zip(split_b) {
                    if let (Ok(num_a), Ok(num_b)) = (part_a.parse::<u32>(), part_b.parse::<u32>()) {
                        if num_a != num_b {
                            return num_a.cmp(&num_b);
                        }
                    } else {
                        let cmp = part_a.cmp(part_b);
                        if cmp != Ordering::Equal {
                            return cmp;
                        }
                    }
                }

                // If all substrings are equal, compare the full strings
                a.0.cmp(&b.0)
            });
        }

        let children = self.children_for(path);
        let mut children: Vec<_> = children
            .iter()
            .map(|x| (x.header.title.clone(), x.path.clone()))
            .collect();
        //perform natural sort on titles

        //children.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        dbg!(&children);
        natural_sort(&mut children);
        let children: Vec<_> = children
            .into_iter()
            .enumerate()
            .map(|(ii, (_title, old_path))| (old_path, path.append(ii as u32)))
            .collect();
        self.remap_children(path.clone(), children)?;
        self.add_and_commit(&format!("Sorted children of {path}"))?;
        Ok(())
    }

    pub fn compact_children(&mut self, path: &TreePath) -> Result<()> {
        let children = self.children_for(path);
        let mapping: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(ii, x)| (x.path.clone(), path.append(ii as u32)))
            .collect();
        self.remap_children(path.clone(), mapping)
    }

    fn remap_children(
        &mut self,
        parent: TreePath,
        mapping: Vec<(TreePath, TreePath)>,
    ) -> Result<()> {
        //I have a mapping old->new
        //and I need to swap the old into the new positions,
        //where the new position might already be occupied.
        //I want to use only a single temp variable.

        //parent = path minus last char
        let temp_root = parent.concat(&TreePath::temp_path());
        let temp_dir = Node::dirname_from_path(&self.data_path, &temp_root);
        std::fs::create_dir_all(&temp_dir)?;
        for (old_path, _new_path) in mapping.iter() {
            let temp_path = temp_root.append(*old_path.0.last().unwrap());
            self.move_node(old_path, &temp_path, false)?;
        }
        for (old_path, new_path) in mapping.iter() {
            let temp_path = temp_root.append(*old_path.0.last().unwrap());
            self.move_node(&temp_path, &new_path, false)?;
        }

        self.make_nodes_sorted();
        Ok(())
    }

    pub fn get_mail_accounts(&self) -> Vec<MailAccount> {
        /*
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
        */
        todo!();
    }
}

impl Node {
    pub fn new(path: &TreePath, raw: &str) -> Node {
        let header = Self::extract_header(raw);
        Node {
            path: path.clone(),
            header,
            raw: raw.to_string(),
        }
    }

    pub fn dirname(&self, data_path: &PathBuf) -> PathBuf {
        Node::dirname_from_path(data_path, &self.path)
    }

    pub fn dirname_from_path(data_path: &PathBuf, path: &TreePath) -> PathBuf {
        path.to_filepath(data_path)
    }

    fn parse(path: TreePath, file_path: &Path) -> Node {
        let raw = std::fs::read_to_string(file_path).unwrap();
        let header = Self::extract_header(&raw);
        Node { path, header, raw }
    }

    fn extract_header(contents: &str) -> Header {
        let untrimmed_title = match contents.split_once("\n") {
            Some((first_line, _)) => first_line,
            _ => contents,
        };
        let title = untrimmed_title.trim_start_matches("=").trim_start();
        let (first_para, has_more) = match contents.split_once("\n\n") {
            Some((first_para, _)) => {
                (
                    if first_para.contains('\n') {
                        let p = first_para.strip_prefix(untrimmed_title);
                        match p {
                            Some(p) => p.trim(),
                            None => {
                                dbg!(&contents);
                                dbg!(&title);
                                panic!("Well this shouldn't happen now that we use the untrimmed title..");
                            }
                        }
                    } else {
                        let mut it = contents.split("\n\n");
                        it.next();
                        it.next().unwrap().trim()
                    },
                    false,
                )
            }
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
