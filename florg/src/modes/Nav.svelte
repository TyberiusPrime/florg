<script lang="ts">
  import { enter_mode, leave_mode } from "../lib/mode_stack.ts";
  import { get_node, no_text_inputs_focused } from "../lib/util.ts";
  import { onMount, onDestroy } from "svelte";

  import View from "../lib/View.svelte";
  import Overlay from "../lib/Overlay.svelte";
  import Help from "../lib/Help.svelte";

  export let mode;
  export let mode_args;

  let inital_path = mode_args.enter_path;
  let path = inital_path;
  let overlay = "";

  let help_entries = [
    { key: "Esc", text: "Go back" },
    { key: "Space", text: "Open current node" },
    { key: "Enter", text: "Open&Edit current node" },
  ];
  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  listener.register_combo({
    keys: "esc",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated) {
        console.log("listener nav leave");
        leave_mode();
      }
    },
  });

  listener.register_combo({
    keys: "h",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated && no_text_inputs_focused()) {
        overlay = "help";
      }
    },
  });

  listener.register_combo({
    keys: "space",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated) {
        enter_mode("node", { path: path }, false);
      }
    },
  });

  listener.register_combo({
    keys: "enter",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated) {
        enter_mode("node", { path: path, edit: true }, false);
      }
    },
  });

  function handle_overlay_leave() {
    overlay = "";
  }

  let last_path = "---";
  let current_node;
  async function handle_input_changed(ev) {
    if (last_path != path) {
      path = path.toUpperCase();
      path = path.replace(/[^A-Z0-9]/g, "");
      last_path = path;
      current_node = await get_node(path);
    }
  }

  onMount(async () => {
    await handle_input_changed();
    listener.listen();
  });

  onDestroy(async () => {
    listener.stop_listening();
  });

  function indent(depth) {
    return "&nbsp;".repeat(depth);
  }

  function goto_level(index) {
    let new_path = "";
    for (let ii = 0; ii < index + 1; ii++) {
      new_path += current_node.levels[ii][0];
    }
    path = new_path;
    handle_input_changed();
  }

  function descend(letter) {
    path += letter;
    handle_input_changed();
  }

  function accept_current_node() {
    enter_mode("node", { path: path }, false);
  }
</script>

<div>
  <View bind:mode bind:mode_args>
    <div slot="header">
      <h1>Navigation mode.</h1>
      Path:<input
        type="text"
        id="path"
        bind:value={path}
        autofocus
        on:keyup={handle_input_changed}
      />
    </div>
    <div slot="content">
      <div id="the_content">
        {#if current_node != null}
          Parents:
          <table>
            {#each current_node.levels as level, index}
              {#if index < current_node.levels.length - 1}
                <tr
                  ><td
                    ><a on:click={(ev) => goto_level(index)}
                      >{level[0]}{@html indent(index)} {level[1]}</a
                    ></td
                  ></tr
                >
              {:else}
                <tr><td>{level[0]}{@html indent(index)} {level[1]}</td></tr>
              {/if}
            {/each}
          </table>
          {#if current_node.levels.length == 0}
            (root)
          {/if} <br />
          Current node:
          <a on:click={accept_current_node}>
            {#if current_node.node !== null}
              {current_node.node.header.title}
            {:else}
              (Empty node)
            {/if}
          </a>
          <br />
          <br />

          Known children:
          <table>
            {#each current_node.children as child}
              <tr>
                <a on:click={descend(child.path.slice(-1))}>
                  <th class="hotkey">
                    {child.path.slice(-1)}
                  </th>
                  <td>{child.header.title}</td>
                </a>
              </tr>
            {/each}
          </table>
        {/if}
      </div>
    </div>
    <div slot="footer">
      <Overlay {listener} on:overlay_leave={handle_overlay_leave} bind:overlay>
        {#if overlay == "help"}
          <Help bind:entries={help_entries} />
        {:else if overlay == ""}
          Press <span class="hotkey">h</span> for help.
        {:else}
          Unknown overlay: {overlay}
        {/if}
      </Overlay>
    </div>
  </View>

</div>

<style>
  #the_content {
    padding: 1em;
  }
</style>
