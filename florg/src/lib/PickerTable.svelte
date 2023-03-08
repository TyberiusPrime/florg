<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import Flatpickr from "svelte-flatpickr";
  import "flatpickr/dist/flatpickr.css";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { Fzf } from "fzf";

  export let elements = [];
  export let focused  = 0;
</script>

<div>
  <table id="pick_table">
    {#each elements as el, index}
      {#if index == focused}
        <tr class="chosen" data-command={el.cmd}>
          <td>{@html el.text}</td>
        </tr>
      {:else}
        <tr data-command={el.cmd}>
          <td>{@html el.text}</td>
        </tr>
      {/if}
    {/each}
  </table>
</div>

<style>
td {
    padding: 12px 15px;
  }
 </style>
