<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let search_term = "";

  async function handle(event) {
    if (event.key == "Enter") {
	  event.stopPropagation();
      dispatch("leave", true);
    } else if (event.key == "Escape") {
	  event.stopPropagation();
      dispatch("leave", false);
    }
  }
</script>

<div>
    <input
      id="search_input-input"
      bind:value={search_term}
      placeholder="search text"
      on:keyup={handle}
      autofocus
    />
</div>

<style>
</style>
