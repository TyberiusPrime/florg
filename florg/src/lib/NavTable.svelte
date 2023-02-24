<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";

  export let nav_table = [
    { key: "A", text: "Hello", hover: "Shu" },
    { key: "B", text: "World", hover: "world hover is a long title" },
    { key: "C", text: "Hello", hover: "Shu" },
    { key: "D", text: "World", hover: "world hover is a long title" },
    { key: "E", text: "Hello", hover: "Shu" },
    { key: "F", text: "World", hover: "world hover is a long title" },
    { key: "G", text: "Hello", hover: "Shu" },
    { key: "H", text: "World", hover: "world hover is a long title" },
    { key: "I", text: "World", hover: "world hover is a long title" },
    { key: "J", text: "Hello", hover: "Shu" },
    { key: "K", text: "World", hover: "world hover is a long title" },
    { key: "L", text: "Hello", hover: "Shu" },
    { key: "M", text: "World", hover: "world hover is a long title" },
    { key: "N", text: "Hello", hover: "Shu" },
    { key: "O", text: "World", hover: "world hover is a long title" },
    { key: "P", text: "Hello", hover: "Shu" },
    { key: "Q", text: "World", hover: "world hover is a long title" },
    { key: "R", text: "Hello", hover: "Shu" },
    { key: "S", text: "World", hover: "world hover is a long title" },
    { key: "T", text: "Hello", hover: "Shu" },
    { key: "U", text: "World", hover: "world hover is a long title" },
    { key: "V", text: "Hello", hover: "Shu" },
    { key: "W", text: "World", hover: "world hover is a long title" },
    { key: "X", text: "Hello", hover: "Shu" },
    { key: "Y", text: "World", hover: "world hover is a long title" },
    { key: "Z", text: "World", hover: "world hover is a long title" },
  ];

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
</script>

<div id="navtable">
  <hr />
  <table>
    {#each chunk_nav(nav_table) as row}
      <tr>
        {#each row as entry}
          <td
            ><a><span class="hotkey">{entry.key}</span>&nbsp;{entry.text}</a
            ></td
          >
        {/each}
      </tr>
    {/each}
    {#if nav_table.length > 25}
      <tr>
        <td colspan="2"><a><span class="hotkey">Z</span> More...</a></td>
      </tr>
    {/if}
  </table>
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
