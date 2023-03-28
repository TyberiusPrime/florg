<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import Expander from "$lib/../components/Expander.svelte";
  import Focusable from "$lib/../components/Focusable.svelte";
  import View from "$lib/../components/View.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { goto, invalidateAll } from "$app/navigation";
  import {
    render_text_cached,
    render_text,
  } from "$lib/../routes/node/[[path]]/funcs";
  import {
    patch_tree,
    flattenObject,
    expand_path,
    delete_from_tree,
    find_siblings,
  } from "./funcs";
  import { onMount, onDestroy, beforeUpdate, afterUpdate, tick } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import { readText as readClipboard } from "@tauri-apps/api/clipboard";
  import { add_code_clipboards, dispatch_keyup, iso_date } from "$lib/util.ts";
  import { focus_first_in_node } from "$lib/util.ts";
  import { emit, listen } from "@tauri-apps/api/event";
  import { fetch as tauri_fetch } from "@tauri-apps/api/http";

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
  ];
  let delete_entries = [{ key: "d", text: "delete node & children" }];
  let keys = {
    " ": () => {
      nav_text = "nav";
      nav_mode = "nav";
      nav_path = data.flat[activeIndex].path;
      nav_start_path = nav_path;
      nav_start_index = activeIndex;
      nav_start_tree = structuredClone(data.tree);
      viewComponent.enter_overlay("nav");
    },
    h: () => {
      viewComponent.enter_overlay("help");
    },
    g: () => {
      viewComponent.enter_overlay("goto");
    },
    a: () => {
      add_node();
    },
    d: () => {
      viewComponent.enter_overlay("delete");
    },
    x: async () => {
      fetch_url_text = await readClipboard();
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
        highlight_node = false;
        move_and_goto = false;
        viewComponent.enter_overlay("move");
      }
    },
    M: (ev) => {
      highlight_node = false;
      move_and_goto = true;
      viewComponent.enter_overlay("move");
    },
  };

  //todo: refactor
  function toggleElementAndChildren(element, isDisabled) {
    // Add wrapper functions to event handlers the first time this function is called on this element

    // Set disabled/enabled state of the element
    element.disabled = isDisabled;

    // Unset/re-set the tabIndex attribute on the element
    if (isDisabled) {
      //element.orgiginalTabIndex = element.tabIndex;
      //element.removeAttribute("tabIndex");
      element.classList.add("disabled");
    } else {
      //element.tabIndex = element.originalTabIndex;
      element.classList.remove("disabled");
    }

    // Disable/enable all children of the element
    var childNodes = element.childNodes;
    for (var i = 0; i < childNodes.length; i++) {
      var child = childNodes[i];
      if (child.nodeType === Node.ELEMENT_NODE) {
        toggleElementAndChildren(child, isDisabled);
      }
    }
  }

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
      return render_text(data.currently_edited[path]);
    }
    if (cache[path] === undefined) {
      let node = await invoke("get_node", { path: path + "" });
      if (node !== undefined && node.node !== null && node.node.raw != "") {
        let rt = await render_text_cached(path, node.node.raw);
        cache[path] = rt;
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
    let new_node = await invoke("get_node", { path: event.payload });
    cache[event.payload] = undefined;
    //patch_tree_content(data.tree, event.payload, new_node.node.raw);
    //data.flat = flattenObject(data.tree);
    let prefix = event.payload.substr(0, event.payload.length - 1);
    toast.push("expanding " + prefix);
    let p = data.current_item;
    if (p == "") {
      data.tree = await invoke("get_tree", { path: "", maxDepth: 2 });
    } else {
      goto_node(event.payload);
      goto_node(p);
    }

    data.flat = flattenObject(data.tree);
    data = data;
  });

  const unliste_node_unchanged = listen("node-unchanged", async (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing?
    data.currently_edited[event.payload] = undefined;
    data = data;
  });
  const unliste_node_temp_changed = listen(
    "node-temp-changed",
    async (event) => {
      if (event.payload[0] == data.current_item) {
        data.currently_edited[data.current_item] = event.payload[1];
        data = data;
      }
    }
  );
  const unlisten_message = listen("message", (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing
  });

  onMount(async () => {
    await goto_node(data.flat[data.start_item].path);
  });
  beforeUpdate(async () => {
    if (data.current_item != "" && activeIndex == undefined) {
      console.log("setting active index", data.current_item);
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
  });

  async function handle_key_up_content(ev) {
    if (ev.key == "ArrowLeft") {
      focus_first_in_node(document.getElementById("tree_parent"));
      document.querySelector(".chosen").scrollIntoView();
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

  async function handle_nav_change(ev) {
    if (ev.key == "Escape") {
      activeIndex = nav_start_index;
      data.path = nav_start_path;
      data.tree = nav_start_tree;
      data.flat = flattenObject(data.tree);
      viewComponent.leave_overlay();
      data = data;
      focus_first_in_node(document.getElementById("tree_parent"));
      ev.stopPropagation();
      ev.preventDefault();
      return;
    } else if (ev.key == " ") {
      if (nav_mode == "nav") {
        toast.push("space");
        viewComponent.leave_overlay();
        data = data;
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
          let parent = data.current_item.substring(
            0,
            data.current_item.length - 1
          );
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
    }

    ev.stopPropagation();
    //key is a..z
  }

  async function parse_path(path) {
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

  let handle_goto = async (path) => {
    let ppath = await parse_path(path);
    if (ppath.startsWith("mail:")) {
      goto("/mail/query/" + ppath.slice(5));
      return;
    }
    try {
      await goto_node(ppath);
    } catch (e) {
      toast.push("Could not go to node" + e);
    }

    viewComponent.leave_overlay();
    await tick();
    window.setTimeout(() => {
      focus_first_in_node(document.getElementById("tree_parent"));
      scroll_to_active();
    }, 10);
  };

  let handle_move = async (path) => {
    viewComponent.leave_overlay();
    let ppath = await parse_path(path);
    if (ppath.startsWith("mail:")) {
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

  function handle_delete(ev) {
    if (data.current_item != "") {
      invoke("delete_node", { path: data.current_item })
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
      viewComponent.leave_overlay();
      //delete_from_tree(data.tree, data.current_item);
      //console.log(data.tree);
      //data.flat = flattenObject(data.tree);
    }
  }

  let filter_goto_for_move = (target_path) => {
    return !target_path.startsWith("mail:");
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
          let text = "Fetched from " + fetch_url_text + "\n\n" + await extractTextFromUrl(fetch_url_text);
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
</script>

<View
  on:keyup={dispatch_keyup(keys, () => {
    return overlay != "";
  })}
  bind:this={viewComponent}
  bind:overlay
>
  <div slot="header">
    {window.location}
  </div>
  <svelte:fragment slot="content">
    <div class="smallcolumn main_div" id="tree_parent">
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
			  {node.path === highlight_node ? 'highlight_in_tree' : ''}
			  "
          >
            <td class="mono">
              {@html node.indention}{node.path}{#if node.has_children && !node.children_shown}<span
                  class="more">+</span
                >
              {/if}
            </td>
            <td>{node.title} </td>
          </tr>
        {/each}
      </Focusable>
    </div>
    <div
      class="main_div column {data.currently_edited[data.current_item] !==
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
    {/if}
  </svelte:fragment>
</View>

<style>
  .mono {
    font-family: monospace;
    color: #666;
  }

  .more {
    color: #3636ff;
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
</style>
