<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import Expander from "$lib/../components/Expander.svelte";
  import Focusable from "$lib/../components/Focusable.svelte";
  import View from "$lib/../components/View.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Search from "$lib/../components/Search.svelte";
  import DateInput from "$lib/../components/DateInput.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { goto, invalidateAll } from "$app/navigation";
  import {
    render_text_cached,
    render_text,
  } from "$lib/../routes/node/[[path]]/funcs";
  import {
    patch_tree,
    patch_tags,
    flattenObject,
    expand_path,
    delete_from_tree,
    find_siblings,
  } from "./funcs";

  import { tag_class } from "$lib/colors.ts";
  import { onMount, onDestroy, beforeUpdate, afterUpdate, tick } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import {
    readText as read_clipboard,
    writeText as copy_to_clipboard,
  } from "@tauri-apps/api/clipboard";
  import {
    add_code_clipboards,
    dispatch_keyup,
    iso_date,
    iso_date_and_time,
    count_lines,
    render_tags,
  } from "$lib/util.ts";
  import { focus_first_in_node } from "$lib/util.ts";
  import { emit, listen } from "@tauri-apps/api/event";
  import { fetch as tauri_fetch } from "@tauri-apps/api/http";
  import asciidoctor from "asciidoctor";

  import readabilityLib from "@mozilla/readability";
  export let data;

  var Readability = readabilityLib.Readability;

  let viewComponent;
  let cache = {};
  let activeIndex;
  let scroll_to_active;
  let overlay = "";
  let nav_text = "";
  let nav_mode = "nav";
  let nav_path = "";
  let nav_start_index = 0;
  let nav_start_path = "";
  let nav_start_tree = null;
  let move_and_goto = false;
  let highlight_node = false;
  let fetch_url_text = "";
  let focus_time = Date.now() - 1000;
  let search_mode;
  let in_page_search_term = "";
  let date_pick_mode = "goto";
  let date_pick_prefix = "";
  let date_pick_value = iso_date(Date.now() - 24 * 60 * 60 * 1000);

  $: data.current_item = data?.flat[activeIndex]?.path;

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "g", text: "Go to node" },
    { key: "m", text: "Move below node" },
    { key: "M", text: "Move below node&got" },
    { key: "ctrl+m", text: "Move to/below path" },
    { key: "space", text: "Enter nav mode" },
    { key: "a", text: "add node below" },
    { key: "d", text: "Delete node" },
    { key: "s", text: "search" },
    { key: "n", text: "search: (next hit)" },
    { key: "N", text: "search: (prev hit)" },
  ];
  let delete_entries = [
    { key: "d", text: "delete node & children", target_path: "delete" },
    { key: "m", text: "merge into text of parent", target_path: "merge" },
    { key: "p", text: "promote one level", target_path: "promote" },
  ];

  let copy_entries = [
    { key: "c", text: "link", target_path: "link" },
    { key: "y", text: "content", target_path: "content" },
    { key: "t", text: "title", target_path: "title" },
    { key: "p", text: "node folder path", target_path: "path" },
    { key: "r", text: "rendered_content", target_path: "rendered_content" },
  ];
  let palette_entries = [
    { key: "t", text: "Open terminal", target_path: "terminal" },
    { key: "s", text: "edit ettings", target_path: "settings" },
    { key: "d", text: "create_date_nodes", target_path: "create_date_nodes" },
    { key: "x", text: "exit the app", target_path: "exit" },
    { key: "r", text: "reload data", target_path: "reload" },
  ];

  let sub_section_entries = [
    { key: "x", text: "extract into subnode", target_path: "extract_subnode" },
  ];

  function map_tags(tags) {
    if (tags !== undefined) {
      let res = [];
      for (let key in tags) {
        let tag = tags[key];
        res.push({ key: key, text: tag, target_path: tag });
      }
      res.push({ key: "#", text: "A Date", target_path: "!!date" });
      return res;
    }
  }

  function start_nav() {
    nav_path = data.flat[activeIndex].path;
    nav_start_path = nav_path;
    nav_start_index = activeIndex;
    nav_start_tree = structuredClone(data.tree);
  }

  let tag_entries = map_tags(data.tags);
  let keys = {
    " ": () => {
      nav_text = "nav";
      nav_mode = "nav";
      start_nav();
      viewComponent.enter_overlay("nav");
    },
    h: () => {
      viewComponent.enter_overlay("help");
    },
    c: (ev) => {
      if (!ev.ctrlKey) {
        viewComponent.enter_overlay("copying");
      } else {
        return false;
      }
    },
    t: () => {
      viewComponent.enter_overlay("tag");
    },
    g: () => {
      viewComponent.enter_overlay("goto");
    },
    "#": () => {
      viewComponent.enter_overlay("date_pick");
      date_pick_mode = "goto";
      date_pick_prefix = data.current_item;
      date_pick_value = iso_date(Date.now());
      start_nav();
      return;
    },
    s: () => {
      viewComponent.enter_overlay("search");
      return true;
    },

    n: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      } else {
        viewComponent.enter_overlay("search");
        search_mode = "in_page";
      }
    },
    N: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      } else {
        viewComponent.enter_overlay("search");
        search_mode = "in_page";
      }
    },
    a: () => {
      add_node();
    },
    d: () => {
      viewComponent.enter_overlay("delete");
    },
    p: () => {
      viewComponent.enter_overlay("palette");
    },

    x: async () => {
      fetch_url_text = await read_clipboard();
      if (!fetch_url_text.startsWith("http")) {
        fetch_url_text = "";
      }
      viewComponent.enter_overlay("fetch_url");
    },

    m: (ev) => {
      if (ev.ctrlKey) {
        nav_text = `move ${data.current_item} - ${data.flat[activeIndex].title} to`;
        nav_mode = "move";
        nav_path = data.flat[activeIndex].path;
        nav_start_path = nav_path;
        nav_start_index = activeIndex;
        nav_start_tree = structuredClone(data.tree);
        viewComponent.enter_overlay("nav");
      } else {
        highlight_node = true;
        move_and_goto = false;
        viewComponent.enter_overlay("move");
      }
    },
    M: (ev) => {
      highlight_node = false;
      move_and_goto = true;
      viewComponent.enter_overlay("move");
    },
    u: (ev) => {
      goto("/undo");
    },
  };

  async function add_node() {
    let new_path = await invoke("find_next_empty_child", {
      path: data.current_item,
    });
    await invoke("edit_node", {
      path: new_path,
      windowTitle: appWindow.label,
    });
  }

  async function get_rendered_node(path) {
    if (data.currently_edited[path] != undefined) {
      let tags = await invoke("extract_tags", {
        text: data.currently_edited[path],
      });
      return (
        render_tags(tags) + (await render_text(data.currently_edited[path]))
      );
    }
    if (cache[path] === undefined) {
      let node = await invoke("get_node", { path: path + "" });
      if (node !== undefined && node.node !== null && node.node.raw != "") {
        let rt = await render_text_cached(path, node.node.raw);
        cache[path] = render_tags(node.tags) + rt;
      } else {
        cache[path] = "(empty node)";
      }
    }
    return cache[path];
  }

  async function itemChanged(ev) {
    let path = ev.detail.path;
    if (path !== undefined) {
      data.current_item = path;
      data = data;
    }
  }

  async function toggle_node(path, do_collapse) {
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (data.flat[ii].path == path) {
        if (data.flat[ii].has_children) {
          let was_expanded = false;
          let yy = ii + 1;
          if (yy > data.flat.length - 1) {
            was_expanded = false;
          } else {
            was_expanded = data.flat[yy].path.startsWith(path);
          }
          if (was_expanded && do_collapse) {
            patch_tree(data.tree, path, []);
            data.flat = flattenObject(data.tree);
            data = data;
          } else {
            let subtree = await invoke("get_tree", { path: path, maxDepth: 1 });
            patch_tree(data.tree, path, subtree.children);
            data.flat = flattenObject(data.tree);
            data = data;
          }
        }
        break;
      }
    }
  }

  async function item_toggle_children(ev) {
    let path = ev.detail.path;
    await toggle_node(path, true);
    window.setTimeout(() => {
      activeIndex = activeIndex;
    }, 10);
  }

  let can_expand = (path, do_expand) => {
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (data.flat[ii].path == path) {
        if (data.flat[ii].has_children) {
          let was_expanded = false;
          let yy = ii + 1;
          if (yy > data.flat.length - 1) {
            was_expanded = false;
          } else {
            was_expanded = data.flat[yy].path.startsWith(path);
          }
          if (do_expand && !was_expanded) {
            toggle_node(path, false);
          }
          return !was_expanded;
        }
      }
    }
    return false;
  };

  let can_contract = (path, do_contract) => {
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (data.flat[ii].path == path) {
        if (data.flat[ii].has_children) {
          let was_expanded = false;
          let yy = ii + 1;
          if (yy > data.flat.length - 1) {
            was_expanded = false;
          } else {
            was_expanded = data.flat[yy].path.startsWith(path);
          }
          if (do_contract && was_expanded) {
            toggle_node(path, true);
          }
          return was_expanded;
        }
      }
    }
    return false;
  };

  async function edit_current_node() {
    let path = data.flat[activeIndex].path;
    edit_node(path);
  }
  async function edit_node(path) {
    if (data.currently_edited[path] == undefined) {
      let node = await invoke("get_node", { path: path });
      data.currently_edited[path] = node?.node?.raw;
      data = data;
      await invoke("edit_node", {
        path: path,
        windowTitle: appWindow.label,
      });
    }
  }

  async function item_selected(ev) {
    edit_current_node();
  }
  //this is an event from rust
  const unlisten_node_changed = listen("node-changed", async (event) => {
    // a specific node was reread
    data.currently_edited[event.payload] = undefined;
    highlight_node = event.payload;
    let new_node = await invoke("get_node", { path: event.payload });
    cache[event.payload] = undefined;
    //patch_tree_content(data.tree, event.payload, new_node.node.raw);
    //data.flat = flattenObject(data.tree);
    let prefix = event.payload.substr(0, event.payload.length - 1);
    patch_tags(data.tree, event.payload, new_node.tags);
    let p = data.current_item;
    if (p.length <= 1) {
      data.tree = await invoke("get_tree", { path: "", maxDepth: 2 });
    } else {
      can_expand(event.payload, false);
      if (event.payload != p) {
        goto_node(p);
      }
    }

    data.flat = flattenObject(data.tree);
    data = data;
    window.setTimeout(async () => await appWindow.setFocus(), 100);
  });

  const unliste_node_unchanged = listen("node-unchanged", async (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing?
    data.currently_edited[event.payload] = undefined;
    data = data;
    window.setTimeout(async () => await appWindow.setFocus(), 100);
  });

  const unliste_node_temp_changed = listen(
    "node-temp-changed",
    async (event) => {
      //	toast.push(event.payload[0] + '-' + data.current_item);
      if (event.payload[0] == data.current_item) {
        //	    toast.push('ts')
        if (event.payload[1] != data.currently_edited[data.current_item]) {
          data.currently_edited[data.current_item] = event.payload[1];
          data = data;
        }
      }
    }
  );
  const unlisten_message = listen("message", (event) => {
    if (event.payload == "Settings updated") {
      toast.push("settings changed");
      window.location.reload();
    }
  });

  let clicked_section = null;

  function add_content_actions(mutationList, observer) {
    //console.log(mutationList);
    //toast.push("content change");
    //find all h2 in #node_content and add a button to the right
    let h2s = document.querySelectorAll(
      "#node_content h2, #node_content h3, #node_content h4, #node_content h5, #node_content h6"
    );
    for (let ii = 0; ii < h2s.length; ii++) {
      let h2 = h2s[ii];
      let btn = document.createElement("button");
      btn.innerHTML = "&#x2630;";
      btn.classList.add("small-button");
      btn.onclick = (ev) => {
        clicked_section = h2;
        viewComponent.enter_overlay("subsection-menu");
      };
      h2.appendChild(btn);
    }

    let tags = document.querySelectorAll(".tags");
    for (let ii = 0; ii < tags.length; ii++) {
      const handle_click = (ev) => {
        goto("/node_search/" + encodeURIComponent(ev.target.innerText));
      };
      tags[ii].removeEventListener("click", handle_click);
      tags[ii].addEventListener("click", handle_click);
    }
  }

  const content_observer = new MutationObserver(add_content_actions);
  onMount(async () => {
    await goto_node(data.flat[data.start_item].path);
    content_observer.observe(document.getElementById("node_content"), {
      attributes: false,
      childList: true,
      subtree: false,
    });
  });
  beforeUpdate(async () => {
    if (data.current_item != "" && activeIndex == undefined) {
      for (let ii = 0; ii < data.flat.length; ii++) {
        if (data.flat[ii].path == data.current_item) {
          activeIndex = ii;
          console.log("hit", ii);
          break;
        }
      }
    }
  });
  onDestroy(async () => {
    (await unlisten_node_changed)();
    (await unliste_node_unchanged)();
    (await unliste_node_temp_changed)();
    (await unlisten_message)();
    content_observer.disconnect();
  });

  async function handle_key_up_content(ev) {
    if (ev.key == "ArrowLeft") {
      focus_first_in_node(document.getElementById("tree_parent"));
      document
        .querySelector(".chosen")
        .scrollIntoView({ behaviour: "smooth", block: "center" });
    }
  }
  function show_help() {
    viewComponent.enter_overlay("help");
  }

  async function goto_node(path) {
    await expand_path(data.tree, path, 1);
    data.flat = flattenObject(data.tree);

    let found = false;
    let expanded = false;
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (path.startsWith(data.flat[ii].path)) {
        activeIndex = ii;
        if (path == data.flat[ii].path) {
          found = true;
          break;
        }
      } else if (data.flat[ii].path > path) {
        break;
      }
    }
    return found;
  }

  function leave_nav(ev) {
    activeIndex = nav_start_index;
    data.path = nav_start_path;
    data.tree = nav_start_tree;
    data.flat = flattenObject(data.tree);
    viewComponent.leave_overlay();
    data = data;
    focus_first_in_node(document.getElementById("tree_parent"));
    ev.stopPropagation();
    ev.preventDefault();
  }

  async function handle_nav_change(ev) {
    if (ev.key == "Escape") {
      leave_nav(ev);

      ev.stopPropagation();
      ev.preventDefault();
      return;
    } else if (ev.key == " ") {
      if (nav_mode == "nav") {
        viewComponent.leave_overlay();
        data = data;
        ev.stopPropagation();
        ev.preventDefault();
        return;
      }
    }
    nav_path = nav_path.toUpperCase();
    nav_path = nav_path.replace(/[^A-Z0-9]/g, "");
    let el = document.getElementById("nav_path_input");
    let found = await goto_node(nav_path);
    if (!found) {
      el.classList.add("notfound");
    } else {
      el.classList.remove("notfound");
    }
    await tick();
    document.querySelector(".chosen").scrollIntoView();

    if (ev.key == "Enter") {
      viewComponent.leave_overlay();
      if (nav_mode == "nav") {
        if (found) {
          edit_current_node();
        } else {
          edit_node(nav_path);
        }
      } else if (nav_mode == "move") {
        await move_by_nav(nav_path);
      }
    }

    ev.stopPropagation();
    //key is a..z
  }

  async function move_by_nav(nav_path) {
    let found = await goto_node(nav_path);
    let new_path;
    if (found) {
      new_path = await invoke("find_next_empty_child", {
        path: nav_path,
      });
    } else {
      new_path = nav_path;
    }
    let res = await invoke("move_node", {
      orgPath: nav_start_path,
      newPath: new_path,
    });
    if (res !== null) {
      toast.push(res);
    } else {
      let parent = data.current_item.substring(0, data.current_item.length - 1);
      let new_parent = new_path.substring(0, new_path.length - 1);
      await toggle_node(new_parent, true);
      await toggle_node(new_parent, false);

      await goto_node(new_path);
      await toggle_node(parent, true);
      await toggle_node(parent, false);
      await goto_node(new_path);
      scroll_to_active();
      highlight_node = new_path;
    }
  }

  async function parse_path(path) {
    if (path.startsWith("today:")) {
      let prefix = path.slice(6);
      let date_suffix = await invoke("date_to_path", {
        dateStr: iso_date(new Date()),
      });
      path = prefix + date_suffix;
    }
    return path;
  }

  let handle_goto = async (path) => {
    viewComponent.leave_overlay();
    if (path.startsWith("today:")) {
      let prefix = path.slice(6);
      let date_suffix = await invoke("date_to_path", {
        dateStr: iso_date(new Date()),
      });
      path = prefix + date_suffix;
    } else if (path.startsWith("date:")) {
      viewComponent.enter_overlay("date_pick");
      date_pick_mode = "goto";
      date_pick_prefix = path.slice(5);
      date_pick_value = iso_date(Date.now() - 24 * 60 * 60 * 1000);
      start_nav();
      return;
    } else if (path.startsWith("search:")) {
      let search_term = path.slice(7);
      goto("/node_search/" + encodeURIComponent(search_term));
      return;
    } else if (path.startsWith("agenda:")) {
      let days = path.slice(7);
      days = parseInt(days);
      if (isNaN(days)) {
        days = 30;
      }
      //start date is beginning of today
      let start_date = new Date();
      start_date = start_date.setUTCHours(0, 0, 0, 0);
      let stop_date = start_date + days * 24 * 60 * 60 * 1000;
      //date in ms since epoch
      goto("/agenda/" + start_date + "/" + stop_date);
      return;
    }

    if (path.startsWith("mail:")) {
      goto("/mail/query/" + path.slice(5));
      return;
    }
    try {
      await goto_node(path);
    } catch (e) {
      toast.push("Could not go to node" + e);
    }

    await tick();
    window.setTimeout(() => {
      focus_first_in_node(document.getElementById("tree_parent"));
      scroll_to_active();
    }, 10);
  };

  let handle_move = async (path) => {
    viewComponent.leave_overlay();
    let ppath = await parse_path(path);
    if (ppath.startsWith("date:")) {
      viewComponent.enter_overlay("date_pick");
      start_nav();
      date_pick_prefix = path.slice(5);
      date_pick_value = iso_date(Date.now() - 24 * 60 * 60 * 1000);
      date_pick_mode = "move";
      return;
    } else if (ppath.startsWith("mail:")) {
      toast.push("can't goto mail");
      return;
    }
    if (ppath.startsWith(data.current_item)) {
      if (data.current_item == ppath) {
        toast.push("Can't move to self");
      } else {
        toast.push("Can't move to own child");
      }
      return;
    }
    if (ppath == data.current_item.substring(0, data.current_item.length - 1)) {
      toast.push("Won't move on same level");
      return;
    }
    let new_path = await invoke("find_next_empty_child", {
      path: ppath,
    });
    //toast.push("moving to " + new_path);
    let siblings = find_siblings(data.flat, activeIndex);
    console.log("siblings", siblings);
    let res = await invoke("move_node", {
      orgPath: data.current_item,
      newPath: new_path,
    });
    if (res !== null) {
      toast.push(res);
    } else {
      let parent = data.current_item.substring(0, data.current_item.length - 1);
      let new_parent = new_path.substring(0, new_path.length - 1);
      //await toggle_node(new_parent, true);
      //await toggle_node(new_parent, false);
      //await expand_path(data.tree, new_path, 1);
      await toggle_node(new_parent, true);
      await toggle_node(new_parent, false);

      await goto_node(new_path);
      await toggle_node(parent, true);
      await toggle_node(parent, false);
      if (move_and_goto) {
        await goto_node(new_path);
      } else {
        if (siblings[0] !== null) {
          await goto_node(siblings[0]);
        } else if (siblings[1] !== null) {
          await goto_node(siblings[1]);
        } else await goto_node(parent);
      }
      scroll_to_active();
      highlight_node = new_path;
    }
  };

  async function handle_delete(ev) {
    let mode = ev.detail;
    console.log(ev);
    viewComponent.leave_overlay();
    if (mode == "delete") {
      await delete_current_node();
    } else if (mode == "merge") {
      await merge_current_node_with_parent();
    } else if (mode == "promote") {
      await move_node_to_parents_parent();
    } else {
      toast.push("Unknown delete mode " + mode);
    }
  }

  async function move_node_to_parents_parent() {
    let parent = data.current_item.substring(0, data.current_item.length - 1);
    let grandparent = parent.substring(0, parent.length - 1);
    if (grandparent == "") {
    } else {
      let new_path = await invoke("find_next_empty_child", {
        path: grandparent,
      });
      try {
        await invoke("move_node", {
          orgPath: data.current_item,
          newPath: new_path,
        });
        await toggle_node(grandparent, true);
        await toggle_node(grandparent, false);

        highlight_node = new_path;
        await goto_node(new_path);
      } catch (e) {
        toast.push("failed to promoto " + e);
      }
    }
  }

  async function merge_current_node_with_parent() {
    if (data.current_item == "") {
      toast.push("Can't merge root");
      return;
    }
    let node = await invoke("get_node", { path: data.current_item });
    let parent_path = data.current_item.substring(
      0,
      data.current_item.length - 1
    );
    let parent = await invoke("get_node", {
      path: parent_path,
    });
    let text = node.node.raw.replace(/([=]+)/g, "$1=");
    while (!text.startsWith("==")) {
      text = "=" + text;
    }
    text = parent.node.raw.trim() + "\n\n" + text;
    await invoke("change_node_text", { path: parent_path, text });
    await delete_current_node();
  }

  async function delete_current_node() {
    if (data.current_item != "") {
      await invoke("delete_node", { path: data.current_item })
        .then(async () => {
          toast.push("node deleted: " + data.current_item);
          let prefix = data.current_item.substring(
            0,
            data.current_item.length - 1
          );
          if (prefix != "") {
            await goto_node(prefix);
          } else {
            toggle_node("", true);
            let tree = await invoke("get_tree", { path: "", maxDepth: 2 });
            data.tree = tree;
            data.flat = flattenObject(data.tree);
            await goto_node("");
          }
        })
        .catch((e) => {
          toast.push(`Error ${e}`);
        });
      //delete_from_tree(data.tree, data.current_item);
      //console.log(data.tree);
      //data.flat = flattenObject(data.tree);
    }
  }
  async function create_date_nodes() {
    let last_path = data.current_item;
    let node = await invoke("get_node", { path: last_path });
    console.log(node);
    if (node.children.length > 0) {
      toast.push(
        "<span class='error'>Can not create date nodes on an node that already has children</span>"
      );
    } else {
      let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(
        new Date()
      );
      let year = window.prompt(
        "Please enter the year to create a calendar for",
        ye
      );
      if (year != null) {
        console.log(year);
        let year_parsed = parseInt(year);
        console.log(year_parsed);
        if (!isNaN(year_parsed)) {
          toast.push("Created calendar");
          await invoke("create_calendar", {
            parentPath: last_path,
            year: year_parsed,
          });
          await invoke("reload_data", {});
          await goto_node(last_path);
        }
      }
    }
  }
  async function handle_palette(ev) {
    viewComponent.leave_overlay();
    let cmd = ev.detail;
    if (cmd == "reload") {
      await invoke("reload_data", {});
      console.log("reloaded");
      invalidateAll();
    } else if (cmd == "exit") {
      await exit(1);
    } else if (cmd == "create_date_nodes") {
      await create_date_nodes();
    } else if (cmd == "settings") {
      return await invoke("edit_settings", {});
    } else if (cmd == "terminal") {
      await invoke("start_terminal", {
        folder: await invoke("get_node_folder_path", {
          path: data.current_item,
        }),
      });
      //} else if (cmd == "download_awesome_chatpgt_prompts") {
      //await download_awesome_chatpgt_prompts();
    } else {
      tosta.push("unhandled command", cmd);
    }
  }

  let filter_goto_for_move = (target_path) => {
    return (
      target_path.indexOf(":") == -1 ||
      target_path.startsWith("date:") ||
      target_path.startsWith("today:")
    );
  };

  let handle_shift_up = async () => {
    let siblings = find_siblings(data.flat, activeIndex);
    let path = data.current_item;
    if (siblings[0] !== null) {
      let res = await invoke("swap_node_with_previous", { path: path });
      if (res !== null) {
        toast.push(res);
        return false;
      }
      let parent = path.substring(0, path.length - 1);
      await toggle_node(parent, true);
      await toggle_node(parent, false);
      await goto_node(siblings[0]);
      return false; //no action by the Focusable
    }
  };

  let handle_shift_down = async () => {
    let siblings = find_siblings(data.flat, activeIndex);
    let path = data.current_item;
    if (siblings[1] !== null) {
      let res = await invoke("swap_node_with_next", { path: path });
      if (res !== null) {
        toast.push(res);
        return false;
      }
      let parent = path.substring(0, path.length - 1);
      await toggle_node(parent, true);
      await toggle_node(parent, false);
      toast.push("going to " + siblings[1]);
      await goto_node(siblings[1]);
      return false; //no action by the Focusable
    } else {
    }
  };

  function handle_nav_blur(ev) {
    //document.getElementById("nav_path_input").focus();
    ev.target.focus();
  }

  async function extractTextFromUrl(url) {
    const response = await tauri_fetch(url, {
      method: "GET",
      timeout: 30,
      responseType: 2,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    let html = response.data;
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    const reader = new Readability(doc);
    const article = reader.parse();
    return article.textContent;
  }

  async function handle_fetch_url_change(ev) {
    if (ev.key == "Enter") {
      if (fetch_url_text.startsWith("http")) {
        let target_node = data.current_item;
        viewComponent.leave_overlay();
        try {
          let text =
            "Fetched from " +
            fetch_url_text +
            "\n\n" +
            (await extractTextFromUrl(fetch_url_text));
          let new_path = await invoke("find_next_empty_child", {
            path: target_node,
          });
          await invoke("edit_node", {
            path: new_path,
            windowTitle: appWindow.label,
            newText: text,
          });
        } catch (e) {
          toast.push("Failed to fetch url. No node added");
          console.log(e);
        }
      } else {
        toast.push("invalid url");
      }
    }
  }

  function filter_tags(tags) {
    return tags.filter((item) => Object.values(data.tags).includes(item));
  }

  async function handle_copy(ev) {
    let mode = ev.detail;
    console.log("copy_to_clipboard", mode);
    let out = null;
    if (mode == "link") {
      //out = `[${data.title}](${data.path})`;
      out = `<<${data.current_item}>>`;
    } else if (mode == "content") {
      let node = await invoke("get_node", { path: data.current_item + "" });
      out = node?.node.raw;
    } else if (mode == "rendered_content") {
      out = await get_rendered_node(data.current_item);
    } else if (mode == "title") {
      out = data.flat[activeIndex].title;
    } else if (mode == "path") {
      out = await invoke("get_node_folder_path", { path: data.current_item });
    } else {
      console.log("unknown copy_to_clipboard mode", mode);
    }
    if (out != null) {
      await copy_to_clipboard(out);
    }
    viewComponent.leave_overlay();
  }

  async function handle_tag(ev) {
    viewComponent.leave_overlay();
    let tag = ev.detail;
    if (tag == "!!date") {
      date_pick_value = iso_date(Date.now());
      viewComponent.enter_overlay("date_pick_for_tag");
    } else {
      let path = data.current_item;
      if (data.currently_edited[path] != undefined) {
        toast.push("Cant toggle tags on currently edited nodes");
        return;
      }
      let node = await invoke("get_node", { path: path + "" });
      let text = node?.node?.raw;
      let add_timestamp = tag.indexOf("<timestamp>") != -1;
      if (add_timestamp) {
        let raw_tag = tag.replace("<timestamp>", "");
        let insert =
          tag.replace("<timestamp>", "<" + iso_date_and_time(Date.now())) + ">";
        let query = RegExp(`${raw_tag}<\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}>`);
        if (text.match(query)) {
          text = text.replace(query, "");
        } else {
          text = text + "\n\n" + insert;
        }
      } else {
        let add_date = tag.indexOf("<date>") != -1;
        if (add_date) {
          let raw_tag = tag.replace("<date>", "");
          let insert = tag.replace("<date>", "<" + iso_date(Date.now())) + ">";
          let query = RegExp(`${raw_tag}<\\d{4}-\\d{2}-\\d{2}>`);
          if (text.match(query)) {
            text = text.replace(query, "");
          } else {
            text = text + "\n\n" + insert;
          }
        } else {
          if (text.indexOf(tag) != -1) {
            text = text.replace(tag, "").trim();
          } else {
            text = text + "\n\n" + tag;
          }
        }
      }
      await invoke("change_node_text", { path, text });
    }
    //await goto_node(path);
  }

  function handle_window_focus() {
    focus_time = Date.now();
  }

  function search_mode_leave(ev) {
    viewComponent.leave_overlay();
  }

  async function update_date_pick_choice() {
    let date_suffix = await invoke("date_to_path", {
      dateStr: date_pick_value,
    });
    let path = date_pick_prefix + date_suffix;
    let found = await goto_node(path);
    let el = document.getElementById("date_pick_target");
    let text = "";
    if (!found) {
      el.classList.add("notfound");
      text = "(not found)";
    } else {
      el.classList.remove("notfound");
      text = data.flat[activeIndex].title;
    }
    await tick();
    document
      .querySelector(".chosen")
      .scrollIntoView({ behaviour: "smooth", block: "center" });
    el.innerText = path + " " + text;
    return "";
  }
  async function handle_date_pick_change(ev) {
    await update_date_pick_choice();
  }

  async function handle_date_space_action() {
    if (date_pick_mode == "goto") {
      viewComponent.leave_overlay();
    }
  }
  async function handle_date_leave() {
    leave_nav();
  }

  async function handle_date_changed(ev) {
    await update_date_pick_choice();
  }

  async function handle_date_action() {
    viewComponent.leave_overlay();
    if (date_pick_mode == "move") {
      let date_suffix = await invoke("date_to_path", {
        dateStr: date_pick_value,
      });
      let path = date_pick_prefix + date_suffix;

      await move_by_nav(path);
      if (!move_and_goto) {
        goto_node(nav_start_path);
      }
    } else {
      edit_current_node();
    }
  }

  async function handle_date_tag() {
    viewComponent.leave_overlay();
    let path = data.current_item;
    if (data.currently_edited[path] != undefined) {
      toast.push("Cant add to currently edited item");
      return;
    }
    let node = await invoke("get_node", { path: path + "" });
    let text = node?.node?.raw || "";
    text = text + `\n<${date_pick_value}>`;
    await invoke("change_node_text", { path, text });
    focus_first_in_node(document.getElementById("tree_parent"));
  }

  async function handle_sub_section_action(ev) {
    viewComponent.leave_overlay();
    let mode = ev.detail;
    switch (mode) {
      case "extract_subnode":
        {
          const Asciidoctor = asciidoctor();
          let node = await invoke("get_node", { path: data.current_item + "" });
          const document = Asciidoctor.load(node.node.raw, { sourcemap: true });
          const subsection = document.findBy({
            context: "section",
          });
          let hit = null;
          let hit_level = null;
          let next = null;
          let query = clicked_section.innerText.replace("☰", "");
          for (let ii = 0; ii < subsection.length; ii++) {
            if (hit == null) {
              if (subsection[ii].getTitle() == query) {
                hit = subsection[ii].getSourceLocation().getLineNumber() - 1;
                hit_level = subsection[ii].getLevel();
              }
            } else {
              if (subsection[ii].getLevel() <= hit_level) {
                next = subsection[ii].getSourceLocation().getLineNumber() - 1;
                break;
              }
            }
          }
          if (next == null) {
            next = count_lines(node.node.raw);
          }
          if (hit == null) {
            toast.push("failed to find subsection");
            return;
          }
          let lines = node.node.raw.split("\n");
          let len = next - hit;
          let extracted = lines.slice(hit, next).join("\n");
          extracted = extracted.replace(/=+/g, (match) => {
            return "=".repeat(match.length - 1);
          });
          lines.splice(hit, len);
          let removed = lines.join("\n");
          //data.currently_edited[data.current_item] = lines.join("\n");
          let new_path = await invoke("find_next_empty_child", {
            path: data.current_item,
          });
          await invoke("change_node_text", {
            path: new_path,
            text: extracted.slice(1), //cut off first =
          });
          await invoke("change_node_text", {
            path: data.current_item,
            text: removed,
          });
          //goto_node(data.current_item);
        }
        break;
      default: {
        tosta.push("unknown sub section action " + mode);
      }
    }
  }
</script>

<svelte:window on:focus={handle_window_focus} />
<View
  single_column="false"
  on:keyup={dispatch_keyup(keys, () => {
    if (Date.now() - focus_time < 500) {
      // toast.push("ignored keypress");
      return true;
    }
    return overlay != "";
  })}
  bind:this={viewComponent}
  bind:overlay
>
  <div slot="header" />
  <svelte:fragment slot="content">
    <div class="Left main_div" id="tree_parent">
      <Focusable
        on:itemChanged={itemChanged}
        on:itemExpand={item_toggle_children}
        bind:activeIndex
        bind:scroll_to_active
        on:itemSelected={item_selected}
        bind:can_expand
        bind:can_contract
        bind:handle_shift_up
        bind:handle_shift_down
      >
        {#each data.flat as node, ii}
          <tr
            data-path={node.path}
            class="{data.currently_edited[node.path] !== undefined
              ? 'edited'
              : ''}
			  {ii == activeIndex ? 'chosen' : ''}
			  "
          >
            <td>
              <div class="node">
                <div class="node-path">
                  {@html node.indention}{node.path}{#if node.has_children && !node.children_shown}<span
                      class="more">+</span
                    >{/if}
                </div>
                <div class="node-title">{node.title}</div>
              </div>

              <!-- <div style="display:inline-block;" class="mono">
              {@html node.indention}{node.path}{#if node.has_children && !node.children_shown}<span
                  class="more">+</span
                >
				{:else}
				<span>&nbsp;</span>
              {/if}&nbsp;
			  </div>
            <div
                style="display:inline-block;"
                class={node.path === highlight_node ? "highlight_in_tree" : ""}
              >
                {node.title} -->
              <!-- {#each filter_tags(node.tags) as tag}
                <div class="tags {tag_class(tag.substring(1))}">
                  {tag}
                </div>
              {/each} -->
            </td>
          </tr>
        {/each}
      </Focusable>
    </div>
    <div
      class="main_div Middle {data.currently_edited[data.current_item] !==
      undefined
        ? 'edited'
        : ''}"
      id="node_content"
      on:keyup={handle_key_up_content}
    >
      {#await get_rendered_node(data.current_item)}
        loading...
      {:then rendered}
        {@html rendered}
      {/await}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="overlays">
    {#if overlay == "help"}
      <Help bind:entries={help_entries} />
    {:else if overlay == "nav"}
      {nav_text} <br />
      <input
        type="text"
        bind:value={nav_path}
        autofocus
        on:keyup={handle_nav_change}
        on:blur={handle_nav_blur}
        id="nav_path_input"
      />
    {:else if overlay == "goto"}
      <Goto bind:action={handle_goto} bind:overlay />
    {:else if overlay == "move"}
      <Goto
        bind:action={handle_move}
        bind:overlay
        bind:filter={filter_goto_for_move}
      />
    {:else if overlay == "delete"}
      <QuickPick bind:entries={delete_entries} on:action={handle_delete} />
    {:else if overlay == "palette"}
      <QuickPick bind:entries={palette_entries} on:action={handle_palette} />
    {:else if overlay == "copying"}
      <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
    {:else if overlay == "subsection-menu"}
      <QuickPick
        bind:entries={sub_section_entries}
        on:action={handle_sub_section_action}
      />
    {:else if overlay == "tag"}
      tag
      <QuickPick bind:entries={tag_entries} on:action={handle_tag} />
    {:else if overlay == "search"}
      <Search
        bind:overlay
        bind:in_page_search_term
        bind:search_mode
        on:leave={search_mode_leave}
      />
    {:else if overlay == "date_pick"}
      <div>
        {date_pick_mode}
        <DateInput
          bind:value={date_pick_value}
          on:space_action={handle_date_space_action}
          on:action={handle_date_action}
          on:leave={handle_date_leave}
          on:change={handle_date_changed}
        />
        <span id="date_pick_target">...</span>
      </div>
      {#await update_date_pick_choice()}{/await}
    {:else if overlay == "date_pick_for_tag"}
      Add date to node:
      <DateInput
        bind:value={date_pick_value}
        on:space_action={handle_date_tag}
        on:action={handle_date_tag}
        on:leave={handle_date_leave}
      />
    {:else if overlay == "fetch_url"}
      Fetch url<br />
      <input
        type="text"
        bind:value={fetch_url_text}
        autofocus
        on:keyup={handle_fetch_url_change}
        on:blur={handle_nav_blur}
        id="fetch_url_input"
        style="width:98%;"
      />
    {:else}
      Unknown overlay: &quot;{overlay}&quot;
    {/if}
  </svelte:fragment>
</View>

<style>
  .mono {
    font-family: monospace;
    color: #666;
  }

  .more {
    color: #6a6aff;
  }

  td {
  }

  .column {
    float: none;
    overflow-y: scroll;
    overflow-x: hidden;
    padding-left: 0.5em;
    border-left: 3px dashed grey;
  }

  .smallcolumn {
    float: left;
    max-width: 49%;
    overflow-y: scroll;
    overflow-x: hidden;

    padding-right: 0.5em;
  }

  :global(.edited) {
    background-color: #ffdfdf;
  }
  :global(.highlight_in_tree) {
    background-color: #ffff7f;
  }

  :global(.notfound) {
    color: red;
  }

  .node {
    clear: both;
    overflow: hidden;
    display: flex;
  }

  .node-path {
    font-family: monospace;
    margin-right: 5px;
    word-wrap: nowrap;
    color: #aaa;
  }

  .node-title {
    white-space: pre-wrap;
    word-wrap: break-word;
    max-width: 80%;
  }
</style>
