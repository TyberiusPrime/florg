<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import Flatpickr from "svelte-flatpickr";
  import "flatpickr/dist/flatpickr.css";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import * as KeyPress from "../../dist/keypress-2.1.5.min.js";
  import { Fzf } from "fzf";

  export let elements = [];
  export let focused  = 0;
</script>

<div>
  <table id="pick_table">
    {#each elements as el, index}
      {#if index == focused}
        <tr class="chosen" data-command={el.cmd}>
          <td>{el.text}</td>
        </tr>
      {:else}
        <tr data-command={el.cmd}>
          <td>{el.text}</td>
        </tr>
      {/if}
    {/each}
  </table>
</div>

<style>
  table {
    border-collapse: collapse;
    margin: 25px 0;
    font-size: 0.9em;
    font-family: sans-serif;
    min-width: 400px;
    box-shadow: 0 0 20px rgba(0, 0, 0, 0.15);
  }

  thead tr {
    background-color: #009879;
    color: #ffffff;
    text-align: left;
  }
  th,
  td {
    padding: 12px 15px;
  }

  .chosen {
    background-color: #bfbfff;
  }
</style>
