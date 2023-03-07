<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import QuickPick from "../lib/QuickPick.svelte";
  const dispatch = createEventDispatcher();

  export let mode;
  export let overlay;
  export let in_page_search_term ="";
  let search_term;

  export let search_entries = [
    { key: "g", target_path: "node", text: "Node search" },
    { key: "m", target_path: "mail", text: "Mail search" },
    { key: "p", target_path: "in_page", text: "in page" },
  ];

  let search_mode = "pick";

  function handle_search_action(event) {
    console.log("search action", event.detail);
    let action = event.detail;
    search_mode = action;
  }

  function handle_keypress(event) {
    console.log("keypress", event.key);
    if (event.key == "Enter") {
      do_search();
    }
  }

  function do_search() {
    if (search_mode == "in_page") {
      window.find(search_term, false, false, true, false);
	  in_page_search_term = search_term;
    }
	overlay = "";
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
    <input
      id="search_input-input"
      bind:value={search_term}
      placeholder="search text"
      on:keyup={handle_keypress}
      autofocus
    />
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
