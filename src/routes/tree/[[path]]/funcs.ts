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
export async function descend_node(path) {
  //console.log("getting", path);
  let node: any = await invoke("get_node", { path });
  let children = [];
  for (let ii = 0; ii < node.children.length; ii++) {
    let c = node.children[ii];
    let res = await descend_node(c.path);
    children.push(res);
  }
  //console.log(path, children.length);
  return {
    title: node.node.header.title,
    path: path,
    children: children,
    first_paragraph: node.node.header.first_paragraph,
    more_text: false, //todo
  };
}

export function patch_tree(tree, path, new_children) {
  if (tree.path == path) {
    tree.children = new_children;
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

export function patch_tree_content(tree, path, new_text) {
  if (tree.path == path) {
    tree.raw = new_text;
    return true;
  } else {
    for (let ii = 0; ii < tree.children.length; ii++) {
      let child = tree.children[ii];
      if (patch_tree_content(child, path, new_text)) {
        return true;
      }
    }
  }
  return false;
}
