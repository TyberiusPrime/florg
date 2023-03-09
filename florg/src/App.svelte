<script lang="ts">
  import View from "./lib/View.svelte";
  import { SvelteToast } from "@zerodevx/svelte-toast";
  //this is global..,
  import * as KeyPress from "./js/keypress-2.1.5.min.js";
  import Router from "svelte-spa-router";
  import { location, querystring  } from "svelte-spa-router";

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
	"*": NotFound,

  };

  const urlParams = new URLSearchParams(window.location.search);
  let mode = urlParams.get("mode") || "node";
</script>

<div>
  {window.location.hash}
  <SvelteToast {options} />
  <Router {routes} />
</div>
