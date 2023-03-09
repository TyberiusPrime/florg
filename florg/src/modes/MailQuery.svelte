<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import {
    enter_mode,
    leave_mode,
    get_last_path,
    mode_args_store,
  } from "../lib/mode_stack.ts";
  import Picker from "../lib/Picker.svelte";
  import { format_date, removeItemOnce, error_toast } from "../lib/util.ts";
  import { toast } from "@zerodevx/svelte-toast";
  import { find_color } from "../lib/colors.ts";
  import PostalMime from "postal-mime";
  import { onMount, onDestroy } from "svelte";

  let mode_args;
  mode_args_store.subscribe((value) => {
    mode_args = value;
  });
  let focused;
  let view_mode = "loading";
  let mail = [];
  let more_mail = false;

  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  onMount(async () => {
    listener.reset();
    let te = await invoke("mail_get_tags", {});
    if (te != null) {
      for (let key in te) {
        console.log("registering meta " + key);
        listener.register_combo({
          keys: "meta " + key,
          prevent_default: true,
          prevent_repeat: true,
          on_keyup: async (e, count, repeated) => {
            console.log("key", key);
            await toggle_tag(te[key]);
          },
        });
      }
    }
    let x = await get_mail(mode_args.query);
    view_mode = x[0];
    mail = x[1];
    more_mail = x[2];

    listener.listen();
  });

  onDestroy(() => {
    listener.stop_listening();
  });

  function find_mail(id) {
    for (let m of mail) {
      if (id.startsWith(m.id)) return m; //todo: do better
    }
	return null;
  }
  async function toggle_tag(tag) {
    console.log(focused);
    let el = document.querySelectorAll(".msg_entry").item(focused);
    let target = el.dataset.cmd;
    if (target.startsWith("message:")) {
      let mail_id = target.slice(8);
      let mail = find_mail(mail_id);
	  if (mail === null) {
		error_toast("could not find message");
		return;
	  }
	  console.log(mail);
      if (await invoke("mail_message_toggle_tag", { id: mail_id, tag: tag })) {
        toast.push("Toggled");
      } else {
        error_toast("Could not toggle tag");
      }
    } else {
      error_toast("Can only toggle tags on messages");
    }
    mail = mail;
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

  async function get_mail(query) {
    let res = [];
    if (query.startsWith("thread:")) {
      res.push("messages");
      let t = await invoke("query_mail", { query: query });
      let threads = t[0];
      let messages = [];
      for (let thread of threads) {
        for (let msg of thread.messages) {
          messages.push(msg);
        }
      }
      messages.reverse();
      res.push(messages);
      res.push(t[1]);
    } else {
      res.push("threads");
      let t = await invoke("query_mail", { query: query });
      res.push(t[0]); //threads
      res.push(t[1]); //more mail
    }
    return res;
  }

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

  async function handle_action(ev) {
    let target = ev.detail.cmd;
    if (target.startsWith("message:")) {
      let mail_id = target.slice(8);
      let msg = await invoke("get_mail_message", {
        id: mail_id,
      });
      if (msg != null) {
        let raw_message = msg.raw;
        let tags = msg.tags;
        if (tags.indexOf("unread") > -1) {
          console.log("unread");
          await invoke("mail_message_remove_tags", {
            id: mail_id,
            tags: ["unread"],
          });
          removeItemOnce(tags, "unread");
        }
        const parser = new PostalMime();
        const email = await parser.parse(raw_message);
        enter_mode(
          "mail_message",
          {
            message: email,
            message_id: mail_id,
            message_tags: tags,
            message_filename: msg.filename,
          },
          false
        );
      } else {
        toast.push('<span class="error">Error: Could loat email</span>');
      }
    } else {
      enter_mode("mail_query", { query: target }, false);
    }
  }

  function link(entry) {
    if (entry.messages.length == 1) {
      return "message:" + entry.messages[0].id;
    } else {
      return "thread:" + entry.id;
    }
  }
</script>

<div>
  {#if view_mode == "Loading"}
    Loading...
  {:else}
    <Picker on:action={handle_action} bind:focused>
      <div slot="message">
        <h1>Mail result</h1>
        Query: {mode_args.query}
        {#if more_mail}
          <br />(More mail available. refine your search)
        {/if}
      </div>
      <svelte:fragment slot="entries">
        {#each mail as el, index}
          {#if view_mode == "threads"}
            <tr data-cmd={link(el)} class="msg_entry">
              <td class="index">{index}</td>
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
                  <div class="tags" style="background-color:{find_color(tag)}">
                    {tag}
                  </div>
                {/each}
              </td>
            </tr>
          {:else if view_mode == "messages"}
            <tr data-cmd={"message:" + el.id} class="msg_entry">
              <td class="date">{@html format_date(Date.parse(el.date))}</td>
              <td>
                <div class="fromsubject">
                  <div class="subject {is_unread(el) > 0 ? 'new' : ''}">
                    {el.subject}
                  </div>
                  <div class="from">{el.from}</div>
                </div>
                {#each el.tags as tag}
                  <div class="tags" style="background-color:{find_color(tag)}">
                    {tag}
                  </div>
                {/each}
              </td></tr
            >
          {/if}
        {/each}
      </svelte:fragment>
    </Picker>
  {/if}
</div>

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

  .tags {
    display: inline-block;
    margin-left: 5px;
    padding: 3px 8px;
    border-radius: 10px;
    font-size: 12px;
    font-weight: bold;
    color: #eee;
    background-color: #3b9cff;
    float: left;
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
