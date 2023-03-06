<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import SvelteTooltip from "svelte-tooltip";
  import html2plaintext from "html2plaintext";

  const dispatch = createEventDispatcher();

  export let message = null;
  export let message_id = null;
  export let message_tags = null;
  let show_html = false;
  let show_images = false;

  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  onMount(async () => {
    listener.listen();
  });

  onDestroy(() => {
    listener.stop_listening();
  });

  listener.register_combo({
    keys: "escape",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      dispatch("leave", null);
    },
  });

  listener.register_combo({
    keys: "h",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.html != null) {
        show_html = !show_html;
      }
    },
  });
  listener.register_combo({
    keys: "i",
    prevent_default: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (message.html != null) {
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
    for (let i = 0; i < message.headers.length; i++) {
      if (
        message.headers[i].key != null &&
        message.headers[i].key.toLowerCase()! == header
      ) {
        return message.headers[i].value;
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
    console.log("resize_iframe");
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
</script>

<div>
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
        <td>{get_header(message, "to")}</td>
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
            <td>{get_header(message, opt_header)}</td>
          </tr>
        {/if}
      {/each}
      <tr>
        <th>Tags</th>
        <td>
          {#each message_tags as tag}
            <div class="tags" style="background-color:{find_color(tag)}">
              {tag}
            </div>
          {/each}
        </td>
      </tr>
    </table>
    {#if show_html}
      <iframe
        srcdoc={csp(message.html)}
        sandbox=""
        style="width:95%; border: 3px solid purple;height:100vh;"
        id="mail_content_iframe"
      />
    {:else if message.text == null}
      {#if message.html != null}
        (extracted from html)
        <pre>{extractContent(message.html)}</pre>
      {:else}
        (no text, no html)
      {/if}
    {:else}
      {#if message.html != null}
        (html available){/if}
      <pre>{message.text}</pre>{/if}
  {/if}
</div>

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
