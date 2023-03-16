<script lang="ts">
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import {
    no_text_inputs_focused,
    dispatch_keyup,
    focus_first_in_node,
  } from "../lib/util.ts";

  export let overlay = false;

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
    console.log("afterUpdate");
    if (overlay != "") {
      window.setTimeout(
        () => focus_first_in_node(document.getElementById("overlay")),
        10
      );
    }
  });
</script>

<div id="overlay" tabIndex="0">
  <slot />

  {#if overlay != ""}
    {start_listening()}
  {:else}
    {stop_listening()}
  {/if}
</div>

<style>
</style>
