<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import Select from "svelte-select";
  import { LoadBars } from "svelte-loading-animation";
  import { onMount, onDestroy } from "svelte";
  import { fetch, Body } from "@tauri-apps/api/http";
  import { marked } from "marked";
  import hljs from "highlight.js";
  import "../styles/highlight.js/github.css";
  import {
    format_date,
    escape_html,
    add_code_clipboards,
    no_text_inputs_focused,
  } from "./util.ts";
  import DOMPurify from "dompurify";
  import SvelteTooltip from "svelte-tooltip";

  const dispatch = createEventDispatcher();

  export let mode;
  export let mode_args;

  let convo = mode_args.convo;
  let filename = mode_args.filename;
  let input = "";

  let items = [];
  let chosen = null;
  let prompt = "I am a helpful assistant.";
  let processing = false;
  let output;
  let last_input_tokens = 0;
  let processing_title = false;

  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.simple_combo("esc", () => {
    if (overlay_mode == "") {
      dispatch("leave", false);
    }
  });

  listener.simple_combo("shift enter", async () => {
    await query_chat_gtp(input, true);
  });

  listener.simple_combo("s", async (e, count, repeated) => {
    if (no_text_inputs_focused()) {
      dispatch("search", {});
    }
  });

  listener.simple_combo("i", async (e, count, repeated) => {
    if (no_text_inputs_focused()) {
      dispatch("mail_search", {});
    }
  });


  listener.register_combo({
    keys: "h",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        dispatch("overlay_change", { overlay: "toggle_help" });
      }
    },
  });

  listener.register_combo({
    keys: "n",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        dispatch("in_page_search", true);
      }
    },
  });

  listener.register_combo({
    keys: "shift n",
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        dispatch("in_page_search", false);
      }
    },
  });

  async function save_convo() {
    await invoke("chatgpt_save_conversation", {
      conversation: convo,
      filename: filename,
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

    for (let ii = 0; ii < convo.messages.length; ii++) {
      let m = convo.messages[ii];
      if (m[0] == "output") {
        let mm = m[1];
        let cb_input = document.getElementById("include_msg_input_" + ii);
        let cb_output = document.getElementById("include_msg_output_" + ii);
        if (cb_input !== null && cb_input.checked) {
          messages.push({
            role: "user",
            content: convo.messages[ii - 1][1],
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
    convo.messages.push(["input", input]);
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
      convo.messages.push(["error", "Error: " + err]);
    }
    if (!response.ok) {
      convo.messages.push([
        "error",
        "Error: " + response.status + " " + response.statusText,
      ]);
      console.log(response.text());
      console.log(response.data);
    } else {
      if (record_in_convo) {
        let res = response.data;
        res.usage.prompt_tokens_netto =
          res.usage.prompt_tokens - last_input_tokens;
        convo.messages.push(["output", res]);
        response = res.choices[0].message.content;
      }
      await save_convo();
      highlight_code();
    }
    console.log(convo);
    processing = false;
    convo = convo;
    return response;
  }

  onMount(async () => {
    let p = await invoke("chatgpt_get_prompts", {});
    items.length = 0;
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
    listener.listen();
    highlight_code();
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
    let m = convo.messages[index][1];
    m["hide"] = true;
    convo.messages[index][1] = m;
    convo = convo;
    await save_convo();
  }

  async function unhide(index) {
    let m = convo.messages[index][1];
    m["hide"] = false;
    convo.messages[index][1] = m;
    convo = convo;
    await save_convo();
  }
  async function handle_title_generate() {
    processing_title = true;
    convo.title = await query_chat_gtp(
      "Write a title for this conversation, make it brief",
      false
    );
    await save_convo();
    processing_title = false;
  }

</script>

<div>
  <h1>ChatGPT</h1>
 </div>

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
    width: 100%;
    padding: 0.25em;
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
