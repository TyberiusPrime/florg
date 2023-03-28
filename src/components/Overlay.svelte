<script lang="ts">
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { toast } from "@zerodevx/svelte-toast";
  import {
    no_text_inputs_focused,
    dispatch_keyup,
    focus_first_in_node,
  } from "$lib/util.ts";

  export let overlay = false;
  let last_overlay = "";

  onMount(async () => {});

  function start_listening() {
    console.log("start listening");
    return "";
  }

  function stop_listening() {
    console.log("stop overaly listen");
    return "";
  }

  onDestroy(async () => {});

  afterUpdate(() => {
    if (overlay != last_overlay) {
      last_overlay = overlay;
      window.setTimeout(
        () => focus_first_in_node(document.getElementById("overlay")),
        10
      );
    }
  });

  function onblur() {
	focus_first_in_node(document.getElementById("overlay"));
	toast.push("onblur");
  }

  onMount(() => {
	focus_first_in_node(document.getElementById("overlay"));
  });

  function handle_keyup(ev) {
    console.log("overlay keyup");
    if (ev.key == "Escape") {
      dispatch("leave");
      ev.preventDefault();
      ev.stopPropagation();
    }
  }
</script>

<div id="overlay" on:keyup={handle_keyup}>
  <slot />

  {#if overlay != ""}
    {start_listening()}
  {:else}
    {stop_listening()}
  {/if}
</div>

<style>
</style>
