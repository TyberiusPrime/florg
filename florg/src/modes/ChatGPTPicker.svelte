<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { enter_mode, leave_mode, get_last_path } from "../lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "../lib/Picker.svelte";
  import { get_node } from "../lib/util.ts";

  async function handle_action(ev) {
    let filename = ev.detail.cmd;
    let convo;
    if (filename == "") {
      convo = await invoke("chatgpt_new_conversation", {});
      filename = new Date().toISOString() + ".json";
    } else {
      convo = await invoke("chatgpt_get_conversation", {
        filename: filename,
      });
    }
    enter_mode("chatgpt", { filename: filename, convo: convo }, false);
  }

  export let mode;
  export let mode_args;

  let start_text = mode_args.start_text;

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
