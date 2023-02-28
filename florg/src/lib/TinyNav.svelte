<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import SvelteTooltip from "svelte-tooltip";
  const dispatch = createEventDispatcher();

  export let nodes = "";
  export let current_path;

  function clicked_node(path) {
    dispatch("goto_node", { path: current_path + path });
  }
</script>

<div id="wrapper">
  Children:
  {#if nodes.length > 0}
  {#each nodes as node, index}
    {#if index > 0}
      &nbsp;{/if}

    <SvelteTooltip tip={node.text} right color="#DFDFDF;border:1px dashed grey;">
      <a on:click={clicked_node(node.key)} id="node-{index}">{node.key}</a>
    </SvelteTooltip>
  {/each}
  {:else}
  (none)
  {/if}
</div>

<style>
</style>
