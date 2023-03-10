<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { get_last_path } from "../lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "../lib/Picker.svelte";
  import { get_node } from "../lib/util.ts";
  import { push as push_mode, replace as replace_mode, } from "svelte-spa-router";

  async function handle_action(ev) {
    let filename = ev.detail.cmd;
    if (filename == "") {
      filename = new Date().toISOString() + ".json";
    }

	push_mode("/chatgpt/" + filename);
  }

  let start_text = get_last_path();
  let mode_args;

  async function get_convos() {
    return await invoke("chatgpt_list_conversations", {});
  }
</script>

<div>
  <Picker on:action={handle_action}>
    <div slot="message"><h1>Pick a ChatGPT conversation</h1></div>
    <svelte:fragment slot="entries">
      <tr data-cmd=""><td>New conversation</td></tr>
      {#await get_convos() then convos}
        {#each convos as convo}
          <tr data-cmd={convo.filename}><td>{convo.title}</td></tr>
        {/each}
      {/await}
    </svelte:fragment>
  </Picker>
</div>
