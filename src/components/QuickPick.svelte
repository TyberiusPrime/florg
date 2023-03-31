<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  const dispatch = createEventDispatcher();
  import { toast } from "@zerodevx/svelte-toast";

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

  function fake_key_up(key) {
    toast.push("clicked" + key);
    let ctrlKey = false;
    if (key == "space") {
      key = " ";
    }
    if (key.startsWith("ctrl+")) {
      ctrlKey = true;
      key = key.slice(5);
    }
    handle_key_up({
      key: key,
      ctrlKey: ctrlKey,
      stopPropagation: () => {},
      preventDefault: () => {},
    });
  }
</script>

<div on:keyup={handle_key_up} tabIndex="0">
  {text}
  {#each entries as entry}
    <div class="qp_entry" on:click={fake_key_up(entry.key)}>
      <span class="hotkey">{entry.key}</span>
      {entry.text}
    </div>
  {/each}
</div>

<style>
  .qp_entry {
    font-family: monospace;
    padding-top: 0.25em;
    padding-bottom: 0.25em;
  }

  .qp_entry:nth-child(even) {
    background-color: #ddd;
  }
</style>
