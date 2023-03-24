import { invoke } from "@tauri-apps/api/tauri";
import { render_text, render_text_cached } from "./funcs";

/** @type {import('./$types').PageLoad} */
export async function load({ params }: { params: any }) {
  console.log(params);
  let path = params.path || "";
  console.log(path);

  //console.log("load node", path);
  let node: any = await invoke("get_node", { path });
  let res: any = {};
  if (node.node != null) {
    //console.log(node);
    res.raw = node.node.raw;
    res.title = node.node.header.title; //only used for root node.
  } else {
    res.raw = "(empty node - enter to create)";
    res.title = "(empty node)";
  }
  res.rendered = render_text_cached(path, res.raw);

  let children = [];
  node.children.forEach((c) => {
    children.push({
      key: c.path.slice(-1),
      text: c.header.title,
      hover: c.header.first_paragraph,
    });
  });
  res.children = children;
  res.path = path;
  res.levels = node.levels;
  let open_paths = await invoke("list_open_paths");
  res.currently_edited = open_paths.indexOf(path) > -1;

  console.log("loaded node", res.path);
  return res;
}
