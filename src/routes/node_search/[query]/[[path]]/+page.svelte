<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/tauri";
  import { get_last_path } from "$lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "$lib/../components/Picker.svelte";
  import Help from "$lib/../components/Help.svelte";
  import View from "$lib/../components/View.svelte";
  import { get_node } from "$lib/util.ts";
  import { onMount, onDestroy } from "svelte";
  import { tag_class } from "$lib/colors.ts";

  export let data;
  let viewComponent;
  let overlay;
  let help_entries = [{ key: "Esc", text: "Go back" }];

  async function handle_action(ev) {
    goto("/tree/" + ev.detail.cmd, { replaceState: true });
  }

  function handle_keys(ev) {}

  let search_results = [];
</script>

<View bind:this={viewComponent} bind:overlay>
  <div slot="header" class="header">
    <h1>Node search</h1>
    {data.search_term}
  </div>

  <div slot="content" on:keyup={handle_keys}>
  {#if data.search_results.length > 0}
    <Picker on:action={handle_action}>
      <div slot="message"><h1>Node search</h1></div>
      <svelte:fragment slot="entries">
        {#each data.search_results as result}
          <tr data-cmd={result.cmd}
            ><td
              >{@html result.text}
              {#each result.tags as tag}
                <div class="tags {tag_class(tag.slice(1))}">{tag}</div>
              {/each}
            </td></tr
          >
        {/each}
      </svelte:fragment>
    </Picker>
	{:else}
	No hits.
	{/if}
  </div>
  <svelte:fragment slot="overlays">
    {#if overlay == "help"}
      <Help bind:entries={help_entries} />
    {/if}
  </svelte:fragment>
</View>
