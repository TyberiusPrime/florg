<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let levels = [];
  export let title = "";
  export let path = "";
  export let mode;

  function goto_level(level) {
    let path = "";
    for (let i = 0; i <= level; i++) {
      path += levels[i][0];
    }
    dispatch("goto_node", { path: path, 
	normal_mode: mode != "nav" });
    console.log("goto level");
    console.log(path);
  }

  function indent(depth) {
    return "&nbsp;".repeat(depth);
  }
</script>

<div>
  Path: '{path}'
  <table>
    {#if levels.length > 0}
	  <tr><td><a on:click={(ev) => goto_level(-1)}>(root node) {title} </a></td> </tr>
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
      <tr><td>(root node) {title} </td> </tr>
    {/if}
  </table>
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
