<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount, onDestroy } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import { enter_mode, leave_mode } from "../lib/mode_stack.ts";

  import asciidoctor from "asciidoctor";
  import hljs from "highlight.js";
  import "../styles/highlight.js/github.css";
  import { add_code_clipboards, get_node, iso_date } from "../lib/util.ts";

  import View from "../lib/View.svelte";
  import Overlay from "../lib/Overlay.svelte";
  import Help from "../lib/Help.svelte";
  import TopTree from "../lib/TopTree.svelte";
  import Search from "../lib/Search.svelte";
  import Goto from "../lib/Goto.svelte";

  export let mode;
  export let mode_args;

  let content_text = "";
  let content_rendered = "";
  let content_levels = "";
  let content_title = "";
  let content_children;
  let current_path = "";
  let currently_edited = false;
  let overlay;

  let Asciidoctor = asciidoctor();

  let in_page_search_term = "";

  let help_entries = [
    { key: "Esc", text: "Go back" },
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
  ];

  async function load_node(path, edit_afterwards = false) {
    //console.log("load node", path);
    let node = await get_node(path);
    if (node.node != null) {
      //console.log(node);
      content_text = node.node.raw;
      content_title = node.node.header.title; //only used for root node.
    } else {
      content_text = "(empty node - enter to create)";
      content_title = "(empty node)";
    }
    let rendered_cached = await invoke("get_cached_node", { path });
    if (rendered_cached == null) {
      let start_time = performance.now();
      content_rendered = Asciidoctor.convert(content_text, {
        attributes: {
          doctype: "article",
          showtitle: true,
          "source-highlighter": "highlight.js",
          "highlightjs-languages": "rust, swift",
        },
      });
      let end_time = performance.now();
      if (end_time - start_time > 100) {
        // probably just as fast to not cache...
        await invoke("set_cached_node", {
          path: path,
          raw: content_text,
          rendered: content_rendered,
        });
      }
    } else {
      content_rendered = rendered_cached;
    }

    let children = [];
    node.children.forEach((c) => {
      children.push({
        key: c.path.slice(-1),
        text: c.header.title,
        hover: c.header.first_paragraph,
      });
    });
    content_children = children;
    current_path = path;
    content_levels = node.levels;
    let open_paths = await invoke("list_open_paths");
    currently_edited = open_paths.indexOf(path) > -1;
    let obj = document.getElementById("the_content");
    if (obj != null) {
      obj.scrollTop = 0;
    }
    if (edit_afterwards) {
      await edit_current_node();
    }
  }

  function apply_mods(_ignored) {
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
    });
    add_code_clipboards();
    return "";
  }

  var listener = new window.keypress.Listener();

  listener.register_combo({
    keys: "h",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated) {
        overlay = "help";
      }
    },
  });

  listener.register_combo({
    keys: "s",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "search";
    },
  });

  listener.register_combo({
    keys: "space",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      await enter_mode("nav", { enter_path: current_path }, true);
    },
  });

  listener.register_combo({
    keys: "backspace",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      if (current_path.length > 0)
        await enter_mode("node", { path: current_path.slice(0, -1) }, false);
    },
  });
  listener.register_combo({
    keys: "esc",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      console.log("from esc in listener");
      leave_mode();
    },
  });

  listener.register_combo({
    keys: "n",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      }
    },
  });

  listener.register_combo({
    keys: "shift n",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      }
    },
  });

  listener.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      edit_current_node();
    },
  });

  listener.register_combo({
    keys: "g",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "goto";
    },
  });

  listener.register_combo({
    keys: "z",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "new_below";
    },
  });

  listener.register_combo({
    keys: "p",
    is_unordered: true,
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "goto";
    },
  });

  async function edit_current_node() {
    currently_edited = true;
    return await invoke("edit_node", { path: current_path });
  }

  onMount(async () => {
    listener.listen();
    overlay = "";
  });

  //this is an event from rust
  const unlisten_node_changed = listen("node-changed", async (event) => {
    // a specific node was reread
    console.log("node changed", event.payload);
    await load_node(event.payload);
  });

  const unliste_node_unchanged = listen("node-unchanged", async (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing?
    console.log("node unchanged", event.payload);
    if (event.payload == current_path) {
      await load_node(current_path);
    }
  });
  const unliste_node_temp_changed = listen(
    "node-temp-changed",
    async (event) => {
      //some node was edited, but not confirmed yet.
      //reload to refresh the currently edited thing?
      console.log("node temp-changed", event.payload[0]);
      if (event.payload[0] == current_path) {
        content_text = event.payload[1];
        content_rendered = Asciidoctor.convert(content_text, {
          attributes: {
            doctype: "article",
            showtitle: true,
            "source-highlighter": "highlight.js",
            "highlightjs-languages": "rust, swift",
          },
        });
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
    listener.stop_listening();
  });

  function handle_overlay_leave() {
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
      //TODO
    }
    return path;
  }

  async function handle_goto_action(ev) {
    let path = await parse_path(ev.detail);
    load_node(path);
    overlay = "";
  }

  async function handle_new_node_below(ev) {
    let path = await parse_path(ev.detail);
    console.log(path);
    let new_path = await invoke("find_next_empty_child", { path: path });
    load_node(new_path, true);
    overlay = "";
  }
</script>

<div>
  <View bind:mode bind:mode_args>
    <div slot="header">
      <TopTree
        bind:levels={content_levels}
        bind:title={content_title}
        bind:path={current_path}
        bind:mode
      />
    </div>
    <div slot="content">
      {@html content_rendered}
    </div>
    <div slot="footer">
      <Overlay {listener} on:overlay_leave={handle_overlay_leave} bind:overlay>
        {#if overlay == "help"}
          <Help bind:entries={help_entries} />
        {:else if overlay == "search"}
          <Search bind:mode bind:overlay bind:in_page_search_term />
        {:else if overlay == "goto"}
          Goto node:
          <Goto on:action={handle_goto_action} />
        {:else if overlay == "new_below"}
          Create new node below
          <Goto on:action={handle_new_node_below} />
        {:else if overlay == ""}
          Press <span class="hotkey">h</span> for help.
        {:else}
          Unknown overlay: {overlay}
        {/if}
      </Overlay>
      {#if currently_edited}
        <span style="color:red">Currently edited</span>
      {/if}
    </div>
  </View>
  {quiet(load_node(mode_args.path, mode_args.edit === true))}
  {quiet(apply_mods(content_rendered))}
</div>
