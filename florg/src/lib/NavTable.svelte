<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();


  export let nav_table = [];

  function chunk_nav(nav_table) {
    let rows = [];
    let cols = 5;
    let entries = 5;
    for (let entry = 0; entry < entries; entry += 1) {
      rows.push([]);
    }
    for (let column = 0; column < cols; column += 1) {
      for (let entry = 0; entry < entries; entry += 1) {
        let index = column * entries + entry;
        if (index < nav_table.length) {
          rows[entry].push(nav_table[index]);
        }
      }
    }
    return rows;
  }

  function go_sub_node(key) {
	  dispatch('go_sub_node', key);
  }

</script>

<div id="navtable">
  {#if nav_table.length > 0}
  Navigation mode:
    <table>
      {#each chunk_nav(nav_table) as row}
        <tr>
          {#each row as entry}
            <td
			  ><a on:click="{(ev) => go_sub_node(entry.key)}"><span class="hotkey">{entry.key}</span>&nbsp;{entry.text}</a
              ></td
            >
          {/each}
        </tr>
      {/each}
      {#if nav_table.length > 25}
        <tr>
          <td colspan="2"><a on:click="{(ev) => go_sub_node('z')}">><span class="hotkey">Z</span> More...</a></td>
        </tr>
      {/if}
    </table>
  {:else}
    (No further children. a-z to create)
  {/if}
  <hr />
</div>

<style>
  #navtable {
    width: 100%;
  }

  #navtable table {
    width: 100%;
  }

  div {
    text-align: left;
  }
</style>
