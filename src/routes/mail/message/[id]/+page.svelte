<script lang="ts">
  import { appWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/tauri";
  import { toast } from "@zerodevx/svelte-toast";
  //import { get_last_path } from "../lib/mode_stack.ts";
  import {
    escape_html,
    error_toast,
    removeItemOnce,
    dispatch_keyup,
    focus_first_in_node,
    format_date,
  } from "$lib/util.ts";
  import { createEventDispatcher } from "svelte";
  import { writeText as copy_to_clipboard } from "@tauri-apps/api/clipboard";
  import { goto } from "$app/navigation";
  import { tag_class } from "$lib/colors.ts";
  import { onMount, onDestroy, afterUpdate } from "svelte";
  import SvelteTooltip from "svelte-tooltip";
  import html2plaintext from "html2plaintext";
  import Expander from "$lib/../components/Expander.svelte";
  import View from "$lib/../components/View.svelte";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import Search from "$lib/../components/Search.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Goto from "$lib/../components/Goto.svelte";
  import linkifyStr from "linkify-string";

  export let data;
  let viewComponent;

  let tag_entries = Object.keys(data.available_tags ?? []).map((key) => {
    return {
      key: key,
      text: data.available_tags[key],
      target_path: data.available_tags[key],
    };
  });
  const dispatch = createEventDispatcher();

  let overlay = "";
  let search_mode;
  let in_page_search_term;

  let show_html = false;
  let show_images = false;
  let all_headers = false;

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "r", text: "reply menu" },
    { key: "l", text: "toggle html" },
    { key: "i", text: "toggle images" },
    { key: "c", text: "copy menu" },
    { key: "d", text: "dump attachments" },
    { key: "H", text: "Show all headers" },
    { key: "s", text: "search" },
    { key: "n/N", text: "in page search" },
  ];

  let keys = {
    Escape: () => {
      if (overlay != "") {
        viewComponent.leave_overlay();
        return true;
      }
    },

    l: () => {
      if (data.html != null && data.html != "") {
        show_html = !show_html;
        return true;
      }
    },
    h: () => {
      viewComponent.enter_overlay("help");
      return true;
    },
    m: () => {
      toggle_tag({ detail: "unread" });
      return true;
    },
    H: () => {
      all_headers = !all_headers;
      return true;
    },
    f: () => {
      toggle_tag({ detail: "flagged" });
    },
    t: () => {
      viewComponent.enter_overlay("tags");
      return true;
    },
    d: async () => {
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
    c: () => {
      viewComponent.enter_overlay("copying");
      return true;
    },
    i: () => {
      if (data.html != null) {
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
    s: () => {
      viewComponent.enter_overlay("search");
      search_mode = "pick";
      return true;
    },
    n: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, false);
      } else {
        viewComponent.enter_overlay("search");
        search_mode = "in_page";
      }
      return true;
    },
    N: () => {
      if (in_page_search_term != "") {
        window.find(in_page_search_term, false, false, true, True);
      } else {
        viewComponent.enter_overlay("search");
        search_mode = "in_page";
      }
      return true;
    },
    g: () => {
      viewComponent.enter_overlay("goto");
      return true;
    },
    r: () => {
      viewComponent.enter_overlay("reply");
      return true;
    },
  };

  function handle_keys(ev) {
    return dispatch_keyup(keys)(ev);
  }

  onMount(async () => {
    focus_first_in_node(document.getElementById("wrapper"));
  });

  afterUpdate(() => {
    //focus_first_in_node(document.getElementById("wrapper"));
  });

  function get_header(parsed, header) {
    for (let i = 0; i < parsed.headers.length; i++) {
      if (parsed.headers[i].name.Rfc?.toLowerCase()! == header) {
        let value;
        if ("Address" in parsed.headers[i].value) {
          if (parsed.headers[i].value.Address.name != null) {
            value = `${parsed.headers[i].value.Address.name} <${parsed.headers[i].value.Address.address}>`;
          } else {
            value = `${parsed.headers[i].value.Address.address}`;
          }
        } else if ("DateTime" in parsed.headers[i].value) {
          console.log("date");
          let input_date = parsed.headers[i].value.DateTime;
          value = new Date(
            input_date.year,
            input_date.month - 1,
            input_date.day,
            input_date.hour,
            input_date.minute,
            input_date.second
          );
        } else {
          value = parsed.headers[i].value.Text;
        }
        return value;
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

  let copy_entries = [
    { key: "c", target_path: "link", text: "Copy link" },
    { key: "y", target_path: "text", text: "Copy text" },
    { key: "m", target_path: "html", text: "Copy extract text from html" },
    { key: "M", target_path: "raw_html", text: "Copy raw html" },
    { key: "h", target_path: "headers", text: "Copy headers" },
    { key: "f", target_path: "filename", text: "Copy filename" },
  ];

  let reply_entries = [
    { key: "r", target_path: "reply", text: "reply" },
    { key: "n", target_path: "new_to_sender", text: "new message to sender" },
    { key: "N", target_path: "new", text: "new message (no receiver)" },
    { key: "A", target_path: "reply_all", text: "Reply all" },
  ];

  function handle_copy(ev) {
    let target = ev.detail;
    if (target == "link") {
      copy_to_clipboard(`<<mail:${data.id}>>`);
    } else if (target == "text") {
      if (data.text != null) {
        copy_to_clipboard(extractContent(data.text));
      } else {
        error_toast("No text part found");
      }
    } else if (target == "html") {
      if (data.html != null) {
        copy_to_clipboard(extractContent(data.html));
      }
    } else if (target == "raw_html") {
      if (data.html != null) {
        copy_to_clipboard(data.html);
      } else {
        error_toast("No html part found");
      }
    } else if (target == "headers") {
      let headers = "";
      for (let i = 0; i < data.headers.length; i++) {
        headers += data.headers[i].key + ": " + data.headers[i].value + "\n";
      }
      copy_to_clipboard(headers);
    } else if (target == "filename") {
      copy_to_clipboard(data.filename);
    } else {
      error_toast("Unknown copy target: " + target);
    }
    viewComponent.leave_overlay();
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
    viewComponent.leave_overlay();
  }

  async function handle_reply(ev) {
    switch (ev.detail) {
      case "reply":
        await invoke("mail_message_reply", {
          id: data.id,
          reply_all: false,
        });
        break;
      case "reply_all":
        await invoke("mail_message_reply", {
          id: data.id,
          reply_all: true,
        });
        break;
      case "new":
        await invoke("mail_message_new", {
          id: null,
          windowTitle: appWindow.label,
        });
        break;
      case "new_to_sender":
        await invoke("mail_message_new", {
          id: data.id,
          windowTitle: appWindow.label,
        });
        break;
      default:
        error_toast("Unknown reply target: " + ev.detail);
    }
    viewComponent.leave_overlay();
  }

 
  function onMessage(msg) {
    toast.push(msg);
  }

  function make_links_target_blank(html) {
	return html.replace("<a", "<a target='_blank' ");
  }


</script>

<svelte:window onMessage={on_message} />

<View bind:this={viewComponent} bind:overlay>
  <div slot="header">
    <table>
      <tr>
        <th>From</th>
        <td>{get_header(data, "from")}</td>
      </tr>
      <tr>
        <th>To</th>
        <td><Expander text={format_to(get_header(data, "to"))} /></td>
      </tr>
      <tr>
        <th>Subject</th>
        <td>{get_header(data, "subject")}</td>
      </tr>
      <tr>
        <th>Date</th>
        <td>
          <SvelteTooltip
            tip={get_header(data, "date")}
            right
            color="#DFDFDF;border:1px dashed grey;"
          >
            {local_date(get_header(data, "date"))}
          </SvelteTooltip>
        </td>
      </tr>
      {#each optional_headers as opt_header}
        {#if get_header(data, opt_header) != null}
          <tr>
            <th>{opt_header}</th>
            {#if opt_header.toLowerCase() == "cc"}
              <td>{format_to(get_header(data, opt_header))}</td>
            {:else}
              <td>{get_header(data, opt_header)}</td>
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

  <div slot="content" on:keyup={handle_keys} tabindex="0" autofocus>
    {#if all_headers}
      <table>
        {#each data.headers as header}
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
        srcdoc={make_links_target_blank(csp(data.html))}
        sandbox="allow-same-origin"
        style="width:95%; border: 3px solid purple;height:100vh; font-size:18pt;"
        id="mail_content_iframe"
      />
    {:else if data.text == null}
      {#if data.html != null}
        (extracted from html)
        <pre>{wrap(make_links_target_blank(linkifyStr(extractContent(data.html))))}</pre>
      {:else}
        (no text, no html)
      {/if}
    {:else}
      {#if data.html != null && data.html != ""}
        (html available){/if}
      <pre class="my_pre">{@html wrap(
          make_links_target_blank(linkifyStr(data.text, { defaultProtocol: "https" }))
        )}</pre>
    {/if}
    <!-- <pre>
	todo: replace mailto links!
	{JSON.stringify(data.parsed, null, 2)}
	</pre> -->
  </div>

  <div slot="overlays">
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
    {:else if overlay == "search"}
      <Search bind:overlay bind:in_page_search_term bind:search_mode on:leave />
    {:else if overlay == "goto"}
      <Goto />
    {:else if overlay == "reply"}
      <QuickPick bind:entries={reply_entries} on:action={handle_reply} />
    {:else if overlay == ""}
      Press <span class="hotkey">h</span> for help.
    {/if}
  </div>
</View>

<style>
  th {
    text-align: left;
    padding-right: 10px;
    vertical-align: top;
  }

  .my_pre {
    margin-left: 0.25em;
  }

  /*todo: combine with MailContent*/
</style>
