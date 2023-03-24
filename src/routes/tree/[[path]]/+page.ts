import { invoke } from "@tauri-apps/api/tauri";
import { render_text } from "./../../node/[[path]]/funcs";
import { descend_node, flattenObject } from "./funcs";

/** @type {import('./$types').PageLoad} */

export async function load({ params }: { params: any }) {
  //console.log(params);
  let path = params.path || "";
  //  let res = await descend_node(path);
  // return { "tree": flattenObject(res) };
  let tree = await invoke("get_tree", { path: path, maxDepth: 2 });
  let out = flattenObject(
    tree,
  );
  if ((out.length > 0) && (out[0].indention == "")) {
    out[0].indention = "(root)";
  }

  return {
    current_item: path,
    "flat": out,
    tree: tree,
    currently_edited: {},
  };
}
