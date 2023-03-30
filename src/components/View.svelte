<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import {
    focus_first_in_node,
    find_first_focusable,
    no_text_inputs_focused,
  } from "../lib/util.ts";

  export let overlay = "";
  let last_focused = null;

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
    last_focused = document.activeElement;
    overlay = ov;
    toggleElementAndChildren(document.getElementById("main_content"), true);
    window.setTimeout(() => {
      focus_first_in_node(document.getElementById("overlay"));
    }, 10);
  }

  export function leave_overlay() {
    overlay = "";
    toggleElementAndChildren(document.getElementById("main_content"), false);
    window.setTimeout(() => {
      last_focused.focus({ preventScroll: true });
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

  function handle_window_resize() {
  const header = document.querySelector("#header");
  console.log(header);
    if (header != null) {
      const headerHeight = header.offsetHeight;
      document.getElementById("main_content").style.marginTop = headerHeight + "px";
    }
    return "";
  }

  onMount(() => {
	handle_window_resize();
	document.getElementById("main_content").scrollIntoView();
  });
</script>

<svelte:window on:resize={handle_window_resize} />

<div id="wrapper" class="wrapper" on:keyup on:click={focus} tabindex="-1">
  {#if overlay != ""}
    <div class="overlay" id="overlay" on:keyup={handle_keyup} tabindex=0>
      <slot name="overlays" />
    </div>
  {/if}
  <div class="header" id="header">
    <div on:click={show_help}>
      <slot name="no_overlay">
        Press <span class="hotkey">h</span> for help.
      </slot>
    </div>
    <slot name="header" />
  </div>

  <div class="main_content" id="main_content">
    <slot name="content" />
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
