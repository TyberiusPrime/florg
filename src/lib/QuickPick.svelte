<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  const dispatch = createEventDispatcher();

  export let action = "";
  export let text = "";
  export let entries = "";

  var listener = new window.keypress.Listener();
  listener.reset();
  listener.stop_listening();

  function key_pressed(key) {
    let letter = key.slice(-1);
    if (key.startsWith("shift")) {
      letter = letter.toUpperCase();
    }
    console.log("key pressed" + letter);
    for (let entry of entries) {
      if (entry.key == letter) {
        dispatch("action", entry.target_path);
      }
    }
  }

  for (let letter of 
    // prettier-ignore
	  ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
	"shift a", "shift b", "shift c", "shift d", "shift e", "shift f", "shift g", "shift h", "shift i", "shift j", "shift k", "shift l", 
	"shift m", "shift n", "shift o", "shift p", "shift q", "shift r", "shift s", "shift t", "shift u", "shift v", "shift w", "shift x", 
	"shift y", "shift z",
	  ]) {
	  listener.register_combo({
	  keys: letter,
	  is_exclusive: true,
	  prevent_repeat: true,
	  on_keyup: (async (ev) => {
	  key_pressed(letter)
		//await go_sub_node(letter.toUpperCase());
	  })
	  }
  );
  }

  onMount(async () => {
    console.log("navtree mount");
    listener.listen();
  });

  onDestroy(() => {
    console.log("navtree destroy");
    listener.stop_listening();
  });
</script>

<div>
  {text}
  {#each entries as entry}
    <div>
      <span class="hotkey">{entry.key}</span>
      {entry.text}
    </div>
  {/each}
</div>
