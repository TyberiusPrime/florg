import { invoke } from "@tauri-apps/api/tauri";
import { render_text } from "./../../node/[[path]]/funcs";
import { expand_path, flattenObject, patch_tree } from "./funcs";

/** @type {import('./$types').PageLoad} */

export async function load({ params }: { params: any }) {
  //console.log(params);
  let path: string = params.path || "";
  //  let res = await descend_node(path);
  // return { "tree": flattenObject(res) };
  let tree = await invoke("get_tree", { path: "", maxDepth: 2 });
  //itterate path and patch tree
  await expand_path(tree, path);

  let out = flattenObject(
    tree,
  );
  if ((out.length > 0) && (out[0].indention == "")) {
    out[0].indention = "(root)";
  }

  let start_item = 0;
  for (let ii = 0; ii < out.length; ii++) {
    if (out[ii].path == path) {
      start_item = ii;
      break;
    }
  }

  return {
    current_item: path,
    "flat": out,
    tree: tree,
    currently_edited: {},
    start_item: start_item,
    tags: await invoke("get_tags", {}),
    bookmarks: await invoke("get_bookmarks", {}) ?? {},
  };
}
