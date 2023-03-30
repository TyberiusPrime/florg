<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { toast } from "@zerodevx/svelte-toast";
  import QuickPick from "$lib/../components/QuickPick.svelte";
  import InputWithHistory from "$lib/../components/InputWithHistory.svelte";
  import { get_last_path } from "$lib/../lib/mode_stack.ts";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/tauri";
  import Select from "svelte-select";
  const dispatch = createEventDispatcher();

  export let path = "";
  export let overlay;
  export let in_page_search_term = "";
  export let default_search_term = "";
  let search_term = default_search_term;

  export let search_entries = [
    { key: "s", target_path: "node", text: "Node search (below last node)" },
    { key: "r", target_path: "root", text: "Node search (all nodes)" },
    { key: "m", target_path: "mail", text: "Mail search" },
    { key: "p", target_path: "in_page", text: "in page" },
  ];

  export let search_mode = "pick";

  function handle_search_action(event) {
    console.log("search action", event.detail);
    let action = event.detail;
    search_mode = action;
  }

  async function handle_accepted(event) {
    await do_search();
  }

  async function do_search() {
    if (search_term == "") {
      return;
    }
    let history_mode = search_mode;
    if (search_mode == "root") {
      history_mode = "node";
    }
    console.log(history_mode);
    let history = await invoke("history_get", {
      name: "search" + history_mode,
    });
    history = history.filter((e) => e != search_term);
    history.push(search_term);
    //limit history to last 10 entries
    history = history.slice(-10);
    if (
      !(await invoke("history_store", {
        name: "search" + history_mode,
        entries: history,
      }))
    ) {
      toast.push("Could not store search history");
    }
    if (search_mode == "in_page") {
      in_page_search_term = search_term;
      window.find(in_page_search_term, false, false, true, false);
    } else if (search_mode == "node" || search_mode == "root") {
      let path = null;
      if (search_mode == "root") {
        path = "";
      } else {
        path = get_last_path();
      }
      console.log("path", path);
      goto("/node_search/" + encodeURIComponent(search_term) + "/" + path);
    } else if (search_mode == "mail") {
      goto("/mail/query/" + encodeURIComponent(search_term));
    } else {
      toast.push("Unknown search_mode: " + search_mode);
    }
    dispatch("leave", {});
  }

  let items = [];
  async function get_history() {
    let history_mode = search_mode;
    if (search_mode == "root") {
      history_mode = "node";
    }

    let history = await invoke("history_get", {
      name: "search" + history_mode,
    });
    items = [];
    for (let entry of history) {
      items.push(entry);
    }
    return items;
  }

  function handle_leave() {
    dispatch("leave", {});
    search_mode = "pick";
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
