<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher, tick } from "svelte";
  const dispatch = createEventDispatcher();
  import { Fzf, byLengthAsc } from "fzf";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { no_text_inputs_focused, focus_first_in_node } from "../lib/util.ts";

  import View from "./View.svelte";
  import Help from "./Help.svelte";
  import { isElementInViewport } from "../lib/util.ts";

  export let elements = [];
  export let focused;
  export let enable_filter = true;

  let input_text = "";
  let last_text = "";
  let test_text = "shu";
  let downstream_elements = [];
  let help_entries = [{ key: "Esc", text: "Go back" }];

  export let advance = () => {
    if (focused < downstream_elements.length - 1) {
      focused += 1;
      update_chosen();
    }
  };

  function handle_key_down(ev) {
    if (
      ev.key == "ArrowUp" ||
      ev.key == "ArrowDown" ||
      ev.key == "Home" ||
      ev.key == "End" ||
      ev.key == "PageUp" ||
      ev.key == "PageDown"
    ) {
      ev.preventDefault();
    }
  }
  function handle_text_change(ev) {
    const fzf = new Fzf(elements, {
      selector: (item) => item.innerText,
      tiebreakers: [byLengthAsc],
      fuzzy: "v2",
    });
    if (input_text != last_text) {
      const entries = fzf.find(input_text);
      downstream_elements = entries.map((entry) => entry.item);
      let pt = document.getElementById("pick_table");
      pt.innerHTML = "";
      for (let child of elements) {
        if (downstream_elements.includes(child)) {
          pt.appendChild(child);
        }
      }
      focused = 0;
      update_chosen();
      last_text = input_text;
      window.history.replaceState(
        { ...history.state, input_text: input_text },
        undefined
      );
    }
    if (ev != null) {
      console.log("picker", ev);
      if (ev.key != "Escape") {
        if (ev.key == "ArrowDown") {
          ev.stopPropagation();
          ev.preventDefault();
          advance();
        } else if (ev.key == "ArrowUp") {
          ev.stopPropagation();
          ev.preventDefault();
          if (focused > 0) {
            focused -= 1;
            update_chosen();
          }
        } else if (ev.key == "PageUp") {
          focused = Math.max(0, focused - 10);
          update_chosen();
          ev.preventDefault();
          ev.stopPropagation();
        } else if (ev.key == "PageDown") {
          focused = Math.min(downstream_elements.length - 1, focused + 10);
          update_chosen();
          ev.preventDefault();
          ev.stopPropagation();
        } else if (ev.key == "Home") {
          focused = 0;
          update_chosen();
          ev.preventDefault();
          ev.stopPropagation();
        } else if (ev.key == "End") {
          console.log("end", no_text_inputs_focused());
          focused = downstream_elements.length - 1;
          update_chosen();
          ev.preventDefault();
          ev.stopPropagation();
        } else if (ev.key == "Enter") {
          if (downstream_elements.length > 0) {
            dispatch("action", {
              cmd: downstream_elements[focused].dataset.cmd,
            });
            ev.preventDefault();
            ev.stopPropagation();
          }
        }
      }
    }
  }

  function update_chosen() {
    let table = document.getElementById("pick_table");
    if (table != null) {
      let actual_index = 0;
      for (let ii = 0; ii < table.children.length; ii++) {
        if (table.children[ii].dataset.skip == "true") {
          continue;
        }
        if (focused == actual_index) {
          for (let yy = 0; yy < table.children[ii].children.length; yy++) {
            table.children[ii].children[yy].classList.add("chosen");
            if (!isElementInViewport(table.children[ii].children[yy])) {
              table.children[ii].children[yy].scrollIntoView({
                behavior: "smooth",
                block: "center",
              });
            }
          }
        } else {
          for (let yy = 0; yy < table.children[ii].children.length; yy++) {
            table.children[ii].children[yy].classList.remove("chosen");
          }
        }
        actual_index += 1;
      }
    }
    window.history.replaceState(
      { ...history.state, focused: focused },
      undefined
    );
    return "";
  }

  function find_parent_tr(el) {
    while (el != null && el.nodeName != "TR") {
      el = el.parentNode;
    }
    if (el == null) {
      return;
    }
    return el;
  }

  function focus_node(ev) {
    let el = find_parent_tr(ev.target);
    let index = Array.from(el.parentNode.children).indexOf(el);
    focused = index;
    update_chosen();
    focus_first_in_node(document.getElementById("pickerdiv"));
  }

  function handle_double_click(ev) {
    let el = find_parent_tr(ev.target);
    let cmd = el.dataset.cmd;
    dispatch("action", {
      cmd: cmd,
    });
  }

  let previousState = null;

  function update_elements_from_dom() {
    elements = [];
    for (let el of document.querySelector("#pick_table").children) {
      if (el.dataset.skip != "true") {
        elements.push(el);
      }
    }
    downstream_elements = elements;
  }

  onMount(async () => {
    console.log("picker onmount");
    focused = 0;
    window.setTimeout(() => {
      document.getElementById("pickerdiv").scrollIntoView();
      document.getElementById("pick_table").focus();
      update_chosen(false);
    }, 100);
  });

  onDestroy(async () => {});

  let last_url = null;
  beforeUpdate(() => {});

  afterUpdate(async () => {
    if (last_url != window.location.href) {
      last_url = window.location.href;
      update_elements_from_dom();
      focused = 0;
    }
    await tick();

    update_chosen();
    //focus_first_in_node(document.getElementById("pickerdiv"));
  });
</script>

<div on:keyup={handle_text_change} on:keydown={handle_key_down} id="pickerdiv">
  <table
    id="pick_table"
    on:click={focus_node}
    on:dblclick={handle_double_click}
    tabIndex="0"
  >
    <slot name="entries" />
    {update_chosen()}
  </table>
</div>

<style>
  table {
    border-collapse: collapse;
    margin: 25px 0;
    font-size: 0.9em;
    font-family: sans-serif;
    min-width: 400px;
    box-shadow: 0 0 20px rgba(0, 0, 0, 0.15);
    width: 99%;
  }

  thead tr {
    background-color: #009879;
    color: #ffffff;
    text-align: left;
  }
  th,
  :global(#pick_table td) {
    padding: 12px 15px;
  }

  :global(.chosen) {
    background-color: #bfbfff;
  }
</style>
