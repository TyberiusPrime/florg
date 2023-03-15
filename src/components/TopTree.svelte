<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { goto } from "$app/navigation";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  export let data;

  function goto_level(level) {
    let path = "";
    for (let i = 0; i <= level; i++) {
      path += data.levels[i][0];
    }
	goto("/node/" + path);
  }

  function indent(depth) {
    return "&nbsp;".repeat(depth);
  }
</script>

<div>
  <table>
    {#if data.levels.length > 0}
      <tr><td><a on:click={(ev) => goto_level(-1)}>(root node) </a></td> </tr>
      {#each data.levels as level, index}
        {#if index < data.levels.length - 1}
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
      <tr><td>(root node) {data.title} </td> </tr>
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
