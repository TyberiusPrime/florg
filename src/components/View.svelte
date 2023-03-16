<script lang="ts">
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { focus_first_in_node, find_first_focusable, no_text_inputs_focused } from "../lib/util.ts";

  function focus_first_in_content() {
    let wrapper = document.getElementById("content");
    focus_first_in_node(wrapper);
  }

  afterUpdate(() => {
    console.log("afterUpdate");
    window.setTimeout(focus_first_in_content, 10);
  });

  function focus(ev) {
    let sel = window.getSelection();
    if (sel.isCollapsed && no_text_inputs_focused()) {
	console.log("was collapsed");
      let wrapper = document.getElementById("wrapper");
      let f = find_first_focusable(wrapper);
      if (f != document.activeElement) {
        f.focus();
      }
    }
  }
</script>

<div id="wrapper" class="wrapper" on:keyup on:click={focus} tabindex="-1">
  <div class="header" id="header">
    <slot name="header" />
  </div>
  <div class="main_content">
    <div class="sticky-spacer" />
    <div class="sticky-content" id="sticky-content">
      <slot name="content" />
    </div>
  </div>
  <div class="footer" id="footer">
    <slot name="footer" />
  </div>
</div>

<style>
  /*  :global body,
  :global .footer,
  :global .header {
    background-color: #eeeeee;
    border-bottom: 2px dashed grey;
  }
  */
</style>
