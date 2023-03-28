<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  const dispatch = createEventDispatcher();

  export let action = "";
  export let text = "";
  export let entries = "";

  function handle_key_up(ev) {
    console.log("quick pick key up", ev.key);
    for (let entry of entries) {
      if (entry.key == ev.key) {
        console.log("quick pick hit");
        ev.stopPropagation();
        ev.preventDefault();
        dispatch("action", entry.target_path);
      }
    }
  }

  onMount(async () => {
    console.log("navtree mount");
  });

  onDestroy(() => {
    console.log("navtree destroy");
  });
</script>

<div on:keyup={handle_key_up} tabIndex=0>
  {text}
  {#each entries as entry}
    <div>
      <span class="hotkey">{entry.key}</span>
      {entry.text}
    </div>
  {/each}
</div>
