<script lang="ts">
  import { keypress } from "keypress.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { toast } from "@zerodevx/svelte-toast";
  //import { get_last_path } from "../lib/mode_stack.ts";
  import { escape_html, error_toast, removeItemOnce } from "$lib/util.ts";
  import { createEventDispatcher } from "svelte";
  import { writeText as copy_to_clipboard } from "@tauri-apps/api/clipboard";
  import { goto } from "$app/navigation";
  import { tag_class } from "$lib/colors.ts";
  import PostalMime from "postal-mime";
  import { onMount, onDestroy } from "svelte";
  import SvelteTooltip from "svelte-tooltip";
  import html2plaintext from "html2plaintext";
  import Expander from "$lib/../components/Expander.svelte";
  import View from "$lib/../components/View.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Goto from "$lib/../components/Goto.svelte";

  export let data;

  let tag_entries = Object.keys(data.available_tags).map((key) => {
    return {
      key: key,
      text: data.available_tags[key],
      target_path: data.available_tags[key],
    };
  });
  const dispatch = createEventDispatcher();

  let overlay = "";

  let show_html = false;
  let show_images = false;
  let all_headers = false;

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "l", text: "toggle html" },
    { key: "i", text: "toggle images" },
    { key: "c", text: "copy menu" },
    { key: "H", text: "Show all headers" },
  ];

  var listener = new keypress.Listener();
  listener.reset();
  listener.stop_listening();

  onMount(async () => {
    overlay = "";
    listener.listen();
    window.scroll(0, 0);
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
    keys: "l",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (data.parsed.html != null) {
        show_html = !show_html;
      }
    },
  });
  listener.register_combo({
    keys: "h",
    prevent_default: true,
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      console.log("help");
      overlay = "help";
    },
  });

  listener.register_combo({
    keys: "m",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      toggle_tag({ detail: "unread" });
    },
  });
  listener.register_combo({
    keys: "f",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      toggle_tag({ detail: "flagged" });
    },
  });

  listener.register_combo({
    keys: "t",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "tags";
    },
  });

  listener.register_combo({
    keys: "p",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: async (e, count, repeated) => {
      if (data.tags.indexOf("attachment") != -1) {
        if (
          await invoke("mail_message_store_attachments", {
            id: data.id,
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
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      all_headers = !all_headers;
    },
  });

  listener.register_combo({
    keys: "c",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
        overlay = "copying";
    },
  });

  listener.register_combo({
    keys: "t",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      overlay = "tags";
    },
  });

  listener.register_combo({
    keys: "i",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (data.parsed.html != null) {
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

  listener.register_combo({
    keys: "g",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      overlay = "goto";
    },
  });

  function get_header(parsed, header) {
    for (let i = 0; i < parsed.headers.length; i++) {
      if (
        parsed.headers[i].key != null &&
        parsed.headers[i].key.toLowerCase()! == header
      ) {
        return parsed.headers[i].value;
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
  function format_to(to) {
	 if (to == null) {
		  return "";
		}
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
      copy_to_clipboard(`[${data.parsed.subject}](mail:${message.id})`);
    } else if (target == "text") {
      if (data.parsed.text != null) {
        copy_to_clipboard(extractContent(data.parsed.text));
      } else {
        error_toast("No text part found");
      }
    } else if (target == "html") {
      if (data.parsed.html != null) {
        copy_to_clipboard(extractContent(data.parsed.html));
      }
    } else if (target == "raw_html") {
      if (data.parsed.html != null) {
        copy_to_clipboard(data.parsed.html);
      } else {
        error_toast("No html part found");
      }
    } else if (target == "headers") {
      let headers = "";
      for (let i = 0; i < data.parsed.headers.length; i++) {
        headers +=
          data.parsed.headers[i].key +
          ": " +
          data.parsed.headers[i].value +
          "\n";
      }
      copy_to_clipboard(headers);
    } else if (target == "filename") {
      copy_to_clipboard(data.filename);
    } else {
      error_toast("Unknown copy target: " + target);
    }
    overlay = "";
  }

  async function toggle_tag(ev) {
    let tag = ev.detail;
    if (data.tags.indexOf(tag) == -1) {
      await invoke("mail_message_add_tags", {
        id: data.id,
        tags: [tag],
      });
      data.tags.push(tag);
    } else {
      await invoke("mail_message_remove_tags", {
        id: data.id,
        tags: [tag],
      });
      removeItemOnce(data.tags, tag);
    }
    data = data;
    overlay = "";
  }
</script>

<View>
  <div slot="header">
    <table>
      <tr>
        <th>From</th>
        <td>{get_header(data.parsed, "from")}</td>
      </tr>
      <tr>
        <th>To</th>
        <td><Expander text={format_to(get_header(data.parsed, "to"))} /></td>
      </tr>
      <tr>
        <th>Subject</th>
        <td>{get_header(data.parsed, "subject")}</td>
      </tr>
      <tr>
        <th>Date</th>
        <td>
          <SvelteTooltip
            tip={get_header(data.parsed, "date")}
            right
            color="#DFDFDF;border:1px dashed grey;"
          >
            {local_date(get_header(data.parsed, "date"))}
          </SvelteTooltip>
        </td>
      </tr>
      {#each optional_headers as opt_header}
        {#if get_header(data.parsed, opt_header) != null}
          <tr>
            <th>{opt_header}</th>
            {#if opt_header.toLowerCase() == "cc"}
              <td>{format_to(get_header(data.parsed, opt_header))}</td>
            {:else}
              <td>{get_header(data.parsed, opt_header)}</td>
            {/if}
          </tr>
        {/if}
      {/each}
      <tr>
        <th>Tags</th>
        <td>
          {#each data.tags as tag}
			<div class="tags {tag_class(tag)}">
              {tag}
            </div>
          {/each}
        </td>
      </tr>
    </table>
      </div>

  <div slot="content">
{#if all_headers}
      <table>
        {#each data.parsed.headers as header}
          <tr>
            <th>{header.key}</th>
            <td>{header.value}</td>
          </tr>
        {/each}
      </table>
      <hr />
    {/if}

    {#if show_html}
      <iframe
        srcdoc={csp(data.parsed.html)}
        sandbox=""
        style="width:95%; border: 3px solid purple;height:100vh;"
        id="mail_content_iframe"
      />
    {:else if data.parsed.text == null}
      {#if data.parsed.html != null}
        (extracted from html)
        <pre>{wrap(extractContent(data.parsed.html))}</pre>
      {:else}
        (no text, no html)
      {/if}
    {:else}
      {#if data.parsed.html != null}
        (html available){/if}
      <pre>{@html wrap(escape_html(data.parsed.text))}</pre>
    {/if}
  </div>

  <div slot="footer">
    {overlay}
    <Overlay {listener} on:leave={handle_overlay_leave} bind:overlay>
      {#if overlay == "help"}
        <Help bind:entries={help_entries} />
      {:else if overlay == "copying"}
        Copy to clipboard:
        <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
      {:else if overlay == "tags"}
        {#if tag_entries.length > 0}
          Toggle Tag:
          <QuickPick bind:entries={tag_entries} on:action={toggle_tag} />
        {:else}
          Fill out [mail_tags] in your settings
        {/if}
      {:else if overlay == "goto"}
        <Goto />
      {:else if overlay == ""}
        Press <span class="hotkey">h</span> for help.
      {/if}
    </Overlay>
  </div>
</View>

<style>
  th {
    text-align: left;
    padding-right: 10px;
    vertical-align: top;
  }

  /*todo: combine with MailContent*/
</style>
