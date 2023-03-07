<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { Fzf, byLengthAsc } from "fzf";
  import { onMount, onDestroy } from "svelte";

  import PickerTable from "./PickerTable.svelte";

  export let message = "";
  export let action = "";
  export let elements = [];

  let focused = 0;
  let input_text = "";
  let last_text = "---";
  let downstream_elements = [];

  function handle_text_change(ev) {
    console.log(ev);
    const fzf = new Fzf(elements, {
      selector: (item) => item.text,
      tiebreakers: [byLengthAsc],
    });
    if (ev.key == "Escape") {
      dispatch("picker_canceled", null);
    } else if (ev.key == "Enter") {
      dispatch("picker_accepted", {
        cmd: downstream_elements[focused].cmd,
        action: action,
      });
	  }
    else if (ev.key == "ArrowDown") {
		ev.preventDefault();
    } 
    else if (ev.key == "ArrowUp") {
		ev.preventDefault();
    } 
	else {
      if (input_text != last_text) {
        const entries = fzf.find(input_text);
        downstream_elements = entries.map((entry) => entry.item);
        focused = 0;
        last_text = input_text;
      }
    }
  }

  function handle_key_down(ev) {
    if (ev.key == "ArrowDown") {
      ev.preventDefault();
      if (focused < downstream_elements.length - 1) {
        focused += 1;
		console.log("focused now", focused);
      }
    } else if (ev.key == "ArrowUp") {
      ev.preventDefault();
      if (focused > 0) {
        focused -= 1;
      }
    }
  }
  downstream_elements = elements;

</script>

<div on:keyup={handle_text_change} on:keydown={handle_key_down}>
  {@html message} <br />
  filter: <input id="typebox" autofocus bind:value={input_text} />
  <PickerTable bind:focused bind:elements={downstream_elements} />
</div>

<style>
</style>
