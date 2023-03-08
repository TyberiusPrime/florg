<script lang="ts">
  import View from "./lib/View.svelte";
  import {
    enter_mode,
    assign_last_mode,
    mode as mode_store,
    mode_args as mode_args_store,
    mode_transient as mode_transient_store,
  } from "./lib/mode_stack.ts";
  //this is global..,
  import * as KeyPress from "./js/keypress-2.1.5.min.js";

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
  import PostalMime from "postal-mime";
  import { fetch } from "@tauri-apps/api/http";
  import { parse as csv_parse } from "csv-parse/browser/esm";
  import { format_date, iso_date } from "./lib/util.ts";
  //let Asciidoctor = asciidoctor();
  */

  import ModeNode from "./modes/Node.svelte";
  import NavMode from "./modes/Nav.svelte";
  import PaletteMode from "./modes/Palette.svelte";

  let mode;
  let mode_args;
  let mode_transient;

  mode_store.subscribe((value) => {
    mode = value;
  });
  mode_args_store.subscribe((value) => {
    mode_args = value;
  });
  mode_transient_store.subscribe((value) => {
    mode_transient = value;
  });

  assign_last_mode();
</script>

<div>
  {#if mode == "node"}
    <ModeNode {mode} {mode_args} />
  {:else if mode == "nav"}
    <NavMode {mode} {mode_args} />
  {:else if mode == "palette"}
    <PaletteMode {mode} {mode_args} />
  {:else}
    unknown mode {JSON.stringify(mode)}
  {/if}
</div>
