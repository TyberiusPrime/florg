<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import TopTree from "./lib/TopTree.svelte";
  import NavTable from "./lib/NavTable.svelte";
  import TinyNav from "./lib/TinyNav.svelte";
  import Content from "./lib/Content.svelte";
  import DateMode from "./lib/DateMode.svelte";
  import PickMode from "./lib/PickMode.svelte";
  import MailListHeader from "./lib/MailListHeader.svelte";
  import MailListContent from "./lib/MailListContent.svelte";
  import MailMessage from "./lib/MailMessage.svelte";
  import GotoMode from "./lib/GotoMode.svelte";
  import SearchMode from "./lib/SearchMode.svelte";
  import Footer from "./lib/Footer.svelte";
  import * as KeyPress from "./js/keypress-2.1.5.min.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { emit, listen } from "@tauri-apps/api/event";
  import { exit } from "@tauri-apps/api/process";
  import { onMount, onDestroy } from "svelte";
  import { readText, writeText } from "@tauri-apps/api/clipboard";
  import asciidoctor from "asciidoctor";
  import PostalMime from "postal-mime";

  let Asciidoctor = asciidoctor();

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
  }

  let show_help = false;
  let mode = "normal";
  let footer_msg = "";
  let content_text = "";
  let content_rendered = "";
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

  let goto_mode_action = "";
  let goto_mode_entries = [];
  let goto_mode_text = "";
  let goto_show_normal = false;

  let search_mode_start_pos = null;
  let search_mode_term = "";
  let search_mode_action = "";

  let mail_mode_queries = [];
  let mail_mode_query = null;
  let mail_mode_focused = 0;
  let mail_mode_elements = []; // unfiltered
  let mail_mode_downstream_elements = []; //ad hoc filtered
  let mail_mode_more_mail_available = false;
  let mail_mode_view = "threads";

  let single_mail_mode_message = null;
  let single_mail_mode_message_tags = null;
  let single_mail_mode_message_id = null;

  var listener_normal = new window.keypress.Listener();
  //listener_normal.reset();
  //listener_normal.stop_listening();

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

  listener_normal.simple_combo("/", async (e, count, repeated) => {
    enter_search_mode("in_page");
  });

  listener_normal.register_combo({
    keys: "s",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      enter_search_mode("global");
    },
  });

  listener_normal.register_combo({
    keys: "shift s",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      enter_search_mode("mail");
    },
  });

  listener_normal.register_combo({
    keys: "n",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      window.find(search_mode_term, false, false, true, false);
    },
  });

  listener_normal.register_combo({
    prevent_repeat: true,
    keys: "shift n",
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      window.find(search_mode_term, false, true, true, false);
    },
  });

  listener_normal.simple_combo("x", async (e, count, repeated) => {
    enter_mail_view(
      "message:16780397160.6f1Bf6.105158@composer.zfsonlinux.topicbox.com"
    );
  });

  listener_normal.register_combo({
    keys: "i",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      goto_mail_search();
      window.find(search_mode_term, false, false, true, false);
    },
  });

  listener_normal.simple_combo("g", async (e, count, repeated) => {
    goto_nav("goto", "Goto");
  });

  listener_normal.simple_combo("z", async (e, count, repeated) => {
    goto_nav("add", "Add node below");
  });

  listener_normal.simple_combo("m", async (e, count, repeated) => {
    goto_nav("move", "Move to next empty node below");
  });

  listener_normal.simple_combo("t", async (e, count, repeated) => {
    let entries = [];
    let tags = await invoke("get_tags", {});
    for (let key in tags) {
      let hashtag = tags[key];
      let present = content_text.includes(hashtag);
      let text;
      if (present) {
        text = "-" + hashtag;
      } else {
        text = "+" + hashtag;
      }
      entries.push({
        key: key,
        target_path: hashtag,
        text: text,
      });
    }
    enter_goto_mode("tag", "Toggle tag", entries, true);
  });

  listener_normal.simple_combo("c", async (e, count, repeated) => {
    let entries = [
      { key: "c", text: "link", target_path: "link" },
      { key: "y", text: "content", target_path: "content" },
      { key: "t", text: "title", target_path: "title" },
      { key: "p", text: "node folder path", target_path: "path" },
      { key: "r", text: "rendered_content", target_path: "rendered_content" },
    ];
    enter_goto_mode("copy", "Copy to clipboard...", entries, true);
  });

  async function goto_nav(action, text) {
    let entries = [];
    let nav = await invoke("get_nav", {});
    for (let key in nav) {
      let target_path = nav[key];
      let query_path = target_path;
      if (query_path.startsWith("#") || query_path.startsWith("!")) {
        query_path = query_path.slice(1);
      }
      let node = await get_node(query_path);
      let text = target_path + " ";
      if (node.node != null) {
        text += node.node.header.title;
      } else {
        text += " (empty node)";
      }
      entries.push({
        key: key,
        target_path: target_path,
        text: text,
      });
    }
    enter_goto_mode(action, text, entries, false);
  }

  listener_normal.simple_combo("p", async (e, count, repeated) => {
    enter_pick_mode("command", "Command palette", [
      { cmd: "settings", text: "Edit settings" },
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

  async function goto_mail_search() {
    let entries = [];
    let searches = await invoke("get_mail_search_folders", {});
    if (searches == null) {
      searches = {
        a: "*",
        m: "tag:inbox and tag:unread",
        M: "tag:inbox",
      };
    }
    for (let key in searches) {
      let query = searches[key];
      entries.push({
        key: key,
        target_path: query,
        text: query,
      });
    }
    enter_goto_mode("mail", "Mail search", entries, false);
  }

  listener_normal.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      enter_normal_mode();
      await edit_current_node();
    },
  });

  listener_normal.listen();

  function enter_normal_mode() {
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
      listener_normal.stop_listening();
      footer_msg =
        "Nav mode activated. <span class='hotkey'>Escape</span> to abort. <span class='hotkey'>Space</span> to accept. <span class='hotkey'>Enter</span> to edit. <span class='hotkey'>Backspace</span> to go up. <span class='hotkey'>Home</span> to go to root";
      mode = "nav";
      nav_mode_start = current_path;
    }
  }

  function enter_date_mode(action, message) {
    mode = "date";
    date_mode_action = action;
    date_mode_message = message;
    nav_mode_start = current_path;
    listener_normal.stop_listening();
  }

  function enter_goto_mode(what, text, entries, show_normal) {
    listener_normal.stop_listening();
    if (show_normal) {
      mode = "quick_pick";
    } else {
      mode = "goto";
    }
    goto_mode_action = what;
    goto_mode_text = text;
    goto_mode_entries = entries;
    goto_show_normal = show_normal;
  }

  function enter_pick_mode(action, message, elements) {
    mode = "pick";
    pick_mode_action = action;
    pick_mode_message = message;
    pick_mode_elements = elements;
    listener_normal.stop_listening();
    footer_msg =
      "<span class='hotkey'>Enter</span> to accept, <span class='hotkey'>Esc</span> to cancel";
  }

  function enter_search_mode(action) {
    listener_normal.stop_listening();
    search_mode_start_pos = document.body.scrollTop;
    if (action == "in_page") {
      footer_msg =
        "In page search mode. <span class='hotkey'>Enter</span> to accept, <span class='hotkey'>Esc</span>";
    } else if (action == "global") {
      footer_msg =
        "Global search. <span class='hotkey'>Enter</span> to accept, <span class='hotkey'>Esc</span>";
    } else if (action == "mail") {
      footer_msg =
        "Mail query. <span class='hotkey'>Enter</span> to accept, <span class='hotkey'>Esc</span>";
    }

    search_mode_action = action;
    mode = "search";
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
  const unlisten_node_changed = listen("node-changed", async (event) => {
    // a specific node was reread
    console.log("node changed", event.payload);
    await load_node(event.payload);
    enter_normal_mode();
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
      }
    }
  );
  const unlisten_message = listen("message", (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing
    footer_msg = event.payload;
  });
  onDestroy(async () => {
    console.log("main app destroy");
    (await unlisten_node_changed)();
    (await unliste_node_unchanged)();
    (await unliste_node_temp_changed)();
    (await unlisten_message)();

    listener_normal.reset();
    listener_normal.stop_listening();
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
      await handle_command(ev.detail.cmd);
    } else if (ev.detail.action === "search") {
      load_node(ev.detail.cmd);
    }
  }

  async function handle_command(cmd) {
    if (cmd == "reload") {
      await invoke("reload_data", {});
      console.log("reloaded");
      await load_node(current_path);
    } else if (cmd == "exit") {
      await exit(1);
    } else if (cmd == "create_date_nodes") {
      await create_date_nodes();
    } else if (cmd == "settings") {
      await edit_settings();
    } else {
      console.log("unhandled command", cmd);
      footer_msg = `<span class='error'>unhandled command ${cmd}</span>`;
    }
  }

  async function edit_settings() {
    return await invoke("edit_settings", {});
  }

  async function create_date_nodes() {
    if (content_children.length > 0) {
      footer_msg =
        "<span class='error'>Can not create date nodes on an node that already has children</span>";
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
          footer_msg = "Created calendar";
          await invoke("create_calendar", {
            parentPath: current_path,
            year: year_parsed,
          });
          await invoke("reload_data", {});
          await load_node(current_path);
        }
      }
    }
  }

  async function handle_goto_leave(ev) {
    console.log("leave", ev);
    enter_normal_mode();
    /* if (ev.detail) {
      await edit_current_node();
    }*/
  }

  function iso_date(date: Date): String {
    let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(date);
    let mo = new Intl.DateTimeFormat("en", { month: "2-digit" }).format(date);
    let da = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
    return `${ye}-${mo}-${da}`;
  }

  async function handle_goto_action(ev) {
    if (goto_mode_action == "tag") {
      await toggle_tag_on_current_node(ev.detail);
    } else if (goto_mode_action == "copy") {
      await copy_to_clipboard(ev.detail);
    } else if (goto_mode_action == "mail") {
      await enter_mail_view(ev.detail);
    } else {
      let path = ev.detail;
      if (path.startsWith("!")) {
        let prefix = path.slice(1);
        let date_suffix = await invoke("date_to_path", {
          dateStr: iso_date(new Date()),
        });
        path = prefix + date_suffix;
      } else if (ev.detail.startsWith("#")) {
      }

      if (goto_mode_action == "goto") {
        enter_normal_mode();
        load_node(path);
      } else if (goto_mode_action == "add") {
        enter_normal_mode();
        let new_path = await invoke("find_next_empty_child", { path: path });
        await load_node(new_path);
        await edit_current_node();
      } else {
        enter_normal_mode();
      }
    }
  }

  async function toggle_tag_on_current_node(tag) {
    let re = new RegExp(tag + "(\\s|$)");
    let new_text;
    if (re.test(content_text)) {
      new_text = content_text.replace(re, "").trim();
    } else {
      new_text = content_text + "\n" + tag;
    }
    await invoke("change_node_text", { path: current_path, text: new_text });
    enter_normal_mode();
    let obj = document.getElementById("the_content");
    obj.scrollTop = obj.scrollHeight;
  }

  async function copy_to_clipboard(mode) {
    console.log("copy_to_clipboard");
    let out = null;
    if (mode == "link") {
      out = `[${content_title}](${current_path})`;
    } else if (mode == "content") {
      out = content_text;
    } else if (mode == "rendered_content") {
      out = document.getElementById("the_content").innerHTML;
    } else if (mode == "title") {
      out = content_title;
    } else if (mode == "path") {
      out = await invoke("get_node_folder_path", { path: current_path });
    } else {
      console.log("unknown copy_to_clipboard mode", mode);
    }
    if (out != null) {
      await writeText(out);
    }
    enter_normal_mode();
  }

  function removeItemOnce(arr, value) {
  var index = arr.indexOf(value);
  if (index > -1) {
    arr.splice(index, 1);
  }
  return arr;
}

  async function enter_mail_view(query) {
    if (
      mail_mode_queries.length == 0 ||
      (mail_mode_queries.length > 0 && mail_mode_queries.slice(-1) != query)
    ) {
      mail_mode_queries.push(query);
    }
    mail_mode_query = query;
    mode = "mail";
    listener_normal.stop_listening();

    mail_mode_elements.length = 0;
    mail_mode_downstream_elements.length = 0;

    if (query.startsWith("message:")) {
      mode = "single_mail";
      let mail_id = query.slice(8);
      let tup = await invoke("get_mail_message", {
        id: mail_id,
      });
      let raw_message = tup[0];
      let tags = tup[1];
      if (tags.indexOf("unread") > -1) {
        console.log("unread");
        await invoke("mail_message_remove_tags", {
          id: mail_id,
          tags: ["unread"],
        });
		removeItemOnce(tags, "unread");
      }

      if (raw_message != null) {
        const parser = new PostalMime();
        const email = await parser.parse(raw_message);
        single_mail_mode_message = email;
        single_mail_mode_message_tags = tags;
        single_mail_mode_message_id = mail_id;
      } else {
        single_mail_mode_message = null;
        single_mail_mode_message_id = null;
      }

      footer_msg =
		"<span class='hotkey'>Esc</span> to go back. <span class='hotkey'>h</span> to enable html view. <span class='hotkey'>i</span> to enable images. <span class='hotkey'>H</span> to view all headers";
    } else {
      footer_msg =
        "<span class='hotkey'>Enter</span> to select, <span class='hotkey'>Esc</span> to cancel. <span class='hotkey'>Ctrl-r</span> to refine.";
      if (query.startsWith("thread:")) {
        let mr = await invoke("query_mail", { query: query });
        let threads = mr[0];
        mail_mode_view = "messages";
        for (let thread of threads) {
          for (let msg of thread.messages) {
            mail_mode_elements.push(msg);
            mail_mode_downstream_elements.push(msg);
          }
        }
        mail_mode_more_mail_available = false;
      } else {
        let mr = await invoke("query_mail", { query: query });
        let threads = mr[0];
        for (let thread of threads) {
          mail_mode_elements.push(thread);
          mail_mode_downstream_elements.push(thread);
        }

        mail_mode_view = "threads";
        mail_mode_more_mail_available = mr[1];
      }
      mail_mode_elements = mail_mode_elements;
      mail_mode_downstream_elements = mail_mode_downstream_elements;
    }
  }

  async function handle_search_mode_leave(ev) {
    let ok = ev.detail;
    enter_normal_mode();
    if (search_mode_action == "in_page") {
      if (ok) {
        window.find(search_mode_term, false, false, true, false);
      } else {
        document.body.scrollTo(search_mode_start_pos[1]);
        search_mode_term = "";
      }
    } else if (search_mode_action == "global") {
      if (ok) {
        let rg_results = await invoke("ripgrep_below_node", {
          path: "",
          searchTerm: search_mode_term,
        });
        let translated_results = [];
        for (let result of rg_results) {
          let pretty_path = result.path;
          if (pretty_path == "") {
            pretty_path = "(root)";
          }
          let text = `${pretty_path}: <strong>${result.title}</strong>`;
          for (let pt of result.parent_titles) {
            text += " / " + pt;
          }

          text += "<br />";
          let counter = 0;
          for (let line of result.lines) {
            text += line[0] + ": " + line[1] + "<br />";
            counter += 1;
            if (counter > 7) {
              text += "...";
              break;
            }
          }
          translated_results.push({
            cmd: result.path,
            text: text,
          });
        }
        enter_pick_mode(
          "search",
          `Search results for <i>${search_mode_term}</i>`,
          translated_results
        );
      }
      search_mode_term = "";
    } else if (search_mode_action == "mail") {
      if (ok) {
        await enter_mail_view(search_mode_term);
      } else {
        search_mode_term = "";
      }
    } else {
      console.log("unknown search action", search_mode_action);
    }
  }

  async function handle_mail_action(ev) {
    console.log("mail show", ev.detail);
    if (ev.detail.single_message) {
      await enter_mail_view("message:" + ev.detail.id);
    } else {
      await enter_mail_view("thread:" + ev.detail.id);
    }
  }

  async function handle_mail_leave(ev) {
    mail_mode_queries.pop();
    if (mail_mode_queries.length > 0) {
      await enter_mail_view(mail_mode_queries.pop());
    } else {
      enter_normal_mode();
    }
  }

  async function handle_mail_refine_search(ev) {
    await enter_mail_view(mail_mode_query);
  }

  load_node("AA");
  enter_normal_mode();
</script>

<svelte:window />

<div class="wrapper">
  <div class="header" id="header">
    {#if mode == "nav" | mode == "normal" || mode == "quick_pick"}
    <TopTree
      bind:title={content_title}
      bind:path={current_path}
      bind:levels={content_levels}
      bind:mode
      on:goto_node={handle_goto_node}
    />
    <hr />
	{/if}
    {#if mode == "nav"}
      <NavTable
        bind:nav_table={content_children}
        on:goto_node={handle_goto_node}
        on:leave={handle_nav_mode_leave}
        bind:current_path
        bind:nav_mode_start
      />
      <hr />
    {:else if mode == "normal" || mode == "quick_pick"}
      <TinyNav
        bind:nodes={content_children}
        on:goto_node={handle_goto_node}
        bind:current_path
      />
      <hr />
    {:else if mode == "mail"}
      <MailListHeader
        bind:query={mail_mode_query}
        bind:more_mail={mail_mode_more_mail_available}
        on:leave={handle_mail_leave}
        on:refine_search={handle_mail_refine_search}
        bind:elements={mail_mode_elements}
        bind:downstream_elements={mail_mode_downstream_elements}
        bind:focused={mail_mode_focused}
        on:action={handle_mail_action}
        bind:view_mode={mail_mode_view}
      />
    {/if}
  </div>
  <div class="main_content">
    <div class="sticky-spacer" />
    <div class="sticky-content">
      {#if mode == "normal" || mode == "nav" || mode == "quick_pick" || mode == "search"}
        <Content bind:rendered={content_rendered} />
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
      {:else if mode == "mail"}
        <MailListContent
          on:action={handle_mail_action}
          on:leave={handle_mail_leave}
          bind:downstream_elements={mail_mode_downstream_elements}
          bind:focused={mail_mode_focused}
          bind:view_mode={mail_mode_view}
        />
      {:else if mode == "single_mail"}
        <MailMessage
          bind:message={single_mail_mode_message}
          bind:message_tags={single_mail_mode_message_tags}
          bind:message_id={single_mail_mode_message_id}
          on:leave={handle_mail_leave}
        />
      {/if}
    </div>
  </div>
  <div class="footer">
    <hr />
    {#if mode == "search"}
      <SearchMode
        on:leave={handle_search_mode_leave}
        bind:search_term={search_mode_term}
      />
    {/if}
    {#if mode == "goto" || mode == "quick_pick"}
      <GotoMode
        bind:action={goto_mode_action}
        bind:text={goto_mode_text}
        bind:entries={goto_mode_entries}
        on:leave={handle_goto_leave}
        on:action={handle_goto_action}
      />
      <hr />
    {/if}
    <Footer
      bind:show_help
      bind:msg={footer_msg}
      bind:currently_edited
      bind:mode
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
