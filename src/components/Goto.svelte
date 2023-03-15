<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { get_node } from "../lib/util.ts";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import { onMount, onDestroy } from "svelte";
  import { goto_node } from "$lib/../components/Goto.ts";
  const dispatch = createEventDispatcher();

  export let action = null;
  export let overlay;

  async function get_entries() {
    let entries = [];
    let nav = await invoke("get_nav", {});
    for (let key in nav) {
      let target_path = nav[key];
      let query_path = target_path;
      if (query_path.startsWith("#") || query_path.startsWith("!")) {
        query_path = query_path.slice(1);
      }
      let node = await get_node(query_path);
      let text = target_path + " ";
      if (node.node != null) {
        text += node.node.header.title;
      } else {
        text += " (empty node)";
      }
      entries.push({
        key: key,
        target_path: target_path,
        text: text,
      });
    }
    return entries;
  }
  let entries = [];

  onMount(async () => {
    entries = await get_entries();
  });

  function handle_action(event) {
    console.log("action", event.detail, action);
    let path = event.detail;
    if (action != null) {
      action(path);
    } else {
	  overlay = "";
      goto_node(path);

    }
  }
</script>

<div>
  <QuickPick bind:entries on:action={handle_action} />
</div>
