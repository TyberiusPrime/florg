#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod mail;
mod openai;
mod storage;

use anyhow::{anyhow, Context, Result};
use chrono::Datelike;
use inotify::{Inotify, WatchMask};
use once_cell::sync::OnceCell;
use serde::Serialize;
use signal_hook::iterator::Signals;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    thread::{self},
};
use storage::{Node, Storage, TreePath};
use tauri::Manager;

static STORAGE: OnceCell<Mutex<Storage>> = OnceCell::new();

pub struct OpenEditor {
    path: String,
    temp_file: PathBuf,
    org_content: String,
    process: std::process::Child,
    window_title: String,
    remove_after: bool,
}

#[derive(Debug)]
struct RuntimeState {
    open_editors: Vec<OpenEditor>,
    app_handle: tauri::AppHandle,
    notmuch_db: mail::MailStore,
}

impl RuntimeState {
    fn new(handle: tauri::AppHandle, notmuch_db: mail::MailStore) -> RuntimeState {
        RuntimeState {
            open_editors: Vec::new(),
            app_handle: handle,
            notmuch_db,
        }
    }
}

#[derive(Debug)]
struct TauriError(anyhow::Error);

//#[derive(Debug, Serialize)]
type TauriResult<T> = Result<T, TauriError>;

impl serde::Serialize for TauriError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ErrorInfo {
            error_chain: Vec<String>,
            //backtrace: String,
        }

        ErrorInfo {
            error_chain: self.0.chain().map(ToString::to_string).collect(),
            //backtrace: self.0.backtrace().to_string(),
        }
        .serialize(serializer)
    }
}

//implement from<anyhow::Error> for tauri error
impl From<anyhow::Error> for TauriError {
    fn from(value: anyhow::Error) -> Self {
        TauriError(value)
    }
}

static RUNTIME_STATE: OnceCell<Mutex<RuntimeState>> = OnceCell::new();

impl std::fmt::Debug for OpenEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenEditor")
            .field("path", &self.path)
            .field("temp_file", &self.temp_file.file_name())
            .field("process", &self.process.id())
            .finish()
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
fn parse_raw_content(raw_content: &str) -> &str {
    match raw_content.split_once("\n--\n") {
        Some((_header, content)) => content.trim(),
        None => &raw_content,
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NodeForJSInner {
    pub path: String,
    pub header: storage::Header,
    pub raw: String,
    pub tags: Vec<String>,
    //children: Vec<Node>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NodeForJS {
    pub node: Option<NodeForJSInner>,
    pub levels: Vec<(String, String)>,
    pub children: Vec<NodeForJSInner>,
    pub tags: Vec<String>,
}

impl From<&Node> for NodeForJSInner {
    fn from(node: &Node) -> Self {
        NodeForJSInner {
            path: node.path.to_human(),
            header: node.header.clone(),
            raw: node.raw.clone(),
            tags: node.get_tags(),
        }
    }
}
#[tauri::command]
fn get_node(path: &str) -> TauriResult<NodeForJS> {
    let s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    let node: Option<NodeForJSInner> = s.get_node(&path).map(|x| x.into());
    let children: Vec<NodeForJSInner> = s.children_for(&path).iter().map(|x| (*x).into()).collect();
    let tags = node.as_ref().map_or_else(|| Vec::new(), |x| x.tags.clone());
    Ok(NodeForJS {
        node,
        levels: s.levels(&path),
        children,
        tags,
    })
}

#[tauri::command]
fn get_node_title(path: &str) -> Option<String> {
    let s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path).ok()?;
    s.get_node(&path).map(|x| x.header.title.clone())
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TreeForJS {
    pub path: String,
    pub title: String,
    pub first_paragraph: String,
    pub more_text: bool,
    pub children: Vec<TreeForJS>,
    pub has_children: bool,
    pub tags: Vec<String>,
}

fn descend(
    path: &TreePath,
    storage: &MutexGuard<Storage>,
    remaining_depth: i32,
) -> Option<TreeForJS> {
    let node = storage.get_node(&path);
    {
        let children = storage.children_paths_for(&path);
        let has_children = !children.is_empty();
        Some(TreeForJS {
            path: path.to_human(),
            title: node.map_or_else(|| "(empty node)".to_string(), |x| x.header.title.clone()),
            first_paragraph: node
                .map_or_else(|| "".to_string(), |x| x.header.first_paragraph.clone()),
            more_text: node.map_or(false, |x| x.header.has_more_content),
            children: if remaining_depth > 0 {
                children
                    .iter()
                    .filter_map(|x| descend(x, storage, remaining_depth - 1))
                    .collect()
            } else {
                Vec::new()
            },
            has_children,
            tags: node.map_or_else(|| Vec::new(), |x| x.get_tags()),
        })
    }
}

#[tauri::command]
fn get_tree(path: &str, max_depth: i32) -> Option<TreeForJS> {
    let s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path).ok()?;
    descend(&path, &s, max_depth)
}

#[tauri::command]
fn get_parent(path: &str) -> TauriResult<String> {
    let tp = TreePath::from_human(path)?;
    Ok(tp.parent().to_human())
}

#[tauri::command]
fn move_node(org_path: &str, new_path: &str) -> Option<String> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    match s.move_node(
        &TreePath::from_human(org_path).ok()?,
        &TreePath::from_human(new_path).ok()?,
        true,
    ) {
        Ok(_) => None,
        Err(e) => {
            println!("{:?}", &e);
            Some(e.to_string())
        }
    }
}

#[tauri::command]
fn swap_node_with_previous(path: &str) -> Option<String> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path).ok()?;
    match s.swap_node_with_previous(&path) {
        Ok(_) => None,
        Err(e) => {
            println!("{:?}", &e);
            Some(e.to_string())
        }
    }
}
#[tauri::command]
fn swap_node_with_next(path: &str) -> Option<String> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path).ok()?;
    match s.swap_node_with_next(&path) {
        Ok(_) => None,
        Err(e) => {
            println!("{:?}", &e);
            Some(e.to_string())
        }
    }
}

#[tauri::command]
fn change_node_text(path: &str, text: &str, commit: Option<bool>) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let tree_path = TreePath::from_human(path)?;
    let node = Node::new(&tree_path, text);

    ss.replace_node(node, commit.unwrap_or(true))?;
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    lock.app_handle.emit_all("node-changed", path).ok();
    TauriResult::Ok(())
}
#[tauri::command]
fn commit(text: &str) -> TauriResult<()> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    TauriResult::Ok(ss.add_and_commit(text)?)
}

#[tauri::command]
fn get_node_folder_path(path: &str) -> TauriResult<String> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    Ok(Node::dirname_from_path(&ss.data_path, &path)
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn delete_node(path: &str, commit: Option<bool>) -> TauriResult<()> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    s.delete_node(&path, commit.unwrap_or(true))?;
    TauriResult::Ok(())
}

#[tauri::command]
fn sort_children(path: &str) -> TauriResult<()> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    s.sort_children(&path)?;
    TauriResult::Ok(())
}

#[tauri::command]
fn compact_children(path: &str) -> TauriResult<()> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    s.compact_children(&path)?;
    TauriResult::Ok(())
}

#[tauri::command]
fn edit_node(path: &str, window_title: &str, new_text: Option<&str>) -> TauriResult<bool> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    dbg!("edit_node", &path);
    let mut runtime_state = RUNTIME_STATE.get().unwrap().lock().unwrap();
    if runtime_state
        .open_editors
        .iter()
        .filter(|entry| entry.path == path.to_string())
        .next()
        .is_none()
    {
        {
            let node_folder = Node::dirname_from_path(&ss.data_path, &path);
            let tf_folder = node_folder;
            std::fs::create_dir_all(&tf_folder).unwrap();
            let tf = tf_folder.join(format!(
                "{}.temp{}",
                path.to_human().replace("/", "_"),
                storage::FLORG_SUFFIX
            ));
            dbg!(&tf);

            let mut content = "".to_string();
            let mut skip_lines = 0;
            if path.is_empty() {
                content += "(root)\n";
                skip_lines = 3;
            } else {
                let mut so_far = TreePath::new();

                for (ii, path_component) in path.iter().enumerate() {
                    //todo: humanize
                    let letter = if *path_component <= 26 {
                        (('A' as u8 + *path_component as u8) as char).to_string()
                    } else {
                        path_component.to_string()
                    };
                    content += &format!(
                        "{} {}{}\n",
                        letter,
                        " ".repeat(ii),
                        ss.get_node(&so_far)
                            .map(|x| &x.header.title[..])
                            .unwrap_or("")
                    );
                    so_far.push(*path_component);
                    skip_lines = ii + 5;
                }
            }
            content += "Content after the next line. First line = new title \n--\n\n";
            let node = ss.get_node(&path).map(|x| x.clone());
            match new_text {
                Some(text) => {
                    content += text;
                }
                None => {
                    content += node.as_ref().map(|x| &x.raw[..]).unwrap_or("");
                }
            };
            std::fs::write(&tf, &content).expect("temp file write failure");

            if let None = node {
                let place_holder = Node::new(&path, "(placeholder)");
                ss.replace_node(place_holder, false)?;
            }
            edit_file(
                tf,
                content,
                skip_lines,
                "node-temp-changed",
                path.to_human(),
                window_title,
                &mut runtime_state,
                true,
            );
        }

        TauriResult::Ok(true)
    } else {
        TauriResult::Ok(false)
    }
}
fn edit_file(
    path: PathBuf,
    content: String,
    skip_lines: usize,
    msg_to_js: &'static str,
    path_for_js: String,
    window_title: &str,
    runtime_state: &mut MutexGuard<RuntimeState>,
    remove_after: bool,
) {
    println!("path_for_js: {}", &path_for_js);
    let process = std::process::Command::new("kitty")
        .arg("--")
        .arg("nvim")
        .arg(format!("+{}", skip_lines))
        .arg(path.to_string_lossy().to_string())
        .spawn()
        .expect("Failed to spawn kitty&neovim");
    let tf_name_for_thread = path.file_name().map(|x| x.to_os_string()).unwrap();
    let tf_for_thread = path.clone();
    let path_for_thread = path_for_js.to_string();
    thread::spawn(move || {
        let mut inotify = Inotify::init().expect("inotify init failed");
        let tf_folder = tf_for_thread.parent().unwrap();
        inotify
            .add_watch(
                tf_folder,
                WatchMask::CLOSE_WRITE | WatchMask::DELETE | WatchMask::MOVED_TO,
            )
            .expect("inotify add watch failed");
        loop {
            let mut buffer = [0; 1024];
            let events = inotify
                .read_events_blocking(&mut buffer)
                .expect("Error while reading events");

            println!("eventS!");

            let mut do_send = false;
            for event in events {
                dbg!(&event);
                /* if event.mask.contains(EventMask::DELETE) {
                if let Some(event_filename) = event.name {
                    if event_filename == tf_name_for_thread {
                        break;
                    };
                } */
                //if event.mask.contains(EventMask::CLOSE_WRITE) {
                if let Some(event_filename) = event.name {
                    if event_filename == tf_name_for_thread {
                        do_send = true;
                    }
                }
            }
            if do_send {
                let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
                let content = std::fs::read_to_string(&tf_for_thread);
                if let Ok(content) = content {
                    //we now also get an event when the temp file get's finally removed.
                    let content = parse_raw_content(&content);
                    //println!("Telling viewer about changed temp file");
                    lock.app_handle
                        .emit_all(&msg_to_js, (&path_for_thread, content))
                        .ok();
                }
            }
            //}
            //dbg!(event);
            //http://localhost:1420/#/node/T
            //http://localhost:1420/#T
            //dbg!(&tf_name_for_thread);
            // Handle event
        }
    });
    runtime_state.open_editors.push(OpenEditor {
        path: path_for_js.to_string(),
        temp_file: path,
        org_content: content,
        process,
        window_title: window_title.to_string(),
        remove_after: remove_after,
    });
}

fn get_settings_temp_filename(ss: &MutexGuard<Storage>) -> PathBuf {
    let tf_folder = ss.data_path.join("temp");
    std::fs::create_dir_all(&tf_folder).unwrap();
    let tf = tf_folder.join("settings.toml");
    tf
}

fn spawn_settings_editor(tf: PathBuf, content: String, lock: &mut MutexGuard<RuntimeState>) {
    //todo: combine with edit_file
    let tf_name = tf.to_str().expect("temp file was not unicode name?");
    let process = std::process::Command::new("kitty")
        .arg("--")
        .arg("nvim")
        .arg(format!("+{}", 0))
        .arg(tf_name)
        .spawn()
        .expect("Failed to spawn kitty&neovim");
    lock.open_editors.push(OpenEditor {
        path: "settings.toml".to_string(),
        temp_file: tf,
        org_content: content,
        process,
        window_title: "Settings".to_string(),
        remove_after: true,
    });
}

#[tauri::command]
fn edit_settings() -> bool {
    let path = "settings.toml";
    let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();

    if lock
        .open_editors
        .iter()
        .filter(|entry| entry.path == path)
        .next()
        .is_none()
    {
        {
            let ss = STORAGE.get().unwrap().lock().unwrap();
            let tf = get_settings_temp_filename(&ss);
            dbg!(&ss.settings);
            let content = ss.settings.to_string();

            std::fs::write(&tf, &content).expect("temp file write failure");

            spawn_settings_editor(tf, content, &mut lock);
        }

        true
    } else {
        false
    }
}

#[tauri::command]
fn list_open_paths() -> Vec<String> {
    RUNTIME_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .open_editors
        .iter()
        .map(|x| x.path.to_string())
        .collect()
}

#[tauri::command]
fn date_to_path(date_str: &str) -> Option<String> {
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    Some(chrono_date_to_path(date).to_human())
}

fn chrono_date_to_path(date: chrono::NaiveDate) -> TreePath {
    let iso_year = date.iso_week().year();
    let kw = if iso_year != date.year() { 0 } else { date.iso_week().week()};
    //range is 0..52
    let quarter = kw / 13;
    let kw = kw % 13;
    let mut path = vec![match quarter {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 3,
        _ => unreachable!(),
    }];
    path.push(match kw {
        0..=12 => kw as u32,
        _ => unreachable!(),
    });
    let weekday = date.weekday();
    path.push(match weekday {
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
        chrono::Weekday::Sun => 7,
    });

    return TreePath::from(path);
}

#[tauri::command]
fn create_calendar(parent_path: &str, year: i32) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let parent_path = TreePath::from_human(parent_path)?;
    if !ss.children_for(&parent_path).is_empty() {
        return TauriResult::<()>::Err(TauriError(anyhow!(
            "Node had children - not filling in calendar nodes"
        )));
    }
    for q in 1..5 {
        let path = parent_path.append(match q {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            _ => unreachable!(),
        });
        let text = format!("Q{q}/{year}");
        let node = Node::new(&path, &text);
        ss.replace_node(node, false)?;
    }
    let mut last_kw = 500u32;
    let start = chrono::NaiveDate::from_ymd_opt(year as i32, 1, 1).unwrap();
    for date in start.iter_days().take_while(|x| x.year() == year) {
        let iso_year = date.iso_week().year();
        let kw = if iso_year != year { 0 } else { date.iso_week().week()};

        if kw != last_kw {
            let pp = chrono_date_to_path(date).parent();
            let path = parent_path.concat(&pp);
            let text = format!("KW {}\n", kw+1);
            let node = Node::new(&path, &text);
            ss.replace_node(node, false)?;
            last_kw = kw;
        }
        let path = parent_path.concat(&chrono_date_to_path(date));
        println!("{} {}", date, path);
        let text = date.format("%Y-%m-%d %a\n").to_string();
        let node = Node::new(&path, &text);
        ss.replace_node(node, false)?;
    }
    println!("added days");
    ss.add_and_commit(&format!("Added date notes below {parent_path}"))?;
    TauriResult::Ok(())
}
#[tauri::command]
fn reload_data() {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    ss.reload();
    println!("reloaded storage");
}

#[tauri::command]
fn get_tags() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("tags")
}

#[tauri::command]
fn get_bookmarks() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("bookmarks")
}
#[tauri::command]
fn set_bookmarks(bookmarks: HashMap<String, String>) -> TauriResult<()> {
    save_to_settings_str_map("bookmarks", bookmarks)?;
    TauriResult::Ok(())
}

#[tauri::command]
fn get_nav() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("nav")
}

#[tauri::command]
fn get_mail_search_folders() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("mail_queries")
}

#[tauri::command]
fn find_next_empty_child(path: &str) -> Result<String, TauriError> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    Ok(ss.find_next_empty_child(&path).to_human())
}

#[derive(Serialize, Debug)]
struct RipgrepResult {
    path: String,
    title: String,
    parent_titles: Vec<String>,
    lines: Vec<(u32, String)>,
    tags: Vec<String>,
}

#[tauri::command]
fn ripgrep_below_node(
    query_path: &str,
    search_term: &str,
    only_matching: Option<bool>,
) -> Option<Vec<RipgrepResult>> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let query_path = TreePath::from_human(query_path).ok()?;
    let search_path = Node::dirname_from_path(&ss.data_path, &query_path);
    println!(
        "searching in {search_path:?}, only_matching: {:?}",
        only_matching
    );
    let mut cmd = std::process::Command::new("rg");
    cmd.arg("--type-add")
        .arg("adoc:*.adoc")
        .arg("-t")
        .arg("adoc")
        .arg("-i")
        .arg("--line-number")
        .arg("--heading")
        .arg(search_term);
    if let Some(true) = only_matching {
        cmd.arg("--only-matching");
    }

    let ok = cmd.current_dir(search_path).output();
    match ok {
        Ok(output) => {
            let stdout = std::str::from_utf8(&output.stdout).unwrap();
            dbg!(stdout);
            let mut result = Vec::new();
            for block in stdout.split("\n\n") {
                if block.trim().is_empty() {
                    continue;
                }
                let mut lines = block.split("\n");
                let filename = lines.next();
                let path = match filename {
                    Some(filename) => {
                        if filename.contains(".temp.") {
                            continue;
                        }
                        let filename = filename.trim_end_matches(storage::FLORG_FILENAME);
                        match TreePath::from_file_path(filename) {
                            Ok(path) => path,
                            Err(e) => {
                                println!("failed to parse file path: {} {}", filename, e);
                                continue;
                            }
                        }
                    }
                    None => continue,
                };
                let path = query_path.concat(&path);
                let node = ss.get_node(&path);
                let title = node
                    .map(|x| x.header.title.clone())
                    .unwrap_or_else(|| "(empty node)".to_string());
                let mut parent_titles = Vec::new();
                let mut cpath = path.clone();
                let tags = node.map(|x| x.get_tags()).unwrap_or_default();
                while !cpath.is_empty() {
                    cpath.pop();
                    parent_titles.push(
                        ss.get_node(&cpath)
                            .map(|x| x.header.title.clone())
                            .unwrap_or_else(|| "(empty node)".to_string()),
                    );
                }
                let mut hits = Vec::new();
                for further in lines {
                    let (line_no, hit) = match further.split_once(":") {
                        Some((line_no, hit)) => (line_no, hit),
                        None => continue,
                    };
                    hits.push((line_no.parse::<u32>().unwrap_or(0), hit.to_string()));
                }
                result.push(RipgrepResult {
                    path: path.to_human(),
                    title,
                    parent_titles,
                    lines: hits,
                    tags: tags,
                })
            }
            result.sort_by(|a, b| a.path.cmp(&b.path));
            Some(result)
        }
        Err(e) => {
            println!("error running ripgrep: {}", e);
            None
        }
    }
}
#[tauri::command]
fn get_cached_node(path: &str) -> Option<String> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path).ok()?;
    ss.get_cached_node(&path)
}

#[tauri::command]
fn set_cached_node(path: &str, raw: &str, rendered: &str) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let path = TreePath::from_human(path)?;
    Ok(ss.set_cached_node(&path, raw, rendered)?)
}

#[tauri::command]
fn query_mail(query: &str) -> (Vec<mail::Thread>, bool) {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let mut filtered_authors = HashSet::new();
    filtered_authors.insert("Florian Finkernagel".to_string()); //todo: read from settings
    let res = lock.notmuch_db.query(query, &filtered_authors);
    res
}

#[tauri::command]
fn mail_get_tags() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("mail_tags")
}

#[tauri::command]
fn get_mail_message(id: &str) -> Option<mail::SingleMessage> {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let res = lock.notmuch_db.get_message(id);
    if !res.is_ok() {
        dbg!(&res);
    }
    res.ok()
}

#[tauri::command]
fn get_mail_message_brief(id: &str) -> Option<mail::SingleMessageBrief> {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let res = lock.notmuch_db.get_message_brief(id);
    if !res.is_ok() {
        dbg!(&res);
    }
    res.ok()
}

#[tauri::command]
fn mail_message_add_tags(id: &str, tags: Vec<String>) -> bool {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let res = lock.notmuch_db.add_tags(id, &tags);
    res.is_ok()
}

#[tauri::command]
fn mail_message_remove_tags(id: &str, tags: Vec<String>) -> bool {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let res = lock.notmuch_db.remove_tags(id, &tags);
    dbg!("removed tags", &id, &tags, &res);
    res.is_ok()
}
#[tauri::command]
fn mail_message_toggle_tag(id: &str, tag: String) -> bool {
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let res = lock.notmuch_db.toggle_tag(id, &tag);
    res.is_ok()
}

#[tauri::command]
fn mail_message_store_attachments(id: &str) -> bool {
    fn inner(id: &str) -> anyhow::Result<()> {
        let ss = STORAGE.get().unwrap().lock().unwrap();

        let attachment_dir = ss
            .settings
            .get("mail.attachment_dir")
            .map(|x| x.to_string())
            .unwrap_or("~/attachments".to_string());
        let attachment_dir = expanduser::expanduser(attachment_dir)?;
        let attachment_dir = PathBuf::from(&attachment_dir);
        if attachment_dir.exists() {
            std::fs::remove_dir_all(&attachment_dir)?;
        }
        std::fs::create_dir_all(&attachment_dir)?;

        let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
        lock.notmuch_db.store_attachments(id, &attachment_dir)?;
        Ok(())
    }
    let res = inner(id);
    if !res.is_ok() {
        dbg!(&res);
    }
    res.is_ok()
}
#[tauri::command]
fn mail_message_new(id: Option<String>, window_title: &str) {
    let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    let (filename, org_content) = lock.notmuch_db.new_mail(id, Vec::new());
    let path_for_js = filename.file_name().unwrap().to_string_lossy().to_string();
    edit_file(
        filename,
        org_content,
        0, //todo
        "mail-temp-changed",
        path_for_js,
        window_title,
        &mut lock,
        false,
    );
}

#[tauri::command]
fn chatgpt_get_prompts() -> HashMap<String, HashMap<String, String>> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        gpt.get_all_prompts().unwrap_or_default()
    } else {
        HashMap::new()
    }
}

#[tauri::command]
fn chatgpt_update_prompts(key: &str, prompts: HashMap<String, String>) {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        let ref_prompts: HashMap<&str, &str> =
            prompts.iter().map(|(k, v)| (&k[..], &v[..])).collect();
        gpt.update_prompts(key, ref_prompts).unwrap();
    }
}
#[tauri::command]
fn chatgpt_list_conversations() -> Option<Vec<openai::ConversationListEntry>> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        let mut convos = gpt.list_conversations().unwrap_or_default();
        convos.sort_by_key(|x| x.date);
        convos.reverse();
        dbg!(&convos);
        Some(convos)
    } else {
        None
    }
}
#[tauri::command]
fn chatgpt_new_conversation() -> openai::Conversation {
    use chrono::prelude::*;
    let local: DateTime<Local> = Local::now();
    openai::Conversation {
        title: None,
        date: local.naive_local(),
        prompt: "You are a a helpful assistant".to_string(),
        messages: Vec::new(),
    }
}

#[tauri::command]
fn chatgpt_get_conversation(filename: &str) -> Option<openai::Conversation> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        gpt.get_conversation(filename).ok()
    } else {
        None
    }
}

#[tauri::command]
fn chatgpt_save_conversation(
    filename: &str,
    conversation: openai::Conversation,
) -> TauriResult<()> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        gpt.save_conversation(filename, &conversation).unwrap();
        ss.add_and_commit(&format!(
            "saved chatgpt conversation {}",
            &conversation.title.unwrap_or("".to_string())
        ))?;
    }
    Ok(())
}
#[tauri::command]
fn chatgpt_get_api_key() -> Option<String> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        Some(gpt.get_api_key())
    } else {
        None
    }
}

//these are 'string' histories for search and so on
#[tauri::command]
fn history_get(name: &str) -> Vec<String> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let res = ss.history_get(name);
    res.unwrap_or_default()
}
#[tauri::command]
fn history_store(name: &str, entries: Vec<String>) -> bool {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let res = ss.history_store(name, &entries);
    res.is_ok()
}
#[tauri::command]
fn start_terminal(folder: String) -> bool {
    let process = std::process::Command::new("kitty")
        .current_dir(folder)
        .spawn();
    //    .expect("Failed to spawn kitty");
    process.is_ok()
}

#[tauri::command]
fn extract_tags(text: &str) -> Vec<String> {
    Node::extract_tags(text).into_iter().collect()
}

#[tauri::command]
fn get_git_history(limit: Option<u32>) -> Vec<storage::GitHistoryEntry> {
    let actual_limit = limit.unwrap_or(100);
    let ss = STORAGE.get().unwrap().lock().unwrap();
    let res = ss.get_git_history(actual_limit);
    match res {
        Ok(x) => x,
        Err(err) => {
            dbg!(err);
            Vec::new()
        }
    }
}

#[tauri::command]
fn git_undo(hash: &str) -> Option<String> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    match ss.git_undo(hash) {
        Ok(_) => None,
        Err(err) => Some(err.to_string()),
    }
}

fn get_from_settings_str_map(key: &str) -> Option<HashMap<String, String>> {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    Some(
        ss.settings
            .get(key)?
            .as_table()?
            .iter()
            .map(|(k, v)| (Some((k.to_string(), v.as_str()?.to_string()))))
            .filter_map(|x| x)
            .collect(),
    )
}

fn save_to_settings_str_map(key: &str, map: HashMap<String, String>) -> Result<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let mut table = toml_edit::Table::new();
    for (k, v) in map {
        table.insert(&k, toml_edit::Item::Value(v.into()));
    }
    ss.settings.insert(key, toml_edit::Item::Table(table));
    ss.store_settings()
}

fn find_git_binary() -> Result<String> {
    Ok(std::str::from_utf8(
        &std::process::Command::new("which")
            .arg("git")
            .output()
            .context("find git binary with which failed")?
            .stdout,
    )?
    .trim()
    .to_string())
}

fn init_data_path(data_path: &PathBuf) -> Result<()> {
    let root = data_path.join("node.florg");
    std::fs::write(root, include_str!("welcome.florg"))?;
    Ok(())
}

fn init_data_path_git(data_path: &PathBuf, git_binary: &str) -> Result<()> {
    dbg!(git_binary);
    //check if it has a .git, or any parent path has
    let mut has_git = false;
    let mut current_path = data_path.clone();
    while current_path != PathBuf::from("/") {
        if current_path.join(".git").exists() {
            has_git = true;
            break;
        }
        current_path = current_path.parent().unwrap().to_path_buf();
    }
    if !has_git {
        std::process::Command::new(git_binary)
            .arg("init")
            .arg(".")
            .current_dir(data_path)
            .status()
            .context("Git init failed")?;
    }
    std::process::Command::new(git_binary)
        .arg("add")
        .arg(".")
        .arg("--all")
        .current_dir(data_path)
        .status()
        .context("git add . --all failed")?;
    std::process::Command::new(git_binary)
        .arg("commit")
        .arg("-m")
        .arg("Capturing status quo on new florg data path")
        .current_dir(data_path)
        .status()
        .context("git initial commit failed")?;

    Ok(())
}

fn init_data_path_gitignore(data_path: &PathBuf) -> Result<()> {
    Ok(std::fs::write(
        data_path.join(".gitignore"),
        "*.temp.adoc
*.cache
",
    )?)
}

fn editor_ended() {
    let mut remove: Vec<(usize, String, Option<String>)> = Vec::new();
    {
        let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
        for (ii, entry) in lock.open_editors.iter_mut().enumerate() {
            match entry.process.try_wait() {
                Ok(res) => match res {
                    Some(exit_status) => {
                        if exit_status.success() {
                            let raw = std::fs::read_to_string(&entry.temp_file)
                                .expect("Failed to read tempfile");
                            //dbg!(raw == entry.org_content);
                            //dbg!(&raw);
                            //dbg!(&entry.org_content);
                            if raw.is_empty() || (raw == entry.org_content) {
                                remove.push((ii, entry.path.to_string(), None))
                            } else {
                                remove.push((ii, entry.path.to_string(), Some(raw)));
                            }
                        } else {
                            remove.push((ii, entry.path.to_string(), None))
                        }
                    }
                    None => {} // still processing
                },
                Err(_) => panic!("error return from child process?!"),
            }
        }
    }
    for (ii, path, result) in remove {
        let window_title = {
            let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
            if lock.open_editors[ii].remove_after {
                std::fs::remove_file(&lock.open_editors[ii].temp_file).ok();
            }
            let window_title = lock.open_editors[ii].window_title.clone();
            lock.open_editors.remove(ii);
            window_title
        };

        match result {
            Some(raw) => {
                println!("Received {path}. nvim exit was success, content was there");
                update_from_edited_file(&path, raw, &window_title).unwrap();
            }
            None => {
                if path != "settings.toml" {
                    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
                    let mut ss = STORAGE.get().unwrap().lock().unwrap();
                    ss.remove_placeholder(
                        &TreePath::from_human(&path)
                            .expect("Failed to parse path from open editors"),
                    ); //todo
                    lock.app_handle.emit_all("node-unchanged", &path).ok();
                }
            }
        }
    }
}

fn update_from_edited_file(
    path: &str,
    raw_contents: String,
    window_title: &str,
) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();

    if path == "settings.toml" {
        println!("received settings");
        let settings = Storage::load_settings(&ss.data_path, Some(raw_contents.clone()));
        dbg!("Received settings", &settings);
        match settings {
            Ok(new_settings) => {
                ss.settings = new_settings;
                println!("Updated settings {:?}", &ss.settings);
                ss.store_settings()?;
                lock.app_handle.emit_all("message", "Settings updated").ok();
            }
            Err(_) => {
                lock.app_handle
                    .emit_all(
                        "message",
                        "<span class='error'>Settings could not be parsed. Try again</span>",
                    )
                    .ok();
                let tf = get_settings_temp_filename(&ss);
                println!("Writing contents - len {}", raw_contents.len());
                std::fs::write(&tf, &raw_contents).expect("failed to write settings temp file");
                spawn_settings_editor(tf, raw_contents, &mut lock);
            }
        }
    } else {
        let content = parse_raw_content(&raw_contents);
        println!("parsed contents {}", path);

        let node = storage::Node::new(
            &TreePath::from_human(path).expect("Failed to parse treepath from open editor"),
            content,
        );
        ss.replace_node(node, true)?;
        println!("Replaced node");

        lock.app_handle
            .emit_to(window_title, "node-changed", path)
            .ok();
        println!("Told editor");
    }
    TauriResult::Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let data_path = if args.len() > 1 {
        PathBuf::from(args[1].to_string())
    } else {
        std::env::var("FLORG_DATAPATH")
            .map(|x| PathBuf::from(x))
            .unwrap_or_else(|_| {
                xdg::BaseDirectories::with_prefix("florg")
                    .expect("failed to xdg::BaseDirectories")
                    .create_state_directory("")
                    .expect("Failed to create state directory in xdg state path")
            })
    };
    let git_binary = find_git_binary()?;

    if data_path
        .read_dir()
        .expect("could not find data_path")
        .next()
        .is_none()
    {
        init_data_path(&data_path).expect("Could not write initial contents");
    }
    if !(data_path.join(".git")).exists() {
        init_data_path_git(&data_path, &git_binary).expect("failed to init git dir");
    }
    if !(data_path.join(".gitignore")).exists() {
        init_data_path_gitignore(&data_path).expect("failed to init gitignore");
    }

    let s = Mutex::new(Storage::new(data_path, git_binary));
    STORAGE.set(s).unwrap();

    let mut signals = Signals::new(&[signal_hook::consts::SIGCHLD])?;
    let signal_handle = signals.handle();

    let jt = thread::spawn(move || {
        for sig in signals.forever() {
            // println!("received a signal {:?}", sig);
            if sig == signal_hook::consts::SIGCHLD {
                editor_ended()
            }
        }
    });

    fn get_mail_setting(key: &str) -> Option<String> {
        let ss = STORAGE.get().unwrap().lock().unwrap();
        Some(
            ss.settings
                .get("mail")?
                .as_table()?
                .get(key)?
                .as_str()?
                .to_string(),
        )
    }
    let mail_path = get_mail_setting("mail_dir").unwrap_or_else(|| {
        dirs::home_dir()
            .expect(
                "Could not find home dir, can't guess maildir, set mail.mail_dir in settings.toml",
            )
            .join("mail")
            .to_string_lossy()
            .to_string()
    });
    let config_path = get_mail_setting("config_path").unwrap_or_else(|| {
        dirs::home_dir()
            .expect(
                "Could not find home dir, can't guess notmuch config path, \
                                set mail.config_path in settings.toml",
            )
            .join(".notmuch-config")
            .to_string_lossy()
            .to_string()
    });

    let mail_store = mail::MailStore::new(&mail_path, &config_path);

    {
        use rdev::{listen, Event};
        thread::spawn(|| {
            listen(|event: Event| {
                match event.event_type {
                    rdev::EventType::ButtonRelease(rdev::Button::Unknown(no)) => {
                        let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
                        lock.app_handle.emit_all("mouse-button-pressed", no).ok();
                    }
                    _ => {}
                };
            })
            .ok();
        });
    }

    tauri::Builder::default()
        .setup(|app| {
            RUNTIME_STATE
                .set(Mutex::new(RuntimeState::new(app.handle(), mail_store)))
                .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            edit_node,
            change_node_text,
            commit,
            get_node,
            get_node_title,
            get_node_folder_path,
            get_tree,
            get_parent,
            move_node,
            swap_node_with_previous,
            swap_node_with_next,
            delete_node,
            sort_children,
            compact_children,
            list_open_paths,
            date_to_path,
            create_calendar,
            reload_data,
            edit_settings,
            get_tags,
            get_bookmarks,
            set_bookmarks,
            get_nav,
            get_mail_search_folders,
            find_next_empty_child,
            ripgrep_below_node,
            get_cached_node,
            set_cached_node,
            query_mail,
            get_mail_message,
            get_mail_message_brief,
            mail_message_add_tags,
            mail_message_remove_tags,
            mail_message_toggle_tag,
            mail_message_store_attachments,
            mail_message_new,
            mail_get_tags,
            chatgpt_get_prompts,
            chatgpt_update_prompts,
            chatgpt_list_conversations,
            chatgpt_get_conversation,
            chatgpt_new_conversation,
            chatgpt_save_conversation,
            chatgpt_get_api_key,
            history_get,
            history_store,
            start_terminal,
            extract_tags,
            get_git_history,
            git_undo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    signal_handle.close();
    signal_handle.close();
    println!("tauri ended");
    jt.join().ok();
    Ok(())
}
