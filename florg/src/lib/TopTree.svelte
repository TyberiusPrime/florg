<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let levels = [];
  export let title = "";
  export let path = "";

  function goto_level(level) {
    let path = "";
    for (let i = 0; i <= level; i++) {
      path += levels[i][0];
    }
    dispatch("load_node", { path: path });
    console.log("goto level");
    console.log(path);
  }
</script>

<div class="header">
  Path: '{path}'
  <table>
    {#if levels.length > 0}
      {#each levels as level, index}
        <tr
          ><td
            ><a on:click={(ev) => goto_level(index)}>{level[0]} {level[1]}</a
            ></td
          ></tr
        >
      {/each}
    {:else}
      <tr><td><a href="#node-">(root node) {title}</a> </td> </tr>
    {/if}
  </table>
  <hr />
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
