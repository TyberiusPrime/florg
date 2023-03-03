<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import * as KeyPress from "../../dist/keypress-2.1.5.min.js";
  import { Fzf, byLengthAsc } from "fzf";

  import PickerTable from "./PickerTable.svelte";

  export let query = "";
  export let elements = [];
  export let downstream_elements = [];
  export let more_mail = false;
  let focused = 0;

  const dispatch = createEventDispatcher();
  let input_text = "";
  let last_text = "---";

  function handle_keyup(ev) {
    console.log(ev);
    const fzf = new Fzf(elements, {
      selector: (item) => item.from + " + " + item.subject,
      tiebreakers: [byLengthAsc],
    });
    if (ev.key == "Escape") {
      dispatch("leave", null);
    } else if (ev.key == "Enter") {
      dispatch("action", {
        cmd: downstream_elements[focused].cmd,
        action: action,
      });
    } else if (ev.key == "r" && ev.ctrlKey) {
      console.log("ctrl-r");
      ev.stopPropagation();
      ev.preventDefault();
	  dispatch("refine_search", null);
    } else if (ev.key == "ArrowDown" || ev.key == "ArrowUp") {
      ev.stopPropagation();
      ev.preventDefault();
    } else {
      if (input_text != last_text) {
        const entries = fzf.find(input_text);
        downstream_elements = entries.map((entry) => entry.item);
        focused = 0;
        refresh_focus();
        last_text = input_text;
      }
    }
  }
  function isElementInViewport(el) {
    // Special bonus for those using jQuery
    if (typeof jQuery === "function" && el instanceof jQuery) {
      el = el[0];
    }

    var rect = el.getBoundingClientRect();
	let header_height = document.getElementById("header").offsetHeight;

    return (
      rect.top >= header_height &&
      rect.left >= 0 &&
      (rect.bottom <=
        (window.innerHeight ||
          document.documentElement.clientHeight) - header_height)
		  &&
      rect.right <=
        (window.innerWidth ||
          document.documentElement.clientWidth) /* or $(window).width() */
    );
  }

  function refresh_focus() {
    let els = document.querySelectorAll("#mail_pick_table tr");
    console.log(els);
    for (let i = 0; i < els.length; i++) {
      els[i].classList.remove("chosen");
      if (i == focused) {
        els[i].classList.add("chosen");
		if (!isElementInViewport(els[i])) {
			els[i].scrollIntoView(true);
			var scrolledY = window.scrollY;
			let header_height = document.getElementById("header").offsetHeight;
			if (scrolledY) {
			  window.scroll(0, scrolledY - header_height);
			}
			}
      }
    }
  }

  function handle_key_down(ev) {
    console.log(ev);
    console.log(downstream_elements.length, focused);
    if (ev.key == "ArrowDown") {
      ev.preventDefault();
      if (focused < downstream_elements.length - 1) {
        focused += 1;
        refresh_focus();
      }
    } else if (ev.key == "ArrowUp") {
      ev.preventDefault();
      if (focused > 0) {
        focused -= 1;
        refresh_focus();
      }
    }
  }
</script>

<div on:keyup={handle_keyup} on:keydown={handle_key_down}>
  Mail search:&nbsp;&nbsp;&nbsp;<input id="query" bind:value={query} /> <br />
  inline filter:
  <input id="typebox" autofocus bind:value={input_text} />
  {#if more_mail}
  <br />(More mail found, refine your search)
  {/if}
</div>

<style>
  input {
    width: 50%;
  }
</style>
