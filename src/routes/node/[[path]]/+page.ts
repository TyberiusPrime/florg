import { invoke } from "@tauri-apps/api/tauri";
import {render_text} from "./funcs";

/** @type {import('./$types').PageLoad} */
export async function load({ params }: {params: any}) {
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
  let rendered_cached = await invoke("get_cached_node", { path });
  if (rendered_cached == null) {
    let start_time = performance.now();

    console.log(res.raw);

    res.rendered = await render_text(res.raw);

    let end_time = performance.now();
    if (end_time - start_time > 100) {
      // probably just as fast to not cache...
      await invoke("set_cached_node", {
        path: path,
        raw: res.raw,
        rendered: res.rendered,
      });
    }
  } else {
    res.rendered = rendered_cached;
  }

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
