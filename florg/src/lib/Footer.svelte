<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let msg = "";
  export let show_help = false;
  export let mode = "normal";
  export let overlay_mode = "";
  export let currently_edited = false;

  </script>

<div id="footer">
  <div>
    {#if msg != ""}
      {@html msg}
    {/if}
	{#if overlay_mode == ""}
	press <span class="hotkey">h</span> for help
	{/if}
  </div>
  mode: {mode}. Overlay: {overlay_mode}
  {#if currently_edited}
  <span style="color:red">Currently edited</span>
  {/if}

</div>

<style>
</style>
