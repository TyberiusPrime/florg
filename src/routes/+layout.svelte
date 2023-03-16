<script lang="ts">
  import { SvelteToast, toast } from "@zerodevx/svelte-toast";
  import { onMount, onDestroy, beforeUpdate, afterUpdate } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { keypress } from "keypress.js";
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

  var listener = new keypress.Listener();

  listener.register_combo({
    keys: "f5",
    is_unordered: true,
    prevent_repeat: true,
    prevent_default: true,
    on_keyup: (e, count, repeated) => {
      window.location.reload();
    },
  });
  let window_count = new Date().getMilliseconds();
  listener.register_combo({
    keys: "ctrl enter",
    is_unordered: false,
    prevent_repeat: true,
    is_exclusive: true,
    on_keyup: (e, count, repeated) => {
      if (!repeated) {
        console.log("ctrl enter");
        set_mode_ignore_enter();
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
    },
  });

  listener.register_combo({
    keys: "o",
    prevent_default: false,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        goto("/chatgpt");
        e.preventDefault();
      }
    },
  });

  listener.register_combo({
    keys: "p",
    is_unordered: true,
    prevent_repeat: true,
    on_keyup: (e, count, repeated) => {
      if (no_text_inputs_focused()) {
        goto("/palette");
        e.preventDefault();
      }
    },
  });

  listener.listen();

  onDestroy(async () => {
    listener.stop_listening();
    (await unlisten_mouse_button_pressed)();
    (await unlisten_message)();
  });
</script>

<svelte:body on:mouseenter={enter_focus} on:mouseleave={leave_focus} />
<SvelteToast bind:options={toast_options} />
<slot />
<div>
  {window.location.href}
</div>
