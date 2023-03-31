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
    tags: obj.tags,
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
export function patch_tags(tree, path, new_tags) {
  if (tree.path == path) {
    tree.tags = new_tags;
    return true;
  } else {
    for (let ii = 0; ii < tree.children.length; ii++) {
      let child = tree.children[ii];
      if (patch_tags(child, path, new_tags)) {
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
  for (let ii = 0; ii < path.length; ii++) {
    let p = path.slice(0, ii + 1);
    console.log("fetch tree", p);
    let node = await invoke("get_tree", { path: p, maxDepth: maxDepth });
    console.log(node);
    patch_tree(tree, p, node.children);
    console.log(tree);
  }
}

export function find_siblings(flat, index) {
  let path = flat[index].path;
  let parent = path.substring(0, path.length - 1);
  let prev = null;
  let next = null;
  let new_index = index - 1;
  while (new_index > 0) {
    if (flat[new_index].path.length == path.length) {
      if (flat[new_index].path.startsWith(parent)) {
        prev = flat[new_index].path;
        break;
      } else if (flat[new_index].path.length < path.length) {
        break;
      }
    }
    new_index -= 1;
  }
  new_index = index + 1;
  while (new_index < flat.length) {
    if (flat[new_index].path.length == path.length) {
      if (
        flat[new_index].path.startsWith(parent)
      ) {
        next = flat[new_index].path;
        break;
      }
    } else if (flat[new_index].path.length < path.length) {
      break;
    }
    new_index += 1;
  }

  return [prev, next];
}
