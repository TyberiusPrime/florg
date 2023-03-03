<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import * as KeyPress from "../../dist/keypress-2.1.5.min.js";
  import { Fzf, byLengthAsc } from "fzf";

  import PickerTable from "./PickerTable.svelte";

  export let downstream_elements = [];
  export let focused = 0;

  const dispatch = createEventDispatcher();

</script>

<div >
 <div style="overflow:scroll">
  <table id="mail_pick_table">
  {#if downstream_elements.length == 0}
nothing found.
  {/if}
    {#each downstream_elements as el, index}
      <tr class={index == focused ? "chosen" : ""}>
	  <td>{index}</td>
        <td>
		{#if el.unread}
			<b>
				{el.authors}<br />
				({el.messages.length}) {el.subject}
				</b>
		{:else}
				{el.authors}<br />
				({el.messages.length}) {el.subject}
		{/if}

</td>

      </tr>
    {/each}
  </table>
  </div>
</div>

<style>
  input {
    width: 50%;
  }
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
