import { invoke } from "@tauri-apps/api/tauri";
import { goto } from "$app/navigation";
import { iso_date } from "$lib/util";
import { toast } from "@zerodevx/svelte-toast";

export async function parse_path(path: string) {
  let mode = "node";
  if (path.startsWith("!")) {
    let prefix = path.slice(1);
    let date_suffix = await invoke("date_to_path", {
      dateStr: iso_date(new Date()),
    });
    path = prefix + date_suffix;
    mode = "node";
  } else if (path.startsWith("#")) {
    mode = "date";
  } else if (path.startsWith("mail")) {
    mode = "mail";
    path = path.slice(5);
  } 
  return [mode, path];
}

export async function goto_node(path: string) {
  let [mode, ppath] = await parse_path(path);
  if (mode == "node") {
    goto("/node/" + ppath);
  } else if (mode == "mail") {
	  goto("/mail/query/" + encodeURIComponent(ppath));
  } else if (mode == "date") {
    toast.push("todo");
    //overlay = "datenav";
    //date_nav_target = path.slice(1);
  } else {
    toast.push(`unknown path mode ${mode}`);
  }
}
