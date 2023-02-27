<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import NavTable from "./NavTable.svelte";
  const dispatch = createEventDispatcher();

  export let levels = [];
  export let title = "";
  export let path = "";
  export let nav_table = [];
  export let mode;

  function goto_level(level) {
    let path = "";
    for (let i = 0; i <= level; i++) {
      path += levels[i][0];
    }
    dispatch("load_node", { path: path });
    console.log("goto level");
    console.log(path);
  }

  function indent(depth) {
    return "&nbsp;".repeat(depth);
  }
  function event_go_sub_node(ev) {
    console.log("event_load_node", ev);
    dispatch("go_sub_node", ev.detail);
  }
</script>

<div class="header">
  Path: '{path}'
  <table>
    {#if levels.length > 0}
      {#each levels as level, index}
        {#if index < levels.length - 1}
          <tr
            ><td
              ><a on:click={(ev) => goto_level(index)}
                >{level[0]}{@html indent(index)} {level[1]}</a
              ></td
            ></tr
          >
        {:else}
          <tr><td>{level[0]}{@html indent(index)} {level[1]}</td></tr>
        {/if}
      {/each}
    {:else}
      <tr><td><a href="#node-">(root node) {title}</a> </td> </tr>
    {/if}
  </table>
  <hr />
  {#if mode == "nav"}
    <NavTable bind:nav_table on:go_sub_node={event_go_sub_node} />
  {/if}
</div>

<style>
  .header {
    /*    position: fixed;
    top: 0;
    width: 100%;
	*/
  }

  td {
    font-family: JetBrains Mono, monospace, monospace;
    text-color: red;
    font-weight: bold;
    text-align: left;
  }
</style>
