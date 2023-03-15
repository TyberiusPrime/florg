<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  import {keypress} from "keypress.js";
  const dispatch = createEventDispatcher();
  import { no_text_inputs_focused } from "../lib/util.ts";
  import { overlay_handles_escape } from "../lib/mode_stack.ts";

  export let overlay = false;
  export let listener;

  let handle_esc = true;

  overlay_handles_escape.subscribe((value) => {
    handle_esc = value;
  });

  var listener_overlay = new keypress.Listener();
  listener_overlay.simple_combo("esc", () => {
    if (handle_esc) {
      dispatch("leave", {});
    }
  });
  listener_overlay.register_combo({
    keys: "h",
    is_unordered: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      console.log("overlay h");

      if (no_text_inputs_focused) {
        if (overlay == "help") {
		console.log("hide help");
          dispatch("leave", {});
          e.preventDefault();
        }
      }
    },
  });

  onMount(async () => {});

  function start_listening() {
    console.log("start listening");
    listener.stop_listening();
    listener_overlay.listen();
    return "";
  }

  function stop_listening() {
  console.log("stop overaly listen");
    listener.listen();
    listener_overlay.stop_listening();
    return "";
  }

  onDestroy(async () => {});
</script>

<div>
  <slot />

  {#if overlay != ""}
    {start_listening()}
  {:else}
    {stop_listening()}
  {/if}
</div>

<style>
</style>
