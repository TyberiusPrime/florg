<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher, tick } from "svelte";
  const dispatch = createEventDispatcher();
  import { Fzf, byLengthAsc } from "fzf";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { set_temp_history, get_temp_history } from "../lib/mode_stack.ts";
  import { no_text_inputs_focused, focus_first_in_node } from "../lib/util.ts";

  import View from "./View.svelte";
  import { isElementInViewport } from "../lib/util.ts";

  export let elements = [];
  export let focused;
  export let enable_filter = true;

  let input_text = "";
  let last_text = "";
  let test_text = "shu";
  let downstream_elements = [];

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
          if (focused < downstream_elements.length - 1) {
            focused += 1;
            update_chosen();
          }
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
      for (let ii = 0; ii < table.children.length; ii++) {
        if (focused == ii) {
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
    let cmd = el.cmd;
    dispatch("action", {
      cmd: cmd,
    });
  }

  let previousState = null;

  function popStateChanged(event) {
    set_temp_history(event.state);
  }
  // This is removed in the destroy() invocation below
  window.addEventListener("popstate", popStateChanged);

  function update_elements_from_dom() {
    elements = [];
    for (let el of document.querySelector("#pick_table").children) {
      elements.push(el);
    }
    downstream_elements = elements;
  }

  onMount(async () => {
    console.log("picker onmount");
    focused = 0;
    window.setTimeout(() => {
      let state = get_temp_history();
      if (state != null && state.input_text != undefined) {
        input_text = state.input_text;
        handle_text_change();
        focused = state.focused;
      }

    focus_first_in_node(document.getElementById("pickerdiv"));

      update_chosen(false);
    }, 100);
  });

  onDestroy(async () => {
    window.removeEventListener("popstate", popStateChanged);
  });

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
  <View>
    <div slot="header">
      <slot name="message" />
      {#if enable_filter}
        filter: <input id="typebox" autofocus bind:value={input_text} />
      {/if}
    </div>
    <div slot="content" tabIndex="0">
      <div id="the_content">
        <table
          id="pick_table"
          on:click={focus_node}
          on:dblclick={handle_double_click}
		  tabIndex=0
        >
          <slot name="entries" />
        </table>
      </div>
    </div>
    <div slot="footer">
      <slot name="footer">
        Press <span class="hotkey">Esc</span> to abort.
      </slot>
    </div>
  </View>
  {update_chosen()}
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
