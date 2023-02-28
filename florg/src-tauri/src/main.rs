#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod storage;
use anyhow::{anyhow, bail, Context, Result};
use chrono::Datelike;
use once_cell::sync::OnceCell;
use serde::Serialize;
use signal_hook::iterator::Signals;
use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::Mutex,
    thread::{self},
};
use storage::{Node, Storage};
use tauri::Manager;

static STORAGE: OnceCell<Mutex<Storage>> = OnceCell::new();

pub struct OpenEditor {
    path: String,
    temp_file: tempfile::NamedTempFile,
    process: std::process::Child,
}

#[derive(Debug)]
struct RuntimeState {
    open_editors: Vec<OpenEditor>,
    app_handle: tauri::AppHandle,
}

impl RuntimeState {
    fn new(handle: tauri::AppHandle) -> RuntimeState {
        RuntimeState {
            open_editors: Vec::new(),
            app_handle: handle,
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
            .field("temp_file", &self.temp_file.path().file_name())
            .field("process", &self.process.id())
            .finish()
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NodeForJS {
    pub node: Option<Node>,
    pub levels: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[tauri::command]
fn get_node(path: &str) -> NodeForJS {
    let s = STORAGE.get().unwrap().lock().unwrap();
    let node = s.get_node(path).map(|x| x.clone());
    let children = s.children_for(path).iter().map(|x| (*x).clone()).collect();
    NodeForJS {
        node,
        levels: s.levels(path),
        children,
    }
}

#[tauri::command]
fn edit_node(path: &str) -> bool {
    if RUNTIME_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .open_editors
        .iter()
        .filter(|entry| entry.path == path)
        .next()
        .is_none()
    {
        {
            let tf = tempfile::Builder::new()
                .prefix("florg")
                .suffix(".md")
                .tempfile()
                .expect("could not create tempfile?");
            let mut content = "".to_string();
            let mut skip_lines = 0;
            if path.is_empty() {
                content += "(root)\n";
                skip_lines = 2;
            } else {
                let mut so_far = "".to_string();

                for (ii, letter) in path.chars().enumerate() {
                    content += &format!(
                        "{} {}{}\n",
                        letter,
                        " ".repeat(ii),
                        STORAGE
                            .get()
                            .unwrap()
                            .lock()
                            .unwrap()
                            .get_node(&so_far)
                            .map(|x| &x.header.title[..])
                            .unwrap_or("")
                    );
                    so_far.push(letter);
                    skip_lines = ii + 3;
                }
            }
            content += "-- Content after this line. First line = new title --\n\n";
            let node = STORAGE
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .get_node(path)
                .map(|x| x.clone());
            content += node.as_ref().map(|x| &x.raw[..]).unwrap_or("");
            let tf_name = tf.path().to_str().expect("temp file was not unicode name?");
            std::fs::write(&tf_name, content).expect("temp file write failure");

            let process = std::process::Command::new("kitty")
                .arg("--")
                .arg("nvim")
                .arg(format!("+{}", skip_lines))
                .arg(tf_name)
                .spawn()
                .expect("Failed to spawn kitty&neovim");
            RUNTIME_STATE
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .open_editors
                .push(OpenEditor {
                    path: path.to_string(),
                    temp_file: tf,
                    process,
                });
        }

        true
    } else {
        false
    }
}

#[tauri::command]
fn edit_settings() -> bool {
    let path = "settings.toml";
    if RUNTIME_STATE
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .open_editors
        .iter()
        .filter(|entry| entry.path == path)
        .next()
        .is_none()
    {
        {
            let tf = tempfile::Builder::new()
                .prefix("florg-settings")
                .suffix(".toml")
                .tempfile()
                .expect("could not create tempfile?");
            let ss = STORAGE.get().unwrap().lock().unwrap();
            dbg!(&ss.settings);
            let content = ss.settings.to_string();
            let skip_lines = 0;

            let tf_name = tf.path().to_str().expect("temp file was not unicode name?");
            std::fs::write(&tf_name, content).expect("temp file write failure");

            let process = std::process::Command::new("kitty")
                .arg("--")
                .arg("nvim")
                .arg(format!("+{}", skip_lines))
                .arg(tf_name)
                .spawn()
                .expect("Failed to spawn kitty&neovim");
            RUNTIME_STATE
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .open_editors
                .push(OpenEditor {
                    path: path.to_string(),
                    temp_file: tf,
                    process,
                });
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
    let path_month = chrono_date_month_to_path(date);
    let path_day = match date.day() {
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
        13 => "M",
        14 => "N",
        15 => "O",
        16 => "P",
        17 => "Q",
        18 => "R",
        19 => "S",
        20 => "T",
        21 => "U",
        22 => "V",
        23 => "W",
        24 => "X",
        25 => "Y",
        26 => "ZA",
        27 => "ZB",
        28 => "ZC",
        29 => "ZD",
        30 => "ZE",
        31 => "ZF",
        _ => unreachable!(),
    };
    format!("{}{}", path_month, path_day)
}

#[tauri::command]
fn create_calendar(parent_path: &str, year: i32) -> TauriResult<()> {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    if !ss.children_for(parent_path).is_empty() {
        TauriResult::<()>::Err(TauriError(anyhow!(
            "Node had children - not filling in calendar nodes"
        )));
    }
    for month in 1..13 {
        let date = chrono::NaiveDate::from_ymd_opt(year as i32, month, 1).unwrap();
        let path = format!("{parent_path}{}", chrono_date_month_to_path(date));
        let text = date.format("%b %Y\n").to_string();
        println!("{path} {text}");
        let node = Node::new(&path, &text);
        ss.replace_node(node, false);
    }
    println!("added months");
    let start = chrono::NaiveDate::from_ymd_opt(year as i32, 1, 1).unwrap();
    for date in start.iter_days().take_while(|x| x.year() == year) {
        let path = format!("{parent_path}{}", chrono_date_to_path(date));
        let text = date.format("%Y-%m-%d (%a KW %V)\n").to_string();
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

fn editor_ended() {
    let mut remove: Vec<(usize, Option<(String, String)>)> = Vec::new();
    {
        let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
        for (ii, entry) in lock.open_editors.iter_mut().enumerate() {
            match entry.process.try_wait() {
                Ok(res) => match res {
                    Some(exit_status) => {
                        if exit_status.success() {
                            let raw = std::fs::read_to_string(entry.temp_file.path())
                                .expect("Failed to read tempfile");
                            if !raw.is_empty() {
                                remove.push((ii, Some((entry.path.to_string(), raw))));
                            } else {
                                remove.push((ii, None))
                            }
                        } else {
                            remove.push((ii, None))
                        }
                    }
                    None => {} // still processing
                },
                Err(_) => panic!("error return from child process?!"),
            }
        }
    }
    for (ii, result) in remove {
        match result {
            Some((path, raw)) => {
                println!("Received {path}");
                if !raw.is_empty() {
                    update_from_edited_file(&path, raw);
                }
            }
            None => {
                let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
                lock.app_handle.emit_all("node-unchanged", 0).ok();
            }
        }
        let mut lock = RUNTIME_STATE.get().unwrap().lock().unwrap();
        lock.open_editors.remove(ii);
    }
}

fn update_from_edited_file(path: &str, raw_contents: String) {
    let mut ss = STORAGE.get().unwrap().lock().unwrap();
    let lock = RUNTIME_STATE.get().unwrap().lock().unwrap();

    if path == "settings.toml" {
        println!("received settings");
        let settings = Storage::load_settings(&ss.data_path, Some(raw_contents.clone()));
        dbg!("Received settings", &settings);
        match settings {
            Ok(new_settings) => {
                ss.settings = new_settings;
                ss.store_settings();
                lock.app_handle.emit_all("message", "Settings updated").ok();
            }
            Err(_) => {
                lock.app_handle.emit_all("message", "<span class='error'>Settings could not be parsed, changes thrown away. Try again</span>").ok();
            }
        }
    } else {
        let content = match raw_contents.split_once("--\n") {
            Some((_header, content)) => content.trim(),
            None => &raw_contents,
        };

        let node = storage::Node::new(path, content);
        ss.replace_node(node, true);

        lock.app_handle.emit_all("node-changed", path).ok();
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let data_path = if args.len() > 1 {
        PathBuf::from(args[1].to_string())
    } else {
        xdg::BaseDirectories::with_prefix("florg")
            .expect("failed to xdg::BaseDirectories")
            .create_state_directory("")
            .expect("Failed to create state directory in xdg state path")
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

    tauri::Builder::default()
        .setup(|app| {
            RUNTIME_STATE
                .set(Mutex::new(RuntimeState::new(app.handle())))
                .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            edit_node,
            get_node,
            list_open_paths,
            date_to_path,
            create_calendar,
            reload_data,
            edit_settings,
            get_tags,
            get_nav,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    signal_handle.close();
    println!("tauri ended");
    jt.join().ok();
    Ok(())
}
