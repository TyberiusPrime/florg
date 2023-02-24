<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import NavTable from "./NavTable.svelte";

  export let msg = "";
  export let show_help = false;
  export let mode = "normal";
  export let nav_table = [];
</script>

<div id="footer">
{show_help}
  {#if mode=="nav"}
    <NavTable bind:navtable={nav_table} />
  {/if}
  <hr />
  {#if mode=="normal" && show_help}
    <div id="help">
      <table>
        <tr> <td class="hotkey">h</td><td>Show/hide help</td> </tr>
        <tr><td class="hotkey">f</td><td>Nav mode</td> </tr>
        <tr><td class="hotkey">/</td><td>Search</td> </tr>
      </table>
    </div>
  {/if}
  <div>
    {#if msg != ""}
      {@html msg}
    {:else if !show_help}
      press <span class="hotkey">h</span> for help
    {/if}
  </div>
</div>

<style>
</style>
