<script lang="ts">
  import { SvelteToast, toast } from "@zerodevx/svelte-toast";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { WebviewWindow as www } from "@tauri-apps/api/window";
  import { emit, listen } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { no_text_inputs_focused } from "$lib/util.ts";
  import { Buffer } from "buffer";
  globalThis.Buffer = Buffer;

  const unlisten_mouse_button_pressed = listen(
    "mouse-button-pressed",
    async (event) => {
      if (has_focus) {
        if (event.payload == 8) {
          window.history.back();
        } else if (event.payload == 9) {
          window.history.forward();
        }
      }
    }
  );
  const unlisten_message = listen("message", async (event) => {
    toast.push(event.payload);
  });

  let has_focus = false;
  function enter_focus() {
    has_focus = true;
  }
  function leave_focus() {
    has_focus = false;
  }
  let toast_options = {
    // toast options
    pausable: true,
    duration: 10000,
  };
  function handle_key_down(e) {
    console.log("handle_key_down on body", e.key);
    if (e.key == " " && no_text_inputs_focused()) {
      e.preventDefault();
      return false;
    }
  }

  let window_count = new Date().getMilliseconds();
  function handle_key_up(e) {
    console.log("handle_key_up on body", e.key);
    if (e.key == "Escape") {
      console.log("going back/forward");
      if (e.shiftKey) {
        window.history.forward();
      } else {
        window.history.back();
      }
    } else if (e.key == "o" && no_text_inputs_focused()) {
      goto("/chatgpt");
    } else if (e.key == "p" && no_text_inputs_focused()) {
      goto("/palette");
    } else if (e.key == "F5") {
      window.location.reload();
    } else if (e.key == "Enter") {
      if (e.ctrlKey) {
        console.log("ctrl enter");
        const webview = new www("florg_" + window_count, {
          url: window.location.href,
          title: "florg" + window_count,
        });
        webview.once("tauri://error", function (e) {
          console.log(e);
          error_toast("Error during window creation" + JSON.stringify(e));
        });
        window_count += 1;
      }
    }
  }

  onDestroy(async () => {
    (await unlisten_mouse_button_pressed)();
    (await unlisten_message)();
  });
</script>

<svelte:body
  on:mouseenter={enter_focus}
  on:mouseleave={leave_focus}
  on:keydown={handle_key_down}
  on:keyup={handle_key_up}
/>
<SvelteToast bind:options={toast_options} />
<slot />
<div>
  {window.location.href}
</div>
