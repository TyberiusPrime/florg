<script lang="ts">
  import {keypress} from "keypress.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { toast } from "@zerodevx/svelte-toast";
  import { get_last_path } from "../lib/mode_stack.ts";
  import { escape_html, error_toast, removeItemOnce } from "../lib/util.ts";
  import { createEventDispatcher } from "svelte";
  import { writeText as copy_to_clipboard } from "@tauri-apps/api/clipboard";
  import PostalMime from "postal-mime";
  import { onMount, onDestroy } from "svelte";
  import SvelteTooltip from "svelte-tooltip";
  import html2plaintext from "html2plaintext";
  import Expander from "../lib/Expander.svelte";
  import View from "../lib/View.svelte";
  import QuickPick from "../lib/QuickPick.svelte";
  import Overlay from "../lib/Overlay.svelte";
  import Help from "../lib/Help.svelte";

  export let params;

  const dispatch = createEventDispatcher();
  let overlay = "";

  let show_html = false;
  let show_images = false;
  let all_headers = false;
  let tag_entries = [];
  let message = null;

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "m", text: "toggle html" },
    { key: "i", text: "toggle images" },
    { key: "c", text: "copy menu" },
  ];

  var listener = new keypress.Listener();
  listener.reset();
  listener.stop_listening();

  onMount(async () => {
    overlay = "";
    listener.listen();
    window.scroll(0, 0);

    let te = await invoke("mail_get_tags", {});
    if (te != null) {
      for (let key in te) {
        let el = te[key];
        tag_entries.push({
          key: key,
          target_path: el,
          text: el,
        });
      }
    }
  });

  onDestroy(() => {
    listener.stop_listening();
  });

  listener.register_combo({
    keys: "escape",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      window.history.back();
    },
  });

  listener.register_combo({
    keys: "m",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        show_html = !show_html;
      }
    },
  });
  listener.register_combo({
    keys: "h",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        overlay = "help";
      }
    },
  });

  listener.register_combo({
    keys: "p",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      if (message_tags.indexOf("attachment") != -1) {
        if (
          await invoke("mail_message_store_attachments", {
            id: message_id,
          })
        ) {
          toast.push("Attachments saved");
        } else {
          error_toast("Error saving attachments");
        }
      } else {
        toast("No attachments");
      }
    },
  });

  listener.register_combo({
    keys: "shift h",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        all_headers = !all_headers;
      }
    },
  });

  listener.register_combo({
    keys: "c",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        overlay = "copying";
      }
    },
  });

  listener.register_combo({
    keys: "t",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        overlay = "tags";
      }
    },
  });

  listener.register_combo({
    keys: "i",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.parsed.html != null) {
        show_images = !show_images;
        show_html = false;
        //we need it to retrigger building the iframe
        //we can't reload the iframe,
        //because of it's sandbox *rolleyes*
        window.setTimeout(() => {
          show_html = true;
        }, 10);
      }
    },
  });

  function get_header(message, header) {
    for (let i = 0; i < message.parsed.headers.length; i++) {
      if (
        message.parsed.headers[i].key != null &&
        message.parsed.headers[i].key.toLowerCase()! == header
      ) {
        return message.parsed.headers[i].value;
      }
    }
  }
  function local_date(date) {
    if (date != null) {
      let t = new Date(date);
      let z = t.getTimezoneOffset() * 60 * 1000;
      let tLocal = t - z;
      tLocal = new Date(tLocal);
      let iso = tLocal.toISOString();
      iso = iso.split(".")[0];
      iso = iso.replace("T", " ");
      return iso;
    } else {
      ("Unknown");
    }
  }
  const optional_headers = ["cc", "bcc", "reply-to", "x-envelope-to"];

  function csp(html) {
    let r = "";
    if (!show_images) {
      r +=
        '<meta http-equiv="Content-Security-Policy" content="child-src self; img-src self" script-src none>';
    } else {
      r +=
        '<meta http-equiv="Content-Security-Policy" content="child-src self; script-src none">';
    }
    return r + html;
  }

  function resize_iframe() {
    document.getElementById("mail_content_iframe").style.height = "200px"; //document.getElementById("mail_content_iframe").contentWindow.document.body.scrollHeight + 'px';
  }

  function extractContent(html) {
    //return new DOMParser().parseFromString(html, "text/html").documentElement .textContent;
    return html2plaintext(html);
  }
  const many_cat_colors = [
    "#1C86EE",
    "#E31A1C", // red
    "#008B00",
    "#6A3D9A", // purple
    "#FF7F00", // orange
    "#4D4D4D",
    "#FFD700",
    "#7EC0EE",
    "#FB9A99", // lt pink
    "#90EE90",
    "#FDBF6F", // lt orange
    "#B3B3B3",
    "#EEE685",
    "#B03060",
    "#FF83FA",
    "#FF1493",
    "#00008F",
    "#36648B",
    "#00CED1",
    "#008F00",
    "#8B8B00",
    "#CDCD00",
    "#A52A2A",
  ];

  function find_color(tag) {
    let first_letter = tag.slice(0, 1);
    let index = first_letter.charCodeAt(0) - 97;
    return many_cat_colors[index % many_cat_colors.length];
  }

  function format_to(to) {
    return (
      to
        //.replaceAll("<", "&lt;")
        //.replaceAll(">", "&gt;")
        .replaceAll('"', "")
        .split(",")
        .map((x) => {
          return x.trim();
        })
        .sort()
        .join("<br />")
    );
  }

  function wrap_at_80_chars(text) {
    return text.replace(/(?![^\n]{1,80}$)([^\n]{1,80})\s/g, "$1\n&#x2937;");
  }

  function wrap(text) {
    return text;
    //not sure if I like this
    text = wrap_at_80_chars(text.replaceAll("\n", "--dan--\n")).replaceAll(
      "--dan--\n&#x2937;",
      "\n"
    );
    let lines = text.split("\n");
    lines = lines.map((line) => "<div class=line>" + line + "</div>");
    return lines.join("");
  }

  function handle_overlay_leave() {
    overlay = "";
  }

  let copy_entries = [
    { key: "c", target_path: "link", text: "Copy link" },
    { key: "y", target_path: "text", text: "Copy text" },
    { key: "m", target_path: "html", text: "Copy extract text from html" },
    { key: "M", target_path: "raw_html", text: "Copy raw html" },
    { key: "h", target_path: "headers", text: "Copy headers" },
    { key: "f", target_path: "filename", text: "Copy filename" },
  ];

  function handle_copy(ev) {
    let target = ev.detail;
    if (target == "link") {
      copy_to_clipboard(`[${message.parsed.subject}](mail:${message.id})`);
    } else if (target == "text") {
      if (message.parsed.text != null) {
        copy_to_clipboard(extractContent(message.parsed.text));
      } else {
        error_toast("No text part found");
      }
    } else if (target == "html") {
      if (message.parsed.html != null) {
        copy_to_clipboard(extractContent(message.parsed.html));
      }
    } else if (target == "raw_html") {
      if (message.parsed.html != null) {
        copy_to_clipboard(message.parsed.html);
      } else {
        error_toast("No html part found");
      }
    } else if (target == "headers") {
      let headers = "";
      for (let i = 0; i < message.parsed.headers.length; i++) {
        headers +=
          message.parsed.headers[i].key +
          ": " +
          message.parsed.headers[i].value +
          "\n";
      }
      copy_to_clipboard(headers);
    } else if (target == "filename") {
      copy_to_clipboard(message.filename);
    } else {
      error_toast("Unknown copy target: " + target);
    }
    overlay = "";
  }

  async function handle_tag(ev) {
    let tag = ev.detail;
    if (message_tags.indexOf(tag) == -1) {
      await invoke("mail_message_add_tags", {
        id: message_id,
        tags: [tag],
      });
      message_tags.push(tag);
    } else {
      await invoke("mail_message_remove_tags", {
        id: message_id,
        tags: [tag],
      });
      removeItemOnce(message_tags, tag);
    }
    message_tags = message_tags;
    overlay = "";
  }

  async function get_message(mail_id) {
    let msg = await invoke("get_mail_message", {
      id: mail_id,
    });
    if (msg == null) {
      throw new Error("Could not retrieve message");
    }
    let raw_message = msg.raw;
    let tags = msg.tags;
    console.log(msg);
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
    let res = {
      id: mail_id,
      raw: raw_message,
      parsed: email,
      tags: tags,
      filename: msg.filename,
    };
    message = res;
    return res;
  }
</script>

{#await get_message(params.message_id)}
  Loading...
{:then message}
  <View>
    <div slot="header">
      {#if message == null}
        Could not retrieve this message.
      {:else}
        <table>
          <tr>
            <th>From</th>
            <td>{get_header(message, "from")}</td>
          </tr>
          <tr>
            <th>To</th>
            <td><Expander text={format_to(get_header(message, "to"))} /></td>
          </tr>
          <tr>
            <th>Subject</th>
            <td>{get_header(message, "subject")}</td>
          </tr>
          <tr>
            <th>Date</th>
            <td>
              <SvelteTooltip
                tip={get_header(message, "date")}
                right
                color="#DFDFDF;border:1px dashed grey;"
              >
                {local_date(get_header(message, "date"))}
              </SvelteTooltip>
            </td>
          </tr>
          {#each optional_headers as opt_header}
            {#if get_header(message, opt_header) != null}
              <tr>
                <th>{opt_header}</th>
                {#if opt_header.toLowerCase() == "cc"}
                  <td>{format_to(get_header(message, opt_header))}</td>
                {:else}
                  <td>{get_header(message, opt_header)}</td>
                {/if}
              </tr>
            {/if}
          {/each}
          <tr>
            <th>Tags</th>
            <td>
              {#each message.tags as tag}
                <div class="tags" style="background-color:{find_color(tag)}">
                  {tag}
                </div>
              {/each}
            </td>
          </tr>
        </table>
        {#if all_headers}
          <hr />
          <table>
            {#each message.parsed.headers as header}
              <tr>
                <th>{header.key}</th>
                <td>{header.value}</td>
              </tr>
            {/each}
          </table>
        {/if}
      {/if}
    </div>

    <div slot="content">
      {#if message == null}{:else}
        {#if show_html}
          <iframe
            srcdoc={csp(message.parsed.html)}
            sandbox=""
            style="width:95%; border: 3px solid purple;height:100vh;"
            id="mail_content_iframe"
          />
        {:else if message.parsed.text == null}
          {#if message.parsed.html != null}
            (extracted from html)
            <pre>{wrap(extractContent(message.parsed.html))}</pre>
          {:else}
            (no text, no html)
          {/if}
        {:else}
          {#if message.parsed.html != null}
            (html available){/if}
          <pre>{@html wrap(escape_html(message.parsed.text))}</pre>
        {/if}
      {/if}
    </div>

    <div slot="footer">
      <Overlay {listener} on:leave={handle_overlay_leave} bind:overlay>
        {#if overlay == "help"}
          <Help bind:entries={help_entries} />
        {:else if overlay == "copying"}
          Copy to clipboard:
          <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
        {:else if overlay == "tags"}
          {#if tag_entries.length > 0}
            Toggle Tag:
            <QuickPick bind:entries={tag_entries} on:action={handle_tag} />
          {:else}
            Fill out [mail_tags] in your settings
          {/if}
        {:else if overlay == ""}
          Press <span class="hotkey">h</span> for help.
        {/if}
      </Overlay>
    </div>
  </View>
{/await}

<style>
  th {
    text-align: left;
    padding-right: 10px;
    vertical-align: top;
  }

  /*todo: combine with MailContent*/
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
</style>
