<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";

  import { onMount, onDestroy, afterUpdate } from "svelte";
  import View from "$lib/../components/View.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import Help from "$lib/../components/Help.svelte";
  import {
    format_date,
    add_code_clipboards,
    no_text_inputs_focused,
  } from "../../../lib/util.ts";
  import Select from "svelte-select";
  import { LoadBars } from "svelte-loading-animation";
  import { keypress } from "keypress.js";
  import DOMPurify from "dompurify";
  import SvelteTooltip from "svelte-tooltip";

  import { writeText as copy_to_clipboard } from "@tauri-apps/api/clipboard";
  import { fetch, Body } from "@tauri-apps/api/http";
  import { marked } from "marked";
  import hljs from "highlight.js";

  export let data;
  let overlay = "";
  let in_page_search_term = "";
  let search_mode;

  export let input = "";
  let items = [];
  let chosen = null;
  let prompt = "I am a helpful assistant.";
  let processing = false;
  let output;
  let last_input_tokens = 0;
  let processing_title = false;
  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "s", text: "Search" },
    { key: "n/N", text: "Search forward/rev in page" },
    { key: "c", text: "copy" },
  ];
  let copy_entries = [
    { key: "c", text: "link", target_path: "link" },
    { key: "y", text: "content", target_path: "content" },
    { key: "t", text: "title", target_path: "title" },
  ];

  var listener = new keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.simple_combo("esc", () => {
    window.history.back();
  });

  listener.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: false,
    is_exclusive: true,
    on_keyup: async (e, count, repeated) => {
      console.log("etner");
    },
  });

  listener.register_combo({
    keys: "shift enter",
    prevent_repeat: true,
    prevent_default: false,
    is_exclusive: true,
    on_keyup: async (e, count, repeated) => {
      await query_chat_gtp(input, true);
    },
  });

  listener.register_combo({
    keys: "s",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        overlay = "search";
        e.preventDefault();
      }
    },
  });

  listener.register_combo({
    keys: "i",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        toast.push("todo");
        e.preventDefault();
      }
    },
  });

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
    keys: "n",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        if (in_page_search_term != "") {
          window.find(in_page_search_term, false, false, true, false);
        } else {
          overlay = "search";
          search_mode = "in_page";
        }
      }
    },
  });

  listener.register_combo({
    keys: "shift n",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        if (in_page_search_term != "") {
          window.find(in_page_search_term, false, false, true, false);
        } else {
          overlay = "search";
          search_mode = "in_page";
        }
      }
    },
  });

  listener.register_combo({
    keys: "c",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused() && !e.ctrlKey && !e.metaKey) {
        overlay = "copying";
        e.preventDefault();
      }
    },
  });

  async function save_convo() {
    await invoke("chatgpt_save_conversation", {
      conversation: data,
      filename: data.filename,
    });
  }

  async function query_chat_gtp(input, record_in_convo = true) {
    if (input == "") {
      return;
    }
    processing = true;
    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = await invoke("chatgpt_get_api_key", {});
    let messages = [];
    messages.push({
      role: "system",
      content: prompt,
    });
    last_input_tokens = 0;

    for (let ii = 0; ii < data.messages.length; ii++) {
      let m = data.messages[ii];
      if (m[0] == "output") {
        let mm = m[1];
        let cb_input = document.getElementById("include_msg_input_" + ii);
        let cb_output = document.getElementById("include_msg_output_" + ii);
        if (cb_input !== null && cb_input.checked) {
          messages.push({
            role: "user",
            content: data.messages[ii - 1][1],
          });
          last_input_tokens +=
            mm.usage.prompt_tokens_netto != null
              ? mm.usage.prompt_tokens_netto
              : mm.usage.prompt_tokens;
        }
        if (cb_output !== null && cb_output.checked) {
          messages.push({
            role: "assistant",
            content: mm.choices[0].message.content,
          });
          last_input_tokens += mm.usage.completion_tokens;
        }
      }
    }

    messages.push({
      role: "user",
      content: input,
    });
    console.log(JSON.stringify(messages, 2, null));
    //return
    if (record_in_convo) {
      data.messages.push(["input", input]);
      data = data;
    }

    let args = {
      method: "POST",
      timeout: 1000,
      responseType: 1,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${api_key}`,
      },
      body: Body.json({
        //model: "gpt-3.5-turbo",
        //messages: messages,
        model: "gpt-3.5-turbo",
        messages: messages,
      }),
    };
    console.log(args);
    let response = null;
    try {
      response = await fetch(url, args);
    } catch (err) {
      data.messages.push(["error", "Error: " + err]);
      console.log(err);
      processing = false;
      return;
    }
    if (!response.ok) {
      data.messages.push([
        "error",
        "Error: " + response.status + " " + response.statusText,
      ]);
      console.log(response.text());
      console.log(response.data);
    } else {
      let res = response.data;
      response = res.choices[0].message.content;
      if (record_in_convo) {
        res.usage.prompt_tokens_netto =
          res.usage.prompt_tokens - last_input_tokens;
        data.messages.push(["output", res]);
      }
      await save_convo();
      highlight_code();
    }
    console.log(data);
    processing = false;
    data = data;
    return response;
  }

  onMount(async () => {
    let p = await invoke("chatgpt_get_prompts", {});
    items.length = 0;
    //prompt items
    for (let outer_key in p) {
      for (let inner_key in p[outer_key]) {
        items.push({
          value: `${outer_key} - ${inner_key}`,
          label: `${outer_key} - ${inner_key}`,
          content: p[outer_key][inner_key],
        });
        if (outer_key == "default" && inner_key == "default") {
          chosen = `${outer_key} - ${inner_key}`;
        }
      }
    }
    //listener.listen();
    highlight_code();
    document.getElementById("input").scrollIntoView({
      behavior: "smooth",
      block: "end",
    });
  });

  function highlight_code() {
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
    });
    add_code_clipboards();
  }

  onDestroy(() => {
    listener.stop_listening();
  });

  function set_prompt(ev) {
    prompt = ev.detail.content;
  }

  async function handle_title_change(ev) {
    await save_convo();
  }

  function disable_all() {
    let checkboxes = document.getElementsByClassName("include_msg_input");
    for (let i = 0; i < checkboxes.length; i++) {
      checkboxes[i].checked = false;
    }
    checkboxes = document.getElementsByClassName("include_msg_output");
    for (let i = 0; i < checkboxes.length; i++) {
      checkboxes[i].checked = false;
    }
  }
  function toggle_pre(ev) {
    let element = ev.srcElement.parentElement.previousElementSibling;
    let innerHTML = element.innerHTML;
    if (innerHTML.startsWith("<pre>")) {
      innerHTML = innerHTML.slice(5, -6);
    } else {
      innerHTML = "<pre>" + innerHTML + "</pre>";
    }
    element.innerHTML = innerHTML;
  }

  async function hide(index) {
    let m = data.messages[index][1];
    m["hide"] = true;
    data.messages[index][1] = m;
    data = data;
    await save_convo();
  }

  async function unhide(index) {
    let m = data.messages[index][1];
    m["hide"] = false;
    data.messages[index][1] = m;
    data = data;
    await save_convo();
  }
  async function handle_title_generate() {
    processing_title = true;
    data.title = await query_chat_gtp(
      "Write a title for this conversation, make it brief",
      false
    );
    await save_convo();
    processing_title = false;
  }

  function handle_overlay_leave() {
    overlay = "";
  }

  async function handle_copy(ev) {
    let mode = ev.detail;
    console.log("copy_to_clipboard", mode);
    let out = null;
    if (mode == "link") {
      out = `<<<chatgpt:${filename}>>>`;
    } else if (mode == "content") {
      out = "";
      for (let i = 0; i < data.messages.length; i++) {
        let m = data.messages[i];
        if (m[0] == "input") {
          out +=
            "Input:\n\t" + trim_eol(m[1].replaceAll("\n", "\n\t")) + "\n\n";
        } else if (m[0] == "output") {
          out +=
            "Output:\n\t" +
            trim_eol(m[1].choices[0].message.content.replaceAll("\n", "\n\t")) +
            "\n\n";
        }
      }
    } else if (mode == "title") {
      out = data.title;
    } else {
      console.log("unknown copy_to_clipboard mode", mode);
    }
    if (out != null) {
      await copy_to_clipboard(out);
    }
    overlay = "";
  }

  async function submit(ev) {
    await query_chat_gtp(input, true);
  }
</script>

<View>
  <div slot="header">
    <h1>ChatGPT</h1>
  </div>

  <div slot="content">
    <table>
      <tr>
        <th>Date</th>
        <td>{@html format_date(data.date)}</td>
      </tr>
      <tr>
        <th>Title</th>
        <td>
          <input
            type="text"
            bind:value={data.title}
            on:change={handle_title_change}
            style="width: 70%;"
          />
          {#if processing_title}
            <button><LoadBars color="#303030" size="1em" ; /></button>
          {:else}
            <button on:click={handle_title_generate}>Generate</button>
          {/if}
        </td>
      </tr>
      <tr>
        <th>Prompt</th>
        <td>
          <Select
            {items}
            bind:value={chosen}
            on:change={set_prompt}
            placeholder=" Replace prompt"
          />
          <textarea id="prompt" bind:value={prompt} rows="5" />
        </td>
      </tr>
      {#each data.messages as message, index}
        {#if message[0] == "input"}
          {#if index > data.messages.length - 2 || data.messages[index + 1][0] != "output" || data.messages[index + 1][1].hide !== true}
            <tr>
              <th
                style="background-color:#eb86bf;color:#EEEEEE;border-radius:.5em;"
                class="unselectable">Input</th
              >
              <td class="llm_input"> {message[1]} </td>
            </tr>
          {:else}{/if}
        {/if}
        {#if message[0] == "output"}
          {@const output = message[1]}
          {#if output.hide !== true}
            <tr>
              <th
                style="background-color:#10a37f;color:#EEEEEE;border-radius:.5em;"
                class="unselectable"
              >
                Output</th
              ><td>
                <div class="llm_output">
                  {@html DOMPurify.sanitize(
                    marked.parse(output.choices[0].message.content)
                  )}
                </div>
                <div class="unselectable">
                  <input
                    type="checkbox"
                    class="include_msg_input"
                    id="include_msg_input_{index}"
                    data-index={index}
                    checked
                  />
                  <label for="include_msg_input_{index}"
                    >Include input ({output.usage.prompt_tokens})</label
                  >
                  <input
                    type="checkbox"
                    class="include_msg_output"
                    id="include_msg_output_{index}"
                    data-index={index}
                    checked
                  />
                  <label for="include_msg_output_{index}"
                    >Include output ({output.usage.completion_tokens})</label
                  >
                  <button on:click={toggle_pre} class="small_button"
                    >Toggle pre</button
                  >
                  <button
                    class="small_button"
                    on:click={() => {
                      hide(index);
                    }}>Hide</button
                  >
                </div>
              </td>
            </tr>
            <tr><td colspan="2"><hr style="border: 1px dashed black" /></td></tr
            >
          {:else}
            <tr
              ><td /><td>
                <SvelteTooltip color="#DFDFDF;border:1px dashed grey;">
                  <div slot="custom-tip" class="hover">
                    {data.messages[index - 1][1]}<br />
                    {output.choices[0].message.content}
                  </div>
                  <button
                    class="small_button"
                    on:click={() => {
                      unhide(index);
                    }}>Unhide</button
                  >
                </SvelteTooltip>
              </td></tr
            >
            <tr><td colspan="2"><hr style="border: 1px dashed black" /></td></tr
            >
          {/if}
        {/if}
        {#if message[0] == "error"}
          <tr>
            <th> Error</th><td>
              <div class="llm_error">{message[1]}</div>
            </td>
          </tr>
        {/if}
      {/each}
      {#if processing}
        <tr>
          <td colspan="2"><LoadBars color="#303030" /> </td>
        </tr>
      {/if}

      <tr>
        <th>New input</th>
        <td>
          <a on:click={disable_all}>Disable previous</a>
          <textarea
            id="input"
            bind:value={input}
            rows="5"
            autofocus
            autocorrect="on"
            spellcheck="on"
          />
          <button on:click={submit}> Submit</button>
        </td>
      </tr>
    </table>
  </div>

  <div slot="footer">
    <Overlay {listener} on:leave={handle_overlay_leave} bind:overlay>
      {#if overlay == "help"}
        <Help bind:entries={help_entries} />
      {:else if overlay == "search"}
        <Search bind:overlay bind:in_page_search_term bind:search_mode />
      {:else if overlay == "goto"}
        Goto node:
        <Goto on:action={handle_goto_action} />
      {:else if overlay == "new_below"}
        Create new node below
        <Goto on:action={handle_new_node_below} />
      {:else if overlay == "copying"}
        <QuickPick bind:entries={copy_entries} on:action={handle_copy} />
      {:else if overlay == ""}
        Press <span class="hotkey">h</span> for help.
      {:else}
        Unknown overlay: {overlay}
      {/if}
    </Overlay>
  </div>
</View>

<style>
  table {
    width: 100%;
    border-spacing: 0 0.25em;
  }
  th,
  td {
    text-align: left;
    vertical-align: top;
    padding: 0.25em;
  }

  input {
    width: auto;
    padding-left: 0.25em;
  }
  textarea {
    font-size: 1em;
    width: 95%;
    padding: 0.25em;
    resize: vertical;
  }

  .llm_input {
    background-color: #ffffff;
    padding: 1em;
    border-radius: 0.5em;
  }
  .llm_output {
    padding: 1em;
    background-color: #dddddd;
    border-radius: 0.5em;
    padding-top: 0.25em;
    padding-bottom: 0.25em;
  }

  .unselectable {
    -moz-user-select: -moz-none;
    -khtml-user-select: none;
    -webkit-user-select: none;
    -o-user-select: none;
    user-select: none;
  }
</style>
