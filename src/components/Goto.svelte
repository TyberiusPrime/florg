<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { get_node } from "../lib/util.ts";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import { onMount, onDestroy } from "svelte";
  import { goto_node } from "$lib/../components/Goto.ts";
  import { toast } from "@zerodevx/svelte-toast";
  const dispatch = createEventDispatcher();

  export let action = null;
  export let overlay;
  export let filter = (x) => true;

  async function get_entries() {
    let entries = [];
    let nav = await invoke("get_nav", {});

    if (nav == null) {
      toast.push(
        "Could not read [nav] from settings. Please check your settings.toml"
      );
      overlay = "";
      return;
    }

    const ordered = Object.keys(nav)
      .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()))
      .reduce((obj, key) => {
        obj[key] = nav[key];
        return obj;
      }, {});
    for (let key in ordered) {
      let target_path = nav[key];
      if (!filter(target_path)) {
        continue;
      }
      let query_path = target_path;
      if (query_path.startsWith("#") || query_path.startsWith("!")) {
        query_path = query_path.slice(1);
      }
      let text = target_path;
      if (
        query_path.startsWith("date:") ||
        query_path.startsWith("today:") ||
        query_path.indexOf(":") == -1
      ) {
        let qp = query_path;
        if (query_path.indexOf(":") != -1) {
          qp = query_path.split(":")[1];
        }
        let node = await get_node(qp);
        if (node.node != null) {
          text += " " + node.node.header.title;
        } else {
          text += " (empty node)";
        }
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
    //console.log("action", event.detail, action);
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
