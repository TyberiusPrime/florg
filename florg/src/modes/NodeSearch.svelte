<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { enter_mode, leave_mode, get_last_path } from "../lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "../lib/Picker.svelte";
  import { get_node } from "../lib/util.ts";
  import { onMount, onDestroy } from "svelte";

  async function handle_action(ev) {
  enter_mode("node", { path: ev.detail.cmd }, false);
	

  }

  export let mode;
  export let mode_args;

  let search_results = [];

  onMount(async () => {
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
  });
</script>

<div>
  <Picker on:action={handle_action} {mode} {mode_args}>
    <div slot="message"><h1>Node search</h1></div>
    <svelte:fragment slot="entries">
	{#each search_results as result}
	  <tr data-cmd="{result.cmd}"><td>{@html result.text}</td></tr>
	  {/each}
    </svelte:fragment>
  </Picker>
</div>
