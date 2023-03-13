<script lang="ts">
  import View from "./lib/View.svelte";
  import { SvelteToast } from "@zerodevx/svelte-toast";
  import { WebviewWindow as www } from "@tauri-apps/api/window";
  //this is global..,
  //#import * as KeyPress from "./js/keypress-2.1.5.min.js";
  import {keypress} from "keypress.js";
  import Router from "svelte-spa-router";
  import { location, querystring } from "svelte-spa-router";
  import { set_mode_ignore_enter } from "./lib/mode_stack.ts";
  import { error_toast } from "./lib/util.ts";
  import { onMount, onDestroy } from "svelte";

  import NodeMode from "./modes/Node.svelte";
  import NodeEdit from "./modes/NodeEdit.svelte";
  import NavMode from "./modes/Nav.svelte";
  import PaletteMode from "./modes/Palette.svelte";
  import NodeSearchMode from "./modes/NodeSearch.svelte";
  import ChatGPTMode from "./modes/ChatGPT.svelte";
  import ChatGPTPickerMode from "./modes/ChatGPTPicker.svelte";
  import MailQuery from "./modes/MailQuery.svelte";
  import MailMessage from "./modes/MailMessage.svelte";
  import NotFound from "./modes/NotFound.svelte";

  let options = {
    // toast options
    pausable: true,
    duration: 10000,
  };
  console.log(keypress);

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
  let window_count = (new Date()).getMilliseconds();
  listener.register_combo({
    keys: "ctrl enter",
    is_unordered: false,
    prevent_repeat: true,
    prevent_default: true,
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
  const routes = {
    "/": NodeMode,
    "/node/:path?": NodeMode,
    "/node_edit/:path?": NodeEdit,
    "/nav/:path?": NavMode,
    "/node_search/:search_term/:path?": NodeSearchMode,
    "/mail_query/:query": MailQuery,
    "/mail_message/:message_id": MailMessage,
    "/chatgpt_picker/": ChatGPTPickerMode,
    "/chatgpt/:filename": ChatGPTMode,
    "/palette": PaletteMode,
    "*": NotFound,
  };

  onDestroy(() => {
	listener.reset();
  });

  const urlParams = new URLSearchParams(window.location.search);
  let mode = urlParams.get("mode") || "node";
</script>

<div>
  <SvelteToast {options} />
  <Router {routes} />
</div>
