<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import {
    enter_mode,
    leave_mode,
    get_last_path,
    mode_args_store,
  } from "../lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "../lib/Picker.svelte";
  import { get_node } from "../lib/util.ts";
  import { onMount, onDestroy } from "svelte";

  let mode_args;
  mode_args_store.subscribe((value) => {
    mode_args = value;
  });

  async function handle_action(ev) {
    enter_mode("node", { path: ev.detail.cmd }, false);
  }
  let search_results = [];

  async function perform_search() {
    console.log("ripgrep below", mode_args);
    let rg_results = await invoke("ripgrep_below_node", {
      queryPath: mode_args.path,
      searchTerm: mode_args.search_term,
    });
    let translated_results = [];
    console.log(rg_results);
    for (let result of rg_results) {
      let pretty_path = result.path;
      if (pretty_path == "") {
        pretty_path = "(root)";
      }
      let text = `${pretty_path}: <strong>${result.title}</strong>`;
      for (let pt of result.parent_titles) {
        text += " / " + pt;
      }

      text += "<br />";
      let counter = 0;
      for (let line of result.lines) {
        text += line[0] + ": " + line[1] + "<br />";
        counter += 1;
        if (counter > 7) {
          text += "...";
          break;
        }
      }
      translated_results.push({
        cmd: result.path,
        text: text,
      });
    }
    search_results = translated_results;
	return search_results;
  }
</script>

<div>
  <Picker on:action={handle_action}>
    <div slot="message"><h1>Node search</h1></div>
    <svelte:fragment slot="entries">
	{#await perform_search() then search_results}
      {#each search_results as result}
        <tr data-cmd={result.cmd}><td>{@html result.text}</td></tr>
      {/each}
	{/await}
    </svelte:fragment>
  </Picker>
</div>
