<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { toast } from "@zerodevx/svelte-toast";
  import QuickPick from "../lib/QuickPick.svelte";
  import InputWithHistory from "../lib/InputWithHistory.svelte";
  import { enter_mode, leave_mode, get_last_path } from "../lib/mode_stack.ts";
  import { invoke } from "@tauri-apps/api/tauri";
  import Select from "svelte-select";
  const dispatch = createEventDispatcher();

  export let mode;
  export let overlay;
  export let in_page_search_term = "";
  let search_term;

  export let search_entries = [
    { key: "s", target_path: "node", text: "Node search (below this node)" },
    { key: "r", target_path: "root", text: "Node search (all nodes)" },
    { key: "m", target_path: "mail", text: "Mail search" },
    { key: "p", target_path: "in_page", text: "in page" },
  ];

  let search_mode = "pick";

  function handle_search_action(event) {
    console.log("search action", event.detail);
    let action = event.detail;
    search_mode = action;
  }

  async function handle_accepted(event) {
    await do_search();
  }

  async function do_search() {
    let history = await invoke("history_get", { name: "search" + search_mode });
	history = history.filter((e) => e != search_term);
    history.push(search_term);
    //limit history to last 10 entries
    history = history.slice(-10);
    if (
      !(await invoke("history_store", {
        name: "search" + search_mode,
        entries: history,
      }))
    ) {
      toast.push("Could not store search history");
    }
    if (search_mode == "in_page") {
      window.find(search_term, false, false, true, false);
      in_page_search_term = search_term;
    } else if (search_mode == "node" || search_mode == "root") {
      let path = null;
      if (search_mode == "root") {
        path = "";
      } else {
        path = get_last_path();
      }
      console.log("path", path);
      enter_mode("node_search", { path: path, search_term: search_term });
    } else {
      toast.push("Unknonw search_mode: " + search_mode);
    }
    overlay = "";
  }

  let items = [];
  async function get_history() {
    let history = await invoke("history_get", { name: "search" + search_mode });
    items = [];
    for (let entry of history) {
      items.push(entry);
    }
    return items;
  }

  function handle_leave() {
    dispatch("leave", {});
  }
</script>

<div>
  {#if search_mode == "pick"}
    <QuickPick
      bind:entries={search_entries}
      on:action={handle_search_action}
      on:leave
    />
  {:else}
    <!--
    <input
      id="search_input-input"
      bind:value={search_term}
      placeholder="search text"
      on:keyup={handle_keypress}
      autofocus
    />
	-->
    {#await get_history() then history}
      <InputWithHistory
        bind:history={items}
        on:leave={handle_leave}
        on:accept={handle_accepted}
        bind:input_value={search_term}
      />
    {/await}

    <button on:click={do_search}>Search</button>
    <br />

    {#if search_mode == "node"}
      Node search.
    {:else if search_mode == "mail"}
      Mail search, notmuch search query.
    {:else if search_mode == "in_page"}
      In page search mode.
    {/if}
    <span class="hotkey">Enter</span> to accept, <span class="hotkey">Esc</span>
  {/if}
</div>
