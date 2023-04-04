<script lang="ts">
  import Picker from "$lib/../components/Picker.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Search from "$lib/../components/Search.svelte";
  import View from "$lib/../components/View.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  import { toast } from "@zerodevx/svelte-toast";
  import {
    format_date,
    no_text_inputs_focused,
    error_toast,
    removeItemOnce,
    dispatch_keyup,
  } from "$lib/util.ts";
  import { goto, invalidateAll } from "$app/navigation";
  import { tag_class } from "$lib/colors.ts";
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";

  export let data;
  let viewComponent;
  let query = data.query;
  let focused = 0;
  let overlay;
  let search_mode;
  let in_page_search_term;

  let advance_picker;

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "t", text: "toggle tag" },
    { key: "m", text: "mark read" },
    { key: "c", text: "copy" },
    { key: "s", text: "search" },
    { key: "n/N", text: "in page search" },
    { key: "r", text: "refresh mails" },
  ];
  let copy_entries = [{ key: "c", text: "link", target_path: "link" }];
  let tag_entries = Object.keys(data.tags ?? []).map((key) => {
    return { key: key, text: data.tags[key], target_path: data.tags[key] };
  });

  let keys = {
    Escape: () => {
      if (overlay != "") {
        viewComponent.leave_overlay();
        let chosen = document.getElementsByClassName("chosen");
        console.log(chosen);
        if (chosen.length > 0) {
          console.log("chosen is", chosen);
          chosen[0].scrollIntoView();
        }
        return true;
      }
    },
    h: () => {
      if (no_text_inputs_focused()) {
        viewComponent.enter_overlay("help");
        return true;
      }
    },
    t: () => {
      if (no_text_inputs_focused()) {
        viewComponent.enter_overlay("tags");
        return true;
      }
    },
    s: () => {
      if (no_text_inputs_focused()) {
        viewComponent.enter_overlay("search");
        return true;
      }
    },
    n: () => {
      if (no_text_inputs_focused) {
        if (in_page_search_term != "") {
          window.find(in_page_search_term, false, false, true, false);
          return true;
        } else {
          viewComponent.enter_overlay("search");
          search_mode = "in_page";
          return true;
        }
      }
    },

    N: () => {
      if (no_text_inputs_focused) {
        if (in_page_search_term != "") {
          window.find(in_page_search_term, false, false, true, true);
          return true;
        } else {
          viewComponent.enter_overlay("search");
          search_mode = "in_page";
          return true;
        }
      }
    },
    m: () => {
      if (no_text_inputs_focused()) {
        toggle_tag({ detail: "unread" });
      }
    },
    r: async () => {
      if (no_text_inputs_focused()) {
        await invalidateAll();
      }
    },
    f: () => {
      if (no_text_inputs_focused()) {
        toggle_tag({ detail: "flagged" });
      }
    },
    g: () => {
      viewComponent.enter_overlay("goto");
    },
    " ": () => {
      toggle_tag({ detail: "unread" });
      advance_picker();
    },
  };

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
          removeItemOnce(msg.tags, tag);
        }
          removeItemOnce(thread.tags, tag);
      } else {
        for (let msg of thread.messages) {
          await invoke("mail_message_add_tags", { id: msg.id, tags: [tag] });
		  msg.tags.push(tag);
        }
        thread.tags.push(tag);
      }
      console.log(target);
      console.log(thread);
    } else {
      error_toast("what");
    }
    data = data;
    viewComponent.leave_overlay();
  }

  function handle_keys(ev) {
    dispatch_keyup(keys)(ev);
  }

  onMount(async () => {
    if (overlay == undefined) {
      overlay = "";
    }
  });

  onDestroy(() => {
    document.getElementById("wrapper");
  });

  function handle_goto_action(ev) {
    goto_node(ev.detail);
  }

  function handle_search_leave() {
    viewComponent.leave_overlay();
  }

  afterUpdate(() => {
    if (overlay == "") {
      window.setTimeout(() => {
        document.getElementById("pick_table").focus();
      }, 100);
    }
  });
</script>

<View bind:this={viewComponent} bind:overlay single_column="false">
  <div slot="header" class="header">
    <h1>Mail result</h1>
    Query: {data.query}
    {#if data.more_mail}
      <div style="float:right">(More mail available. refine your search)</div>
    {/if}
  </div>
  <svelte:fragment slot="content">
    <div on:keyup={handle_keys} class="Middle main_div">
      <Picker
        on:action={handle_action}
        bind:focused
        bind:advance={advance_picker}
      >
        <svelte:fragment slot="entries">
          {#each data.messages as el, index}
            {#if data.mode == "threads"}
              <tr data-cmd={link(el)} class="msg_entry" data-thread={el.id}>
                <td class="index"
                  >{index}
                  {#if el.tags.indexOf("flagged") > -1}
                    <b class="flagged">★</b>
                  {/if}
                </td>
                <td class="unread_count">
                  {#if count_unread(el) > 0}
                    <span class="new"
                      >{count_unread(el)}/{el.messages.length}</span
                    >
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
              <tr
                data-cmd={"message:" + el.id}
                class="msg_entry"
                data-id={el.id}
              >
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
      </Picker>
    </div>
  </svelte:fragment>

  <svelte:fragment slot="overlays">
    {#if overlay == "help"}
      <Help bind:entries={help_entries} />
    {:else if overlay == "tags"}
      <QuickPick bind:entries={tag_entries} on:action={toggle_tag} />
    {:else if overlay == "copying"}
      <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
    {:else if overlay == "goto"}
      <Goto bind:overlay />
    {:else if overlay == "search"}
      <Search
        bind:overlay
        bind:in_page_search_term
        bind:default_search_term={data.query}
        bind:search_mode
        on:leave={handle_search_leave}
      />
    {:else if overlay == ""}
      Press <span class="hotkey">h</span> for help.
    {:else}
      Unknown overlay: {overlay}
    {/if}
  </svelte:fragment>
</View>

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

  .flagged {
    color: red;
    font-size: 1.5em;
  }
</style>
