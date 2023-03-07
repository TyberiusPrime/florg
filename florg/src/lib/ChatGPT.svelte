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
  import { format_date, escape_html, add_code_clipboards } from "./util.ts";
  import DOMPurify from "dompurify";
  import SvelteTooltip from "svelte-tooltip";

  const dispatch = createEventDispatcher();

  export let convo = null;
  export let filename = null;

  let items = [];
  let chosen = null;
  let prompt = "I am a helpful assistant.";
  let input = "";
  let processing = false;
  let output;
  let last_input_tokens = 0;
  let processing_title = false;

  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.simple_combo("esc", () => {
    dispatch("leave", false);
  });

  listener.simple_combo("shift enter", async () => {
    await query_chat_gtp(input, true);
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
        let mm = JSON.parse(m[1]);
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
        convo.messages.push(["output", JSON.stringify(res)]);
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
    let m = JSON.parse(convo.messages[index][1]);
    m["hide"] = true;
    convo.messages[index][1] = JSON.stringify(m);
    convo = convo;
    await save_convo();
  }

  async function unhide(index) {
    let m = JSON.parse(convo.messages[index][1]);
    m["hide"] = false;
    convo.messages[index][1] = JSON.stringify(m);
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
  <table>
    <tr>
      <th>Date</th>
      <td>{@html format_date(convo.date)}</td>
      1G
    </tr>
    <tr>
      <th>Title</th>
      <td>
        <input
          type="text"
          bind:value={convo.title}
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
    {#each convo.messages as message, index}
      {#if message[0] == "input"}
        {#if index > convo.messages.length - 1 || convo.messages[index + 1][0] != "output" || JSON.parse(convo.messages[index + 1][1]).hide !== true}
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
        {@const output = JSON.parse(message[1])}
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
              <!--{message[1]} -->
            </td>
          </tr>
          <tr><td colspan="2"><hr style="border: 1px dashed black" /></td></tr>
        {:else}
          <tr
            ><td colspan="2">
              <SvelteTooltip
                color="#DFDFDF;border:1px dashed grey;"
              >
			  <div slot="custom-tip" class="hover">
				{convo.messages[index-1][1]}<br />
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
          <tr><td colspan="2"><hr style="border: 1px dashed black" /></td></tr>
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
        <textarea id="input" bind:value={input} rows="10" autofocus /></td
      >
    </tr>
  </table>
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
