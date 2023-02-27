<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import TopTree from "./lib/TopTree.svelte";
  import Content from "./lib/Content.svelte";
  import DateMode from "./lib/DateMode.svelte";
  import Footer from "./lib/Footer.svelte";
  import * as KeyPress from "../dist/keypress-2.1.5.min.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { emit, listen } from "@tauri-apps/api/event";

  async function get_node(path) {
    return await invoke("get_node", { path });
  }

  async function load_node(path) {
    console.log("load node");
    let node = await get_node(path);
    if (node.node != null) {
      console.log(node);
      content_text = node.node.raw;
      content_title = node.node.header.title; //only used for root node.
    } else {
      content_text = "(empty node - enter to create)";
      content_title = "(empty node)";
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
    console.log("open paths", open_paths);
    currently_edited = open_paths.indexOf(path) > -1;
  }

  let show_help = false;
  let mode = "normal";
  let footer_msg = "";
  let content_text = "";
  let content_levels = "";
  let content_title = "";
  let content_children;
  let current_path = "";
  let currently_edited = false;

  let nav_mode_start = "";

  let date_mode_message = "";
  let date_mode_action = "";

  var listener_normal = new window.keypress.Listener();
  listener_normal.reset();
  listener_normal.stop_listening();

  var listener_nav = new window.keypress.Listener();
  listener_nav.reset();
  listener_nav.stop_listening();

  var listener_date = new window.keypress.Listener();
  listener_date.reset();
  listener_date.stop_listening();

  listener_normal.register_combo({
    keys: "space",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      enter_nav_mode();
    },
  });
  listener_normal.register_combo({
    keys: "h",
    is_unordered: true,
    on_keydown: (e, count, repeated) => {
      if (!repeated) {
        console.log("help mode");
        show_help = !show_help;
      }
    },
  });
  listener_normal.simple_combo("x", async (e, count, repeated) => {
    console.log("debug pressed");
    enter_date_mode("goto", "Goto Date below #insert-hashtag");
  });

  listener_normal.simple_combo("backspace", async (e, count, repeated) => {
    if (current_path.length > 0) {
      load_node(current_path.slice(0, -1));
    }
  });

  listener_normal.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      enter_normal_mode();
      edit_current_node();
    },
  });

  //help mode
  listener_nav.simple_combo("esc", () => {
    console.log("going back to", nav_mode_start);
    load_node(nav_mode_start);
    enter_normal_mode();
    mode = "normal";
  });

  for (let letter of 
    // prettier-ignore
	  ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"]) {
	  listener_nav.register_combo({
	  keys: letter,
	  prevent_repeat: true,
	  on_keyup: (async (ev) => {
		console.log("key pressed" + letter);
		await load_node(current_path + letter.toUpperCase());
	  })
	  }
  );

  }

  listener_nav.register_combo({
    keys: "space",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      enter_normal_mode();
    },
  });

  listener_nav.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      enter_normal_mode();
      edit_current_node();
    },
  });

  listener_nav.register_combo({
    keys: "backspace",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      if (current_path.length > 0) {
        load_node(current_path.slice(0, -1));
      }
    },
  });
  listener_nav.register_combo({
    keys: "home",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      if (current_path.length > 0) {
        load_node("");
      }
    },
  });

  listener_date.register_combo({
    keys: "esc",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
		enter_normal_mode();
    },
  })

  listener_normal.listen();

  function enter_normal_mode() {
    listener_nav.stop_listening();
    listener_date.stop_listening();
    listener_normal.listen();
    footer_msg = "";
    mode = "normal";
    nav_mode_start = "";
  }

  function enter_nav_mode() {
    if (mode != "nav") {
      console.log("entering nav mode");
      listener_normal.stop_listening();
      listener_date.stop_listening();
      listener_nav.listen();
      footer_msg =
        "Nav mode activated. <span class='hotkey'>Escape</span> to abort. <span class='hotkey'>Space</span> to accept. <span class='hotkey'>Enter</span> to edit. <span class='hotkey'>Backspace</span> to go up. <span class='hotkey'>Home</span> to go to root";
      mode = "nav";
      console.log("setting nav mode start", nav_mode_start);
      nav_mode_start = current_path;
    }
  }

  function enter_date_mode(action, message) {
    mode = "date";
    date_mode_action = action;
    date_mode_message = message;
    nav_mode_start = current_path;
    listener_normal.stop_listening();
    listener_nav.stop_listening();
	listener_date.listen();
  }

  function handle_toptree_load(ev) {
    load_node(ev.detail.path);
    enter_normal_mode();
  }

  function handle_go_sub_node(ev) {
    load_node(current_path + ev.detail);
    enter_normal_mode();
  }
  async function edit_current_node() {
    currently_edited = true;
    return await invoke("edit_node", { path: current_path });
  }

  //this is an event from rust
  const unlisten_node_changed = listen("node-changed", (event) => {
    // a specific node was reread
    console.log(event.payload);
    load_node(event.payload);
    enter_normal_mode();
  });

  const unliste_node_unchanged = listen("node-unchanged", (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing
    load_node(current_path);
  });

  //this is an event from dispatch / jvaascript
  async function handle_date_chosen(ev) {
    console.log(ev);
    enter_normal_mode();
    if (ev.detail.action === null) {
    } else if (ev.detail.action == "goto") {
      let sub_path = await invoke("date_to_path", { dateStr: ev.detail.date });
      load_node(current_path + sub_path);
    }
  }

  load_node("AA");
</script>

<svelte:window />

<div class="wrapper">
  <div class="header">
    <TopTree
      bind:title={content_title}
      bind:path={current_path}
      bind:levels={content_levels}
      bind:mode
      bind:nav_table={content_children}
      on:load_node={handle_toptree_load}
    />
  </div>
  <div class="content">
    <div class="sticky-spacer" />
    <div class="sticky-content">
      {#if mode == "normal" || mode == "nav"}
        <Content bind:text={content_text} />
      {:else}
        <DateMode
          bind:message={date_mode_message}
          bind:action={date_mode_action}
          on:date_chosen={handle_date_chosen}
        />
      {/if}
    </div>
  </div>
  <div class="footer">
    <Footer
      bind:show_help
      bind:msg={footer_msg}
      bind:currently_edited
      on:go_sub_node={handle_go_sub_node}
    />
  </div>
</div>

<!-- <header> <TopTree /> </header>
  <div class="content">
  <Content />
   </div>
<footer> <Footer bind:show_help /></footer>
  -->
<style>
</style>
