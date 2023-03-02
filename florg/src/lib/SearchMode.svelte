<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let search_term = "";

  async function handle(e) {
    if (e.key == "Enter") {
      dispatch("leave", true);
    } else if (e.key == "Escape") {
      dispatch("leave", false);
    }
  }
</script>

<div>
    <input
      id="search_input-input"
      bind:value={search_term}
      placeholder="search text"
      on:keypress={handle}
      autofocus
    />
</div>

<style>
</style>
