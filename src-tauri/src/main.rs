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
use storage::{Node, Storage};
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

#[derive(Debug, Serialize)]
enum TauriResult<T> {
    Ok(T),
    Err(TauriError),
}

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
pub(crate) struct NodeForJS {
    pub node: Option<Node>,
    pub levels: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub tags: Vec<String>,
}

#[tauri::command]
fn get_node(path: &str) -> NodeForJS {
    let s = STORAGE.get().unwrap().lock().unwrap();
    let node = s.get_node(path).map(|x| x.clone());
    let children = s.children_for(path).iter().map(|x| (*x).clone()).collect();
    let tags = node.as_ref().map_or_else(|| Vec::new(), |x| x.get_tags());
    NodeForJS {
        node,
        levels: s.levels(path),
        children,
        tags: tags,
    }
}

#[tauri::command]
fn get_node_title(path: &str) -> Option<String> {
    let s = STORAGE.get().unwrap().lock().unwrap();
    s.get_node(path).map(|x| x.header.title.clone())
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

fn descend(path: &str, storage: &MutexGuard<Storage>, remaining_depth: i32) -> Option<TreeForJS> {
    let node = storage.get_node(path);
    {
        let children = storage.children_paths_for(path);
        let has_children = !children.is_empty();
        Some(TreeForJS {
            path: path.to_string(),
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
    descend(path, &s, max_depth)
}

#[tauri::command]
fn move_node(org_path: &str, new_path: &str) -> Option<String> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    match s.move_node(org_path, new_path, true) {
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
    match s.swap_node_with_previous(path) {
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
    match s.swap_node_with_next(path) {
        Ok(_) => None,
        Err(e) => {
            println!("{:?}", &e);
            Some(e.to_string())
        }
    }
}

#[tauri::command]
fn change_node_text(path: &str, text: &str) {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let node = Node::new(path, text);
    ss.replace_node(node, true);
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
    lock.app_handle.emit_all("node-changed", path).ok();
}
#[tauri::command]
fn get_node_folder_path(path: &str) -> String {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    Node::dirname_from_path(&ss.data_path, path)
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn delete_node(path: &str) -> Result<(), String> {
    let mut s = STORAGE.get().unwrap().lock().unwrap();
    s.delete_node(path)?;
    Ok(())
}

#[tauri::command]
fn edit_node(path: &str, window_title: &str, new_text: Option<&str>) -> bool {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let mut runtime_state = RUNTIME_STATE.get().unwrap().lock().unwrap();
    if runtime_state
        .open_editors
        .iter()
        .filter(|entry| entry.path == path)
        .next()
        .is_none()
    {
        {
            let node_folder = Node::dirname_from_path(&ss.data_path, path);
            let tf_folder = node_folder;
            std::fs::create_dir_all(&tf_folder).unwrap();
            let tf = tf_folder.join(format!("{}.temp{}", path, storage::FLORG_SUFFIX));

            let mut content = "".to_string();
            let mut skip_lines = 0;
            if path.is_empty() {
                content += "(root)\n";
                skip_lines = 3;
            } else {
                let mut so_far = "".to_string();

                for (ii, letter) in path.chars().enumerate() {
                    content += &format!(
                        "{} {}{}\n",
                        letter,
                        " ".repeat(ii),
                        ss.get_node(&so_far)
                            .map(|x| &x.header.title[..])
                            .unwrap_or("")
                    );
                    so_far.push(letter);
                    skip_lines = ii + 5;
                }
            }
            content += "Content after the next line. First line = new title \n--\n\n";
            let node = ss.get_node(path).map(|x| x.clone());
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
                let place_holder = Node::new(path, "(placeholder)");
                ss.replace_node(place_holder, false);
            }
            edit_file(
                tf,
                content,
                skip_lines,
                "node-temp-changed",
                path.to_string(),
                window_title,
                &mut runtime_state,
                true,
            );
        }

        true
    } else {
        false
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
    Some(chrono_date_to_path(date))
}

fn chrono_date_month_to_path(date: chrono::NaiveDate) -> &'static str {
    match date.month() {
        1 => "A",
        2 => "B",
        3 => "C",
        4 => "D",
        5 => "E",
        6 => "F",
        7 => "G",
        8 => "H",
        9 => "I",
        10 => "J",
        11 => "K",
        12 => "L",
        _ => unreachable!(),
    }
}

fn chrono_date_to_path(date: chrono::NaiveDate) -> String {
    let mut kw = date.iso_week().week0();
    //range is 0..52
    let quarter = kw / 13;
    kw = kw % 13;
    let mut path = match quarter {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        _ => unreachable!(),
    }
    .to_string();
    path.push(match kw {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        4 => 'E',
        5 => 'F',
        6 => 'G',
        7 => 'H',
        8 => 'I',
        9 => 'J',
        10 => 'K',
        11 => 'L',
        12 => 'M',
        _ => unreachable!(),
    });
    let weekday = date.weekday();
    path.push(match weekday {
        chrono::Weekday::Mon => 'A',
        chrono::Weekday::Tue => 'B',
        chrono::Weekday::Wed => 'C',
        chrono::Weekday::Thu => 'D',
        chrono::Weekday::Fri => 'E',
        chrono::Weekday::Sat => 'F',
        chrono::Weekday::Sun => 'G',
    });
    return path;
}

#[tauri::command]
fn create_calendar(parent_path: &str, year: i32) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    if !ss.children_for(parent_path).is_empty() {
        TauriResult::<()>::Err(TauriError(anyhow!(
            "Node had children - not filling in calendar nodes"
        )));
    }
    /* for month in 1..13 {
        let date = chrono::NaiveDate::from_ymd_opt(year as i32, month, 1).unwrap();
        let path = format!("{parent_path}{}", chrono_date_month_to_path(date));
        let text = date.format("%b %Y\n").to_string();
        println!("{path} {text}");
        let node = Node::new(&path, &text);
        ss.replace_node(node, false);
    }
    println!("added months"); */
    for q in 1..5 {
        let path = format!(
            "{parent_path}{}",
            match q {
                1 => "A",
                2 => "B",
                3 => "C",
                4 => "D",
                _ => unreachable!(),
            }
        );
        let text = format!("Q{q}/{year}");
        let node = Node::new(&path, &text);
        ss.replace_node(node, false);
    }
    let mut last_kw = 500;
    let start = chrono::NaiveDate::from_ymd_opt(year as i32, 1, 1).unwrap();
    for date in start.iter_days().take_while(|x| x.year() == year) {
        let kw = date.iso_week().week();
        if kw != last_kw {
            let mut pp = chrono_date_to_path(date);
            pp.pop();
            let path = format!("{parent_path}{}", pp);
            let text = format!("KW {kw}\n");
            let node = Node::new(&path, &text);
            ss.replace_node(node, false);
            last_kw = kw;
        }
        let path = format!("{parent_path}{}", chrono_date_to_path(date));
        let text = date.format("%Y-%m-%d %a\n").to_string();
        let node = Node::new(&path, &text);
        ss.replace_node(node, false);
    }
    println!("added days");
    ss.add_and_commit(&format!("Added date notes below {parent_path}"));
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
fn get_nav() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("nav")
}

#[tauri::command]
fn get_mail_search_folders() -> Option<HashMap<String, String>> {
    get_from_settings_str_map("mail_queries")
}

#[tauri::command]
fn find_next_empty_child(path: &str) -> String {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    ss.find_next_empty_child(path)
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
    let search_path = Node::dirname_from_path(&ss.data_path, query_path);
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
            let mut result = Vec::new();
            for block in stdout.split("\n\n") {
                if (block.trim().is_empty()) {
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
                        filename.replace("/", "")
                    }
                    None => continue,
                };
                let path = format!("{query_path}{path}");
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
                    path,
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
    ss.get_cached_node(path)
}

#[tauri::command]
fn set_cached_node(path: &str, raw: &str, rendered: &str) -> bool {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    match ss.set_cached_node(path, raw, rendered) {
        Ok(_) => true,
        Err(_) => false, //todo: error handling
    }
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
fn chatgpt_save_conversation(filename: &str, conversation: openai::Conversation) {
    let ss = STORAGE.get().unwrap().lock().unwrap();
    if let Some(gpt) = &ss.chatgpt {
        gpt.save_conversation(filename, &conversation).unwrap();
        ss.add_and_commit(&format!(
            "saved chatgpt conversation {}",
            &conversation.title.unwrap_or("".to_string())
        ));
    }
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
    std::process::Command::new(git_binary)
        .arg("init")
        .arg(".")
        .current_dir(data_path)
        .status()
        .context("Git init failed")?;
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
                update_from_edited_file(&path, raw, &window_title)
            }
            None => {
                let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
                let mut ss = STORAGE.get().unwrap().lock().unwrap();
                ss.remove_placeholder(&path);
                lock.app_handle.emit_all("node-unchanged", &path).ok();
            }
        }
    }
}

fn update_from_edited_file(path: &str, raw_contents: String, window_title: &str) {
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
                ss.store_settings();
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
        println!("parsed contents");

        let node = storage::Node::new(path, content);
        ss.replace_node(node, true);
        println!("Replaced node");

        lock.app_handle
            .emit_to(window_title, "node-changed", path)
            .ok();
        println!("Told editor");
    }
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
            get_node,
            get_node_title,
            get_node_folder_path,
            get_tree,
            move_node,
            swap_node_with_previous,
            swap_node_with_next,
            delete_node,
            list_open_paths,
            date_to_path,
            create_calendar,
            reload_data,
            edit_settings,
            get_tags,
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
