<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/tauri";
  import { get_last_path } from "$lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "$lib/../components/Picker.svelte";
  import { get_node } from "$lib/util.ts";
  import { onMount, onDestroy } from "svelte";

  export let data;

  async function handle_action(ev) {
    goto("/node/" + ev.detail.cmd, { replaceState: true });
  }
  let search_results = [];
</script>

<div>
  <Picker on:action={handle_action}>
    <div slot="message"><h1>Node search</h1></div>
    <svelte:fragment slot="entries">
      {#each data.search_results as result}
        <tr data-cmd={result.cmd}><td>{@html result.text}</td></tr>
      {/each}
    </svelte:fragment>
  </Picker>
</div>
