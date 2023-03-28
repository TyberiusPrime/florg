import { invoke } from "@tauri-apps/api/tauri";

export function flattenObject(
  obj,
  depth = 0,
  indention = "",
  parent_is_last = [],
) {
  let result = [];

  result.push({
    indention,
    depth,
    title: obj.title,
    path: obj.path,
    first_paragraph: obj.first_paragraph,
    more_text: obj.more_text,
    has_children: obj.has_children,
    children_shown: obj.has_children && (obj.children.length > 0),
  });

  const len = obj.children ? obj.children.length : 0;
  for (let i = 0; i < len; i++) {
    const child = obj.children[i];
    const isLast = i === len - 1;
    const childIndention = isLast ? "└" : "├";
    let parentIndention = "";
    for (let yy = 0; yy < parent_is_last.length; yy++) {
      if (parent_is_last[yy]) {
        parentIndention += "&nbsp;";
      } else {
        parentIndention += "┆";
      }
    }
    parent_is_last.push(isLast);
    const childResult = flattenObject(
      child,
      depth + 1,
      parentIndention + childIndention,
      parent_is_last,
    );
    parent_is_last.pop();
    result.push(...childResult);
  }

  return result;
}

export function patch_tree(tree, path, new_children) {
  if (tree.path == path) {
    tree.children = new_children;
    tree.children_shown = new_children.length > 0;
    tree.has_children = tree.has_children || (new_children.length > 0);
    return true;
  } else {
    for (let ii = 0; ii < tree.children.length; ii++) {
      let child = tree.children[ii];
      if (patch_tree(child, path, new_children)) {
        return true;
      }
    }
  }
  return false;
}

export function delete_from_tree(tree, path) {
  for (let ii = 0; ii < tree.children.length; ii++) {
    let child = tree.children[ii];
    if (child.path == path) {
      tree.children.splice(ii, 1);
      console.log("found and deleted", path);
	  console.log(tree.children);
      return true;
    } else if (path.startsWith(child.path)) {
      delete_from_tree(child, path);
    }
  }
}

export async function expand_path(tree, path: string, maxDepth: int = 2) {
  console.log("expand_path", path);
  for (let ii = 0; ii < path.length; ii++) {
    let p = path.slice(0, ii + 1);
    console.log("fetch tree", p);
    let node = await invoke("get_tree", { path: p, maxDepth: maxDepth });
    patch_tree(tree, p, node.children);
    console.log(tree);
  }
}
