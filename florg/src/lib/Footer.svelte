<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let msg = "";
  export let show_help = false;
  export let mode = "normal";
  export let currently_edited = false;

  </script>

<div id="footer">
  <hr />
  {#if mode=="normal" && show_help}
    <div id="help">
      <table>
        <tr> <td class="hotkey">h</td><td>Show/hide help</td> </tr>
        <tr><td class="hotkey">space</td><td>Nav mode</td> </tr>
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
  mode: {mode}
  {#if currently_edited}
  <span style="color:red">Currently edited</span>
  {/if}

</div>

<style>
</style>
