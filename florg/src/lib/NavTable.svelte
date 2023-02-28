<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  const dispatch = createEventDispatcher();

  export let nav_table = [];
  export let current_path;
  export let nav_mode_start = "";

  function chunk_nav(nav_table) {
    let rows = [];
    let cols = 5;
    let entries = 5;
    for (let entry = 0; entry < entries; entry += 1) {
      rows.push([]);
    }
    for (let column = 0; column < cols; column += 1) {
      for (let entry = 0; entry < entries; entry += 1) {
        let index = column * entries + entry;
        if (index < nav_table.length) {
          rows[entry].push(nav_table[index]);
        }
      }
    }
    return rows;
  }

  function go_sub_node(key, normal_mode) {
    dispatch("goto_node", {
      path: current_path + key,
      normal_mode: normal_mode,
    });
  }
  function goto_node(key) {
    dispatch("goto_node", { path: key });
  }

  var listener_nav = new window.keypress.Listener();
  listener_nav.reset();
  listener_nav.stop_listening();

  listener_nav.simple_combo("esc", () => {
    console.log("going back to", nav_mode_start);
    goto_node(nav_mode_start);
    dispatch("leave", false);
  });

  for (let letter of 
    // prettier-ignore
	  ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"]) {
	  listener_nav.register_combo({
	  keys: letter,
	  prevent_repeat: true,
	  on_keyup: (async (ev) => {
		console.log("key pressed" + letter);
		await go_sub_node(letter.toUpperCase());
	  })
	  }
  );

  }

  listener_nav.register_combo({
    keys: "space",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      dispatch("leave", false);
    },
  });

  listener_nav.register_combo({
    keys: "enter",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      dispatch("leave", true);
    },
  });

  listener_nav.register_combo({
    keys: "backspace",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      if (current_path.length > 0) {
        goto_node(current_path.slice(0, -1));
      }
    },
  });
  listener_nav.register_combo({
    keys: "home",
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: async (ev) => {
      if (current_path.length > 0) {
        goto_node("");
      }
    },
  });

  onMount(async () => {
    console.log("navtree mount");
    listener_nav.listen();
  });

  onDestroy(() => {
    console.log("navtree destroy");
    listener_nav.stop_listening();
  });
</script>

<div id="navtable">
  {#if nav_table.length > 0}
    Navigation mode:
    <table>
      {#each chunk_nav(nav_table) as row}
        <tr>
          {#each row as entry}
            <td
              ><a on:click={(ev) => go_sub_node(entry.key, false)}
                ><span class="hotkey">{entry.key}</span>&nbsp;{entry.text}</a
              ></td
            >
          {/each}
        </tr>
      {/each}
      {#if nav_table.length > 24}
        <tr>
          <td colspan="2"
            ><a on:click={(ev) => go_sub_node("z", false)}
              >><span class="hotkey">Z</span> More...</a
            ></td
          >
        </tr>
      {/if}
    </table>
  {:else}
    (No further children. a-z to create)
  {/if}
</div>

<style>
  #navtable {
    background-color: #cfcfcf;
    margin: 0px;
    padding: 5px;
    margin-right: 5px;
  }

  #navtable table {
    width: 100%;
  }

  div {
    text-align: left;
  }
</style>
