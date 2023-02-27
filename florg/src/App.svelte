<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import TopTree from "./lib/TopTree.svelte";
  import NavTable from "./lib/NavTable.svelte";
  import Content from "./lib/Content.svelte";
  import DateMode from "./lib/DateMode.svelte";
  import PickMode from "./lib/PickMode.svelte";
  import Footer from "./lib/Footer.svelte";
  import * as KeyPress from "../dist/keypress-2.1.5.min.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { emit, listen } from "@tauri-apps/api/event";
  import { exit } from "@tauri-apps/api/process";

  async function get_node(path) {
    return await invoke("get_node", { path });
  }

  async function load_node(path) {
    console.log("load node", path);
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

  let pick_mode_message = "";
  let pick_mode_action = "";
  let pick_mode_elements = "";

  var listener_normal = new window.keypress.Listener();
  listener_normal.reset();
  listener_normal.stop_listening();

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
    enter_date_mode("goto", "Goto Date below #insert-hashtag");
  });

  listener_normal.simple_combo("p", async (e, count, repeated) => {
    enter_pick_mode("command", "Command palette", [
      { cmd: "create_date_nodes", text: "create date nodes" },
      { cmd: "exit", text: "Exit the app" },
      { cmd: "reload", text: "Reload data from disk" },
    ]);
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

  listener_date.register_combo({
    keys: "esc",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      enter_normal_mode();
    },
  });

  listener_normal.listen();

  function enter_normal_mode() {
    listener_date.stop_listening();
    listener_normal.listen();
    footer_msg = "";
    mode = "normal";
    nav_mode_start = "";
    pick_mode_action = null;
    pick_mode_elements = [];
    date_mode_action = null;
  }

  function enter_nav_mode() {
    if (mode != "nav") {
      console.log("entering nav mode");
      listener_normal.stop_listening();
      listener_date.stop_listening();
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
    listener_date.listen();
  }

  function enter_pick_mode(action, message, elements) {
    mode = "pick";
    pick_mode_action = action;
    pick_mode_message = message;
    pick_mode_elements = elements;

    listener_normal.stop_listening();
    listener_date.stop_listening();
  }

  function handle_goto_node(ev) {
    load_node(ev.detail.path);
    if (ev.detail.normal_mode) {
      enter_normal_mode();
    }
  }

  async function handle_nav_mode_leave(ev) {
    console.log("leave", ev);
    enter_normal_mode();
    if (ev.detail) {
      await edit_current_node();
    }
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
    enter_normal_mode();
    if (ev.detail.action === null) {
    } else if (ev.detail.action == "goto") {
      let sub_path = await invoke("date_to_path", { dateStr: ev.detail.date });
      load_node(current_path + sub_path);
    }
  }

  async function handle_picker_canceled(ev) {
    enter_normal_mode();
    pick_mode_action = null;
  }

  async function handle_picker_accepted(ev) {
    enter_normal_mode();
    if (ev.detail.action === "command") {
      if (ev.detail.cmd == "reload") {
        await invoke("reload_data", {});
        load_node(current_path);
      } else if (ev.detail.cmd == "exit") {
        await exit(1);
      } else {
        console.log("unhandlede command", ev.detail.cmd);
        footer_msg = `<span class='error'>unhandled command ${ev.detail.cmd}</span>`;
      }
    }
  }

  load_node("A");
  enter_normal_mode();
</script>

<svelte:window />

<div class="wrapper">
  <div class="header">
    <TopTree
      bind:title={content_title}
      bind:path={current_path}
      bind:levels={content_levels}
      bind:mode
	  on:goto_node={handle_goto_node}
    />
    {#if mode == "nav"}
      <NavTable
        bind:nav_table={content_children}
        on:goto_node={handle_goto_node}
        on:leave={handle_nav_mode_leave}
        bind:current_path
        bind:nav_mode_start
      />
    {/if}
  </div>
  <div class="content">
    <div class="sticky-spacer" />
    <div class="sticky-content">
      {#if mode == "normal" || mode == "nav"}
        <Content bind:text={content_text} />
      {:else if mode == "date"}
        <DateMode
          bind:message={date_mode_message}
          bind:action={date_mode_action}
          on:date_chosen={handle_date_chosen}
        />
      {:else if mode == "pick"}
        <PickMode
          bind:message={pick_mode_message}
          bind:action={pick_mode_action}
          bind:elements={pick_mode_elements}
          on:picker_canceled={handle_picker_canceled}
          on:picker_accepted={handle_picker_accepted}
        />
      {/if}
    </div>
  </div>
  <div class="footer">
    <Footer bind:show_help bind:msg={footer_msg} bind:currently_edited />
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
