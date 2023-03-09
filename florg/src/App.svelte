<script lang="ts">
  import View from "./lib/View.svelte";
  import { SvelteToast } from "@zerodevx/svelte-toast";
  import {
    enter_mode,
    assign_last_mode,
    mode_store,
    mode_args_store,
    mode_transient_store,
  } from "./lib/mode_stack.ts";
  //this is global..,
  import * as KeyPress from "./js/keypress-2.1.5.min.js";
  import Router from "svelte-spa-router";
  import { location, querystring  } from "svelte-spa-router";
  /*
  import Greet from "./lib/Greet.svelte";
  import TopTree from "./lib/TopTree.svelte";
  import NavTable from "./lib/NavTable.svelte";
  import TinyNav from "./lib/TinyNav.svelte";
  import Content from "./lib/Content.svelte";
  import DateMode from "./lib/DateMode.svelte";
  import PickMode from "./lib/PickMode.svelte";
  import MailListHeader from "./lib/MailListHeader.svelte";
  import MailListContent from "./lib/MailListContent.svelte";
  import MailMessage from "./lib/MailMessage.svelte";
  import GotoMode from "./lib/GotoMode.svelte";
  import SearchMode from "./lib/SearchMode.svelte";
  import Footer from "./lib/Footer.svelte";
  import ChatGPT from "./lib/ChatGPT.svelte";
  import * as KeyPress from "./js/keypress-2.1.5.min.js";
  import { invoke } from "@tauri-apps/api/tauri";
  import { emit, listen } from "@tauri-apps/api/event";
  import { exit } from "@tauri-apps/api/process";
  import { onMount, onDestroy } from "svelte";
  import { readText, writeText } from "@tauri-apps/api/clipboard";
  import asciidoctor from "asciidoctor";
  import { fetch } from "@tauri-apps/api/http";
  import { parse as csv_parse } from "csv-parse/browser/esm";
  import { format_date, iso_date } from "./lib/util.ts";
  //let Asciidoctor = asciidoctor();
  */

  import NodeMode from "./modes/Node.svelte";
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
  {$location}
  <br />
  <SvelteToast {options} />
  <Router {routes} />
</div>
