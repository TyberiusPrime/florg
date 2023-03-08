<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { Fzf, byLengthAsc } from "fzf";
  import { onMount, onDestroy } from "svelte";
  import { leave_mode } from "../lib/mode_stack.ts";
  import View from "../lib/View.svelte";

  export let elements = [];
  export let mode;
  export let mode_args;

  let focused = 0;
  let input_text = "";
  let last_text = "";
  let downstream_elements = [];
  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.register_combo({
    keys: "esc",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      console.log("leave picker");
      leave_mode();
    },
  });

  listener.register_combo({
    keys: "enter",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      console.log("entr");
      if (downstream_elements.length > 0) {
        dispatch("action", {
          cmd: downstream_elements[focused].dataset.cmd,
        });
      }
    },
  });

  listener.register_combo({
    keys: "down",
    is_unordered: true,
    prevent_default: true,
    on_keydown: (e, count, repeated) => {
      if (focused < downstream_elements.length - 1) {
        focused += 1;
        update_chosen();
      }
    },
  });
  listener.register_combo({
    keys: "up",
    is_unordered: true,
    prevent_default: true,
    on_keydown: (e, count, repeated) => {
      if (focused > 0) {
        focused -= 1;
        update_chosen();
      }
    },
  });
  function handle_text_change(ev) {
    const fzf = new Fzf(elements, {
      selector: (item) => item.innerText,
      tiebreakers: [byLengthAsc],
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
    }
  }

  function update_chosen() {
    let table = document.getElementById("pick_table");
    if (table != null) {
      for (let ii = 0; ii < table.children.length; ii++) {
        if (focused == ii) {
          for (let yy = 0; yy < table.children[ii].children.length; yy++) {
            table.children[ii].children[yy].classList.add("chosen");
          }
        } else {
          for (let yy = 0; yy < table.children[ii].children.length; yy++) {
            table.children[ii].children[yy].classList.remove("chosen");
          }
        }
      }
    }
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
  }

  function handle_double_click(ev) {
    let el = find_parent_tr(ev.target);
    let cmd = el.cmd;
    dispatch("action", {
      cmd: cmd,
    });
  }

  onMount(async () => {
    window.setTimeout(() => {
      elements = [];
      for (let el of document.querySelector("#pick_table").children) {
        elements.push(el);
      }
      console.log(elements);
      downstream_elements = elements;
      update_chosen();
      listener.listen();
    }, 100);
  });

  onDestroy(async () => {
    listener.stop_listening();
  });
</script>

<div on:keyup={handle_text_change}>
  <View bind:mode bind:mode_args>
    <div slot="header">
      <slot name="message" />
      filter: <input id="typebox" autofocus bind:value={input_text} />
    </div>
    <div slot="content">
      <div id="the_content">
        <table
          id="pick_table"
          on:click={focus_node}
          on:dblclick={handle_double_click}
        >
          <slot name="entries" />
        </table>
      </div>
    </div>
    <div slot="footer">
      Press <span class="hotkey">Esc</span> to abort.
    </div>
  </View>
</div>

<style>
  table {
    border-collapse: collapse;
    margin: 25px 0;
    font-size: 0.9em;
    font-family: sans-serif;
    min-width: 400px;
    box-shadow: 0 0 20px rgba(0, 0, 0, 0.15);
  }

  thead tr {
    background-color: #009879;
    color: #ffffff;
    text-align: left;
  }
  th,
  :global #pick_table td {
    padding: 12px 15px;
  }

  :global .chosen {
    background-color: #bfbfff;
  }
</style>
