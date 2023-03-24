<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import Expander from "$lib/../components/Expander.svelte";
  import Focusable from "$lib/../components/Focusable.svelte";
  import View from "$lib/../components/View.svelte";
  import Overlay from "$lib/../components/Overlay.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { render_text_cached } from "$lib/../routes/node/[[path]]/funcs";
  import { patch_tree, patch_tree_content, flattenObject } from "./funcs";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";
  import { add_code_clipboards, dispatch_keyup } from "$lib/util.ts";
  import { focus_first_in_node} from "$lib/util.ts";
  import { emit, listen } from "@tauri-apps/api/event";
  export let data;

  let cache = {};
  let activeIndex;
  let overlay = "";
  let help_entries = [{ key: "Esc", text: "Go back" }];
  let keys = {
    x: () => {
      toast.push("hello");
      document
        .getElementById("node_content")
        .find("all", false, false, true, false);
      return true;
    },
  };

  async function get_rendered_node(path) {
    console.log("retreiving rendered node", path);
    if (cache[path] === undefined) {
      let node = await invoke("get_node", { path: path });
      if (node.node.raw != "") {
        let rt = await render_text_cached(path, node.node.raw);
        cache[path] = rt;
      } else {
        cache[path] = "(empty node)";
      }
    }
    return cache[path];
  }
  async function itemChanged(ev) {
    let path = ev.detail.path;
    if (path !== undefined) {
      data.current_item = path;
      data = data;
    }
  }

  async function toggle_node(path, do_collapse) {
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (data.flat[ii].path == path) {
        if (data.flat[ii].has_children) {
          let was_expanded = false;
          let yy = ii + 1;
          if (yy > data.flat.length - 1) {
            was_expanded = false;
          } else {
            was_expanded = data.flat[yy].path.startsWith(path);
          }
          if (was_expanded && do_collapse) {
            patch_tree(data.tree, path, []);
            data.flat = flattenObject(data.tree);
            data = data;
          } else {
            console.log("expanding", path);
            let subtree = await invoke("get_tree", { path: path, maxDepth: 1 });
            patch_tree(data.tree, path, subtree.children);
            data.flat = flattenObject(data.tree);
            data = data;
          }
        }
        break;
      }
    }
  }

  async function item_toggle_children(ev) {
    let path = ev.detail.path;
    await toggle_node(path, true);
    window.setTimeout(() => {
      activeIndex = activeIndex;
    }, 10);
  }

  let can_expand = (path, do_expand) => {
    console.log("querying expandability", path);
    for (let ii = 0; ii < data.flat.length; ii++) {
      if (data.flat[ii].path == path) {
        if (data.flat[ii].has_children) {
          let was_expanded = false;
          let yy = ii + 1;
          if (yy > data.flat.length - 1) {
            was_expanded = false;
          } else {
            was_expanded = data.flat[yy].path.startsWith(path);
          }
          if (do_expand && !was_expanded) {
            toggle_node(path, false);
          }
          return !was_expanded;
        }
      }
    }
    return false;
  };

  let can_contract = (path, do_contract) => {
	console.log("querying contractability", path);
	for (let ii = 0; ii < data.flat.length; ii++) {
	  if (data.flat[ii].path == path) {
		if (data.flat[ii].has_children) {
		  let was_expanded = false;
		  let yy = ii + 1;
		  if (yy > data.flat.length - 1) {
			was_expanded = false;
		  } else {
			was_expanded = data.flat[yy].path.startsWith(path);
		  }
		  if (do_contract && was_expanded) {
			toggle_node(path, true);
		  }
		  return was_expanded;
		}
	  }
	}
	return false;
  };

  async function item_selected(ev) {
    let path = ev.detail.path;
    if (data.currently_edited[path] == undefined) {
      let node = await invoke("get_node", { path: path });
      data.currently_edited[path] = [node.node.raw];
      data = data;
      await invoke("edit_node", {
        path: path,
        windowTitle: appWindow.label,
      });
    }
  }
  //this is an event from rust
  const unlisten_node_changed = listen("node-changed", async (event) => {
    // a specific node was reread
    data.currently_edited[event.payload] = undefined;
    let new_node = await invoke("get_node", { path: event.payload });
    cache[event.payload] = undefined;
    //patch_tree_content(data.tree, event.payload, new_node.node.raw);
    //data.flat = flattenObject(data.tree);
    console.log("node changed", new_node.node.raw);
    data = data;
  });

  const unliste_node_unchanged = listen("node-unchanged", async (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing?
    console.log("node unchanged", event.payload);
    data.currently_edited[event.payload] = undefined;
    data = data;
  });
  const unliste_node_temp_changed = listen(
    "node-temp-changed",
    async (event) => {
      //some node was edited, but not confirmed yet.
      //reload to refresh the currently edited thing?
      console.log("node temp-changed", event.payload[0]);
      //todo
      /*
      if (event.payload[0] == data.path) {
        let content_text = event.payload[1];
        data.rendered = await render_text(content_text);
        data = data;
      }
	  */
    }
  );
  const unlisten_message = listen("message", (event) => {
    //some node was not changed / editing aborted.
    //reload to refresh the currently edited thing
  });

  onMount(async () => {});

  onDestroy(async () => {
    (await unlisten_node_changed)();
    (await unliste_node_unchanged)();
    (await unliste_node_temp_changed)();
    (await unlisten_message)();
  });
  function handle_overlay_leave() {
    // toast.push("overlay:leave in node");
    overlay = "";
  }

  async function handle_key_up_content(ev) {
  console.log(ev);
	if (ev.key == "ArrowLeft") {
	console.log("arrowLeft");
	focus_first_in_node(document.getElementById("tree_parent"))
	}
  }
</script>

<View on:keyup={dispatch_keyup(keys)}>
  <div slot="header" >
  hello
  </div>
  <svelte:fragment slot="content">
    <div class="smallcolumn main_div" id="tree_parent">
      <Focusable
        on:itemChanged={itemChanged}
        on:itemSpace={item_toggle_children}
        bind={activeIndex}
        on:itemSelected={item_selected}
        bind:can_expand
        bind:can_contract
      >
        {#each data.flat as node}
          <tr
            data-path={node.path}
            class={data.currently_edited[node.path] !== undefined
              ? "edited"
              : ""}
          >
            <td class="mono">
              {@html node.indention}{node.path}{#if node.has_children && !node.children_shown}<span
                  class="more">+</span
                >
              {/if}
            </td>
            <td>{node.title} </td>
          </tr>
        {/each}
      </Focusable>
    </div>
    <div
      class="main_div column {data.currently_edited[data.current_item] !== undefined
        ? 'edited'
        : ''}"
      id="node_content"
	  on:keyup={handle_key_up_content}
    >
      {#await get_rendered_node(data.current_item)}
        loading...
      {:then rendered}
        {@html rendered}
      {/await}
    </div>
  </svelte:fragment>
  <div slot="footer">
    <Overlay on:leave={handle_overlay_leave} bind:overlay>
      {#if overlay == "help"}
        <Help bind:entries={help_entries} />
      {:else if overlay == ""}
        <div on:click={show_help}>
          Press <span class="hotkey">h</span> for help.
        </div>
      {/if}
    </Overlay>
  </div>
</View>

<style>
  .mono {
    font-family: monospace;
    color: #666;
  }

  .more {
    color: #3636ff;
  }

  .column {
    float: none;
    overflow-y: scroll;
    overflow-x: hidden;
    padding-left: 0.5em;
    border-left: 3px dashed grey;
  }
  .smallcolumn {
    float: left;
    max-width: 49%;
  overflow-y: scroll;
    overflow-x: hidden;

    padding-right: 0.5em;
  }

  :global(.edited) {
    background-color: #ffdfdf;
  }
</style>
