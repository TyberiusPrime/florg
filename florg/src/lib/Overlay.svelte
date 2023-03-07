<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let overlay = false;
  export let listener;

  var listener_overlay = new window.keypress.Listener();
  listener_overlay.simple_combo("esc", () => {
    dispatch("overlay_leave", {});
  });
  listener_overlay.register_combo({
    keys: "h",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
	   console.log(overlay);
      if (overlay == "help") {
        dispatch("overlay_leave", {});
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
