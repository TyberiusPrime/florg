<script lang="ts">
  import Picker from "$lib/../components/Picker.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  import {
    format_date,
    no_text_inputs_focused,
    error_toast,
    removeItemOnce,
  } from "$lib/util.ts";
  import { goto } from "$app/navigation";
  import { tag_class } from "$lib/colors.ts";
  import { invoke } from "@tauri-apps/api/tauri";
  import { keypress } from "keypress.js";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";

  export let data;
  let query = data.query;
  let focused = 0;
  let overlay = "";

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "t", text: "toggle tag" },
    { key: "m", text: "mark read" },
    { key: "c", text: "copy" },
  ];
  let copy_entries = [{ key: "c", text: "link", target_path: "link" }];
  let tag_entries = Object.keys(data.tags).map((key) => {
    return { key: key, text: data.tags[key], target_path: data.tags[key] };
  });

  var listener = new keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.register_combo({
    keys: "h",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        overlay = "help";
      }
    },
  });
  listener.register_combo({
    keys: "t",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        overlay = "tags";
      }
    },
  });

  listener.register_combo({
    keys: "m",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        toggle_tag({ detail: "unread" });
      }
    },
  });

  listener.register_combo({
    keys: "f",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        toggle_tag({ detail: "flagged" });
      }
    },
  });
  listener.register_combo({
    keys: "g",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      overlay = "goto";
    },
  });

  function latest_date(entry) {
    //go through all the messages, extract date, keep the largest one
    let latest = 0;
    for (let i = 0; i < entry.messages.length; i++) {
      let date = Date.parse(entry.messages[i].date);
      if (date > latest) {
        latest = date;
      }
    }
    return latest;
  }

  function is_unread(msg) {
    return msg.tags.indexOf("unread") > -1;
  }
  function count_unread(entry) {
    let counter = 0;
    for (let i = 0; i < entry.messages.length; i++) {
      if (entry.messages[i].tags.indexOf("unread") > -1) {
        counter++;
      }
    }
    return counter;
  }

  async function handle_action(ev) {
    let target = ev.detail.cmd;
    if (target.startsWith("message:")) {
      let mail_id = target.slice(8);
      let msg = await invoke("get_mail_message", {
        id: mail_id,
      });
      if (msg != null) {
	  console.log(mail_id);
	  console.log(encodeURIComponent(mail_id));
        goto("/mail/message/" + encodeURIComponent(mail_id));
      } else {
        toast.push('<span class="error">Error: Could not load email</span>');
      }
    } else {
      console.log("target", target);
      goto("/mail/query/" + encodeURIComponent(target));
    }
  }

  function link(entry) {
    if (entry.messages.length == 1) {
      return "message:" + entry.messages[0].id;
    } else {
      return "thread:" + entry.id;
    }
  }

  function find_mail(id) {
    for (let m of data.messages) {
      if (m.id == id) {
        return m;
      }
    }
    return null;
  }

  async function toggle_tag(ev) {
    let tag = ev.detail;
    if (data.mode == "messages") {
      let el = document.querySelectorAll(".msg_entry").item(focused);
      let target = el.dataset.id;
      let msg = find_mail(target);
      if (msg.tags.indexOf(tag) > -1) {
        await invoke("mail_message_remove_tags", { id: msg.id, tags: [tag] });
        removeItemOnce(msg.tags, tag);
      } else {
        await invoke("mail_message_add_tags", { id: msg.id, tags: [tag] });
        msg.tags.push(tag);
      }
    } else if (data.mode == "threads") {
      console.log(focused);
      let el = document.querySelectorAll(".msg_entry").item(focused);
      let target = el.dataset.thread;
      let thread = find_mail(target);
      if (thread.tags.indexOf(tag) > -1) {
        for (let msg of thread.messages) {
          await invoke("mail_message_remove_tags", { id: msg.id, tags: [tag] });
        }
        removeItemOnce(thread.tags, tag);
      } else {
        for (let msg of thread.messages) {
          await invoke("mail_message_add_tags", { id: msg.id, tags: [tag] });
        }
        thread.tags.push(tag);
      }
      console.log(target);
      console.log(thread);
    } else {
      error_toast("what");
    }
    data = data;
    overlay = false;
  }

  onMount(async () => {
    listener.listen();
    console.log("start lest");
  });

  onDestroy(() => {
    console.log("stop lest");
    listener.stop_listening();
  });

  function handle_overlay_leave() {
    overlay = "";
  }

  let enable_filter = false;

  function handle_goto_action(ev) {
    goto_node(ev.detail);
  }
</script>

<Picker on:action={handle_action} bind:focused bind:enable_filter>
  <div slot="message">
    <h1>Mail result</h1>
    Query: {data.query}
    {#if data.more_mail}
      <br />(More mail available. refine your search)
    {/if}
  </div>
  <svelte:fragment slot="entries">
    {#each data.messages as el, index}
      {#if data.mode == "threads"}
        <tr data-cmd={link(el)} class="msg_entry" data-thread={el.id}>
          <td class="index">{index}</td>
          <td class="unread_count">
            {#if count_unread(el) > 0}
              <span class="new">{count_unread(el)}/{el.messages.length}</span>
            {:else}
              {count_unread(el)}/{el.messages.length}
            {/if}
          </td>
          <td class="date">{@html format_date(latest_date(el))}</td>
          <td>
            <div class="fromsubject">
              <div class="subject {count_unread(el) > 0 ? 'new' : ''}">
                {el.subject}
              </div>
              <div class="from">{el.authors}</div>
            </div>
            {#each el.tags as tag}
			  <div class="tags {tag_class(tag)}">
                {tag}
              </div>
            {/each}
          </td>
        </tr>
      {:else if data.mode == "messages"}
        <tr data-cmd={"message:" + el.id} class="msg_entry" data-id={el.id}>
          <td class="date">{@html format_date(Date.parse(el.date))}</td>
          <td>
            <div class="fromsubject">
              <div class="subject {is_unread(el) > 0 ? 'new' : ''}">
                {el.subject}
              </div>
              <div class="from">{el.from}</div>
            </div>
            {#each el.tags as tag}
			  <div class="tags {tag_class(tag)}">
                {tag}
              </div>
            {/each}
          </td></tr
        >
      {/if}
    {/each}
  </svelte:fragment>
  <div slot="footer">
    <Overlay {listener} on:leave={handle_overlay_leave} bind:overlay>
      {#if overlay == "help"}
        <Help bind:entries={help_entries} />
      {:else if overlay == "tags"}
        <QuickPick bind:entries={tag_entries} on:action={toggle_tag} />
      {:else if overlay == "copying"}
        <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
      {:else if overlay == "goto"}
        <Goto />
      {:else if overlay == ""}
        Press <span class="hotkey">h</span> for help.
      {:else}
        Unknown overlay: {overlay}
      {/if}
    </Overlay>
  </div>
</Picker>

<style>
  input {
    width: 50%;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }

  td,
  th {
    vertical-align: top;
    padding-right: 0.5em;
    border-bottom: 1px solid #ddd;
    padding-top: 5px;
  }

  .date {
    padding-top: 5px;
    text-align: left;
    font-size: 0.8em;
  }
  .index {
    width: 0.5em;
    font-size: 0.7em;
  }
  .unread_count {
    font-size: 0.7em;
  }

  td div {
    padding: 0px;
    margin: 0px;
  }

  .from {
    font-size: 14px;
    color: #555;
  }

  .subject {
    font-size: 1em;
    color: #111;
  }
  .fromsubject {
    float: left;
    padding-right: 1em;
  }

  .new {
    font-weight: bold;
  }

  .chosen {
    background-color: #bfbfff;
  }
  .chosen:nth-child(odd) {
    background-color: #bfbfff;
  }

  tr:nth-child(odd) {
    background-color: #feffee;
  }
</style>
