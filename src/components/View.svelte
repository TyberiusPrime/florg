<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import {
    focus_first_in_node,
    find_first_focusable,
    no_text_inputs_focused,
  } from "../lib/util.ts";

  export let overlay = "";

  function toggleElementAndChildren(element, isDisabled) {
    // Add wrapper functions to event handlers the first time this function is called on this element

    // Set disabled/enabled state of the element
    element.disabled = isDisabled;

    // Unset/re-set the tabIndex attribute on the element
    if (isDisabled) {
      //element.orgiginalTabIndex = element.tabIndex;
      //element.removeAttribute("tabIndex");
      element.classList.add("disabled");
    } else {
      //element.tabIndex = element.originalTabIndex;
      element.classList.remove("disabled");
    }

    // Disable/enable all children of the element
    var childNodes = element.childNodes;
    for (var i = 0; i < childNodes.length; i++) {
      var child = childNodes[i];
      if (child.nodeType === Node.ELEMENT_NODE) {
        toggleElementAndChildren(child, isDisabled);
      }
    }
  }

  export function enter_overlay(ov) {
    document.getElementById("footer").tabIndex = 0;
    overlay = ov;
    toggleElementAndChildren(document.getElementById("main_content"), true);
    window.setTimeout(() => {
      focus_first_in_node(document.getElementById("footer"));
    }, 10);
  }

  export function leave_overlay() {
    document.getElementById("footer").removeAttribute("tabIndex");
    overlay = "";
    toggleElementAndChildren(document.getElementById("main_content"), false);
    window.setTimeout(() => {
      focus_first_in_content();
    }, 10);
  }

  function focus_first_in_content() {
    let wrapper = document.getElementById("main_content");
    if (wrapper != null) {
      focus_first_in_node(wrapper);
    }
  }

  afterUpdate(() => {
	/*
    if (no_text_inputs_focused()) {
      window.setTimeout(focus_first_in_content, 10);
    }
	*/
	
  });

  function focus(ev) {
  /*
    let sel = window.getSelection();
    if (sel.isCollapsed && no_text_inputs_focused()) {
      console.log("was collapsed");
      let wrapper = document.getElementById("wrapper");
      let f = find_first_focusable(wrapper);
      if (f != document.activeElement) {
	  toast.push('focus');
        f.focus();
      }
    }
	*/
  }
  function show_help() {
    enter_overlay("help");
  }

  function handle_keyup(ev) {
    console.log("footer keyup");
    if (ev.key == "Escape") {
      leave_overlay();
      ev.preventDefault();
      ev.stopPropagation();
    }
  }
</script>

<div id="wrapper" class="wrapper" on:keyup on:click={focus} tabindex="-1">
  <div class="header" id="header">
    <slot name="header" />
  </div>
  <div class="main_content" id="main_content">
    <div class="sticky-spacer" />
    <div class="sticky-content" id="sticky-content">
      <slot name="content" />
    </div>
  </div>
  <div class="footer" id="footer" on:keyup={handle_keyup}>
    <slot name="footer">
      {#if overlay != ""}
        <slot name="overlays" />
      {:else}
        <div on:click={show_help}>
          <slot name="no_overlay">
            Press <span class="hotkey">h</span> for help.
          </slot>
        </div>
      {/if}
    </slot>
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
