<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import * as KeyPress from "../js/keypress-2.1.5.min.js";
  import { Fzf, byLengthAsc } from "fzf";

  import PickerTable from "./PickerTable.svelte";

  export let query = "";
  export let elements = [];
  export let downstream_elements = [];
  export let more_mail = false;
  export let focused = 0;
  export let view_mode = "threads";

  const dispatch = createEventDispatcher();
  let input_text = "";
  let last_text = "---";

  function handle_keyup(ev) {
    const fzf = new Fzf(elements, {
      selector: (item) => item.from + " + " + item.subject + " " + item.authors,
      tiebreakers: [byLengthAsc],
    });
    if (ev.key == "Escape") {
      dispatch("leave", null);
    } else if (ev.key == "Enter") {
      if (view_mode == "threads") {
        if (downstream_elements[focused].messages.length == 1) {
          dispatch("action", {
            id: downstream_elements[focused].messages[0].id,
            single_message: true,
          });
        } else {
          dispatch("action", {
            id: downstream_elements[focused].id,
            single_message: false,
          });
        }
      } else if (view_mode == "messages") {
        dispatch("action", {
          id: downstream_elements[focused].id,
          single_message: true,
        });
      }
    } else if (ev.key == "r" && ev.ctrlKey) {
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
    let footer_height = document.getElementById("footer").offsetHeight;

    return (
      rect.top >= header_height &&
      rect.left >= 0 &&
      rect.bottom <=
        (window.innerHeight || document.documentElement.clientHeight) -
          footer_height &&
      rect.right <=
        (window.innerWidth ||
          document.documentElement.clientWidth) /* or $(window).width() */
    );
  }

  function refresh_focus() {
    let els = document.querySelectorAll("#mail_pick_table tr");
    for (let i = 0; i < els.length; i++) {
      els[i].classList.remove("chosen");
      if (i == focused) {
        els[i].classList.add("chosen");
        if (!isElementInViewport(els[i])) {
          els[i].scrollIntoView({ behavior: "smooth", block: "center" });
          /*var scrolledY = window.scrollY;
			let header_height = document.getElementById("header").offsetHeight;
			if (scrolledY - header_height > 0) {
			  window.scroll(0, scrolledY - header_height);
			}
			*/
        }
      }
    }
  }

  function handle_key_down(ev) {
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
    } else if (ev.key == "PageDown") {
      ev.preventDefault();
      if (focused < downstream_elements.length - 1) {
        focused += 10;
        if (focused > downstream_elements.length - 1) {
          focused = downstream_elements.length - 1;
        }
        refresh_focus();
      }
    } else if (ev.key == "PageUp") {
      ev.preventDefault();
      if (focused > 0) {
        focused -= 10;
        if (focused < 0) {
          focused = 0;
        }
        refresh_focus();
      }
    } else if (ev.key == "Home") {
      ev.preventDefault();
      focused = 0;
      refresh_focus();
    } else if (ev.key == "End") {
      ev.preventDefault();
      focused = downstream_elements.length - 1;
      refresh_focus();
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
  {downstream_elements.length}
</div>

<style>
  input {
    width: 50%;
  }
</style>
