import { invoke } from "@tauri-apps/api/tauri";
import { goto } from "$app/navigation";
import { iso_date } from "$lib/util";

export async function parse_path(path: string) {
  if (path.startsWith("!")) {
    let prefix = path.slice(1);
    let date_suffix = await invoke("date_to_path", {
      dateStr: iso_date(new Date()),
    });
    path = prefix + date_suffix;
  } else if (path.startsWith("#")) {
  }
  return path;
}

export async function goto_node(path: string) {
  let ppath = await parse_path(path);
  if (ppath.startsWith("#")) {
    console.log("todo");
    //overlay = "datenav";
    //date_nav_target = path.slice(1);
  } else {
    goto("/node/" + ppath);
  }
}
