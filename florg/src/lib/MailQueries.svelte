<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { toast } from "@zerodevx/svelte-toast";
  import QuickPick from "../lib/QuickPick.svelte";
  import { enter_mode, leave_mode } from "../lib/mode_stack.ts";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { onMount, onDestroy } from "svelte";

  let entries = [];
  async function get_entries() {
    entries = [];
    let searches = await invoke("get_mail_search_folders", {});
    if (searches == null) {
      searches = {
        a: "*",
        m: "tag:inbox and tag:unread",
        M: "tag:inbox",
      };
    }
    for (let key in searches) {
      let query = searches[key];
      entries.push({
        key: key,
        target_path: query,
        text: query,
      });
    }
    console.log(searches);
    entries = entries;
    return entries;
  }

  function handle_action(event) {
	console.log("action", event.detail);
	enter_mode("mail_query", { query: event.detail}, false);
  }
</script>

<div>
  {#await get_entries()}{/await}

  <QuickPick bind:entries on:action={handle_action} />
</div>

<style>
</style>
