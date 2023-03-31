<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { render_text } from "./funcs";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import { writeText as copy_to_clipboard } from "@tauri-apps/api/clipboard";
  import { add_code_clipboards, dispatch_keyup } from "$lib/util.ts";
  import { goto, invalidateAll } from "$app/navigation";
  import { toast } from "@zerodevx/svelte-toast";
  import { appWindow } from "@tauri-apps/api/window";
  import {
    set_last_path,
    check_and_reset_mode_ignore_enter,
  } from "$lib/mode_stack.ts";
  import { page } from "$app/stores";

  import View from "$lib/../components/View.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import Help from "$lib/../components/Help.svelte";
  import TopTree from "$lib/../components/TopTree.svelte";
  import Search from "$lib/../components/Search.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  //import MailQueries from "$lib/../components/MailQueries.svelte";
  //import DatePicker from "../lib/DatePicker.svelte";

  import hljs from "highlight.js";

  export let data;
  let overlay = "";
  let search_mode;
  let in_page_search_term = "";

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "Shift-Esc", text: "Go forward" },
    { key: "Enter", text: "Edit node" },
    { key: "Space", text: "Nav mode" },
    { key: "o", text: "Goto: ChatGPT" },
    { key: "i", text: "Goto Mail" },
    { key: "s", text: "search" },
    { key: "n", text: "search: (next hit)" },
    { key: "N", text: "search: (prev hit)" },
    { key: "g", text: "goto" },
    { key: "z", text: "new node below" },
    { key: "m", text: "move node to node below" },
    { key: "p", text: "command palette" },
    { key: "c", text: "copy menu" },
    { key: "a", text: "add node below" },
  ];

  let copy_entries = [
    { key: "c", text: "link", target_path: "link" },
    { key: "y", text: "content", target_path: "content" },
    { key: "t", text: "title", target_path: "title" },
    { key: "p", text: "node folder path", target_path: "path" },
    { key: "r", text: "rendered_content", target_path: "rendered_content" },
  ];

  let delete_entries = [{ key: "d", text: "delete node & children" }];

  function apply_mods() {
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
    });
    add_code_clipboards();
    return "";
  }
  function show_help() {
    overlay = "help";
  }

  let keys = {
    " ": () => {
      goto("/nav/" + data.path);
      return true;
    },
    Escape: () => {
      if (overlay != "") {
        overlay = "";
        return true;
      }
    },
    g: () => {
      overlay = "goto";
      return true;
    },
    s: () => {
      overlay = "search";
      return true;
    },
    h: () => {
      show_help();
      return true;
    },
    z: () => {
      overlay = "new_below";
      return true;
    },
    m: () => {
      //overlay = "move_node";
      toast.push("todo");
      return true;
    },
    a: async () => {
      let next_empty = await invoke("find_next_empty_child", {
        path: data.path,
      });
      goto("/node/" + next_empty + "?edit=true");

      return true;
    },
    c: () => {
      overlay = "copying";
      return true;
    },
    d: () => {
      overlay = "delete";
      return true;
    },
    n: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      } else {
        overlay = "search";
        search_mode = "in_page";
      }
    },
    N: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      } else {
        overlay = "search";
        search_mode = "in_page";
      }
    },
    "#": () => {
      toast.push("hello");
      //overlay = "datenav";
      date_nav_target = data.path;
    },
    Backspace: () => {
      if (data.path.length > 0) goto("/node/" + data.path.slice(0, -1));
    },
    i: () => {
      toast.push("todo");
      overlay = "mail_queries";
    },
    Enter: (ev) => {
      if (!ev.ctrlKey) {
        edit_current_node();
      }
    },
    t: () => {
      goto("/tree/" + data.path);
    },
  };

  async function edit_current_node() {
    data.currently_edited = true;
    data.org_text = data.text;
    data.org_rendered = data.rendered;
    data = data;
    return await invoke("edit_node", {
      path: data.path,
      windowTitle: appWindow.label,
    });
  }

  onMount(async () => {});

  afterUpdate(async () => {
    if ($page.url.searchParams.get("edit") == "true") {
      await edit_current_node();
      await goto("/node/" + data.path, { replaceState: true });
    }
  });

  //this is an event from rust
  const unlisten_node_changed = listen("node-changed", async (event) => {
    // a specific node was reread
    console.log(event);
    if (event.payload == data.path) {
      await invalidateAll();
      console.log("invalidated", "/node/" + event.payload);
    } else {
      goto("/node/" + event.payload);
    }
  });

  const unliste_node_unchanged = listen("node-unchanged", async (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing?
    console.log("node unchanged", event.payload);
    if (event.payload == data.path) {
      data.text = data.org_text;
      data.rendered = data.org_rendered;
      data.currently_edited = false;
      data = data;
    }
  });
  const unliste_node_temp_changed = listen(
    "node-temp-changed",
    async (event) => {
      //some node was edited, but not confirmed yet.
      //reload to refresh the currently edited thing?
      console.log("node temp-changed", event.payload[0]);
      if (event.payload[0] == data.path) {
        let content_text = event.payload[1];
        data.rendered = await render_text(content_text);
        data = data;
      }
    }
  );
  const unlisten_message = listen("message", (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing
  });
  onDestroy(async () => {
    console.log("main app destroy");
    (await unlisten_node_changed)();
    (await unliste_node_unchanged)();
    (await unliste_node_temp_changed)();
    (await unlisten_message)();
  });

  function handle_overlay_leave() {
    // toast.push("overlay:leave in node");
    overlay = "";
  }

  function quiet(_) {
    return "";
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

  let handle_new_node_below = async (path) => {
    let ppath = await parse_path(path);
    console.log(ppath);
    let new_path = await invoke("find_next_empty_child", { path: ppath });
    await invoke("edit_node", {
      path: path,
      windowTitle: appWindow.label,
    });
    overlay = "";
  };

  async function handle_copy(ev) {
    let mode = ev.detail;
    console.log("copy_to_clipboard", mode);
    let out = null;
    if (mode == "link") {
      //out = `[${data.title}](${data.path})`;
      out = `<<${data.path}>>`;
    } else if (mode == "content") {
      out = data.text;
    } else if (mode == "rendered_content") {
      out = document.getElementById("the_content").innerHTML;
    } else if (mode == "title") {
      out = data.title;
    } else if (mode == "path") {
      out = await invoke("get_node_folder_path", { path: current_path });
    } else {
      console.log("unknown copy_to_clipboard mode", mode);
    }
    if (out != null) {
      await copy_to_clipboard(out);
    }
    overlay = "";
  }

  async function handle_date_nav_chosen(ev) {
    let date_suffix = await invoke("date_to_path", {
      dateStr: ev.detail.date,
    });
    let path = date_nav_target + date_suffix;
    if (date_nav_target == current_path) {
      goto("/node/" + path);
    } else {
      goto("/node/" + path, { replaceState: true });
    }

    overlay = "";
  }
  beforeUpdate(() => {});

  afterUpdate(() => {
    apply_mods();
    set_last_path(data.path);
  });

  async function handle_delete(ev) {
    await invoke("delete_node", { path: data.path })
      .then(() => {
        toast.push("node deleted");
        goto("/node/" + data.path.slice(0, -1));
      })
      .catch((e) => {
        toast.push(`Error ${e}`);
      });
    overlay = "";
  }
</script>

<div>
  <View on:keyup={dispatch_keyup(keys)}>
    <div slot="header">
      <TopTree bind:data />
    </div>
    <div
      slot="content"
      id="content"
      class={data.currently_edited ? "edited" : ""}
    >
      {@html data.rendered}
      {#if data.children != null && data.path != null}
        {#if data.children.length > 0}
          <h2 class="children">Children</h2>
          <table class="table_children">
            {#each data.children as child}
              <tr
                ><td
                  ><a href="/node/{data.path}{child.key}"
                    >{data.path.toLowerCase()}{child.key}</a
                  ></td
                ><td>{child.text}</td></tr
              >
            {/each}
          </table>
        {:else}
          <h2 class="children">No children</h2>
        {/if}
      {/if}
    </div>
    <div slot="footer">
      <Overlay on:leave={handle_overlay_leave} bind:overlay>
        {#if overlay == "help"}
          <Help bind:entries={help_entries} />
        {:else if overlay == "search"}
          <Search
            bind:overlay
            bind:in_page_search_term
            bind:search_mode
            on:leave
          />
        {:else if overlay == "goto"}
          <Goto bind:overlay />
        {:else if overlay == "new_below"}
          Create new node below
          <Goto bind:action={handle_new_node_below} />
        {:else if overlay == "copying"}
          <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
        {:else if overlay == "datenav"}
          Datenav:
          <DatePicker on:date_chosen={handle_date_nav_chosen} />
        {:else if overlay == "delete"}
          <QuickPick bind:entries={delete_entries} on:action={handle_delete} />
        {:else if overlay == ""}
          <div on:click={show_help}>
            Press <span class="hotkey">h</span> for help.
          </div>
        {:else}
          Unknown overlay: {overlay}
        {/if}
      </Overlay>
      {#if data.currently_edited}
        <span style="color:red">Currently edited</span>
      {/if}
    </div>
  </View>
</div>

<style>
  .children {
    font-size: 1em;
    padding-bottom: 0;
    margin-bottom: 0;
  }
  .table_children {
    margin: 0;
    border-collapse: collapse;
  }
  .table_children td {
    border: 1px solid grey;
    padding: 0.5em;
  }

  .edited {
    background-color: #ffafa8;
  }
</style>
