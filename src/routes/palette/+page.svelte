<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { get_last_path } from "$lib/mode_stack.ts";
  import { exit } from "@tauri-apps/api/process";
  import Picker from "$lib/../components/Picker.svelte";
  import { fetch, Body } from "@tauri-apps/api/http";
  import { toast } from "@zerodevx/svelte-toast";
  import {parse as csv_parse} from 'csv-parse/browser/esm/sync';


async function get_node(path) {
  await invoke("get_node", { path });
}

  async function handle_action(ev) {
    let cmd = ev.detail.cmd;
    if (cmd == "reload") {
      await invoke("reload_data", {});
      console.log("reloaded");
    } else if (cmd == "exit") {
      await exit(1);
    } else if (cmd == "create_date_nodes") {
      await create_date_nodes();
    } else if (cmd == "settings") {
      await edit_settings();
    } else if (cmd == "terminal") {
	  await invoke("start_terminal", {folder: await invoke("get_node_folder_path", {path: get_last_path()})});
    } else if (cmd == "download_awesome_chatpgt_prompts") {
      await download_awesome_chatpgt_prompts();
    } else {
      console.log("unhandled command", cmd);
    }
	window.history.back();

  }

  async function edit_settings() {
    return await invoke("edit_settings", {});
  }
  async function create_date_nodes() {
    let last_path = get_last_path();
    let node = await get_node(last_path);
	console.log(node);
    if (node.children.length > 0) {
	toast.push(
        "<span class='error'>Can not create date nodes on an node that already has children</span>");
    } else {
      let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(
        new Date()
      );
      let year = window.prompt(
        "Please enter the year to create a calendar for",
        ye
      );
      if (year != null) {
        console.log(year);
        let year_parsed = parseInt(year);
        console.log(year_parsed);
        if (!isNaN(year_parsed)) {
          toast.push("Created calendar");
          await invoke("create_calendar", {
            parentPath: current_path,
            year: year_parsed,
          });
          await invoke("reload_data", {});
          await load_node(current_path);
        }
      }
    }
  }

  async function download_awesome_chatpgt_prompts() {
    let url =
      "https://github.com/f/awesome-chatgpt-prompts/raw/main/prompts.csv";
    const response = await fetch(url, {
      method: "GET",
      timeout: 10,
      responseType: 2,
    });
    if (response.ok) {
      csv_parse(
        response.data,
        {
          columns: true,
          skip_empty_lines: true,
        },
        async (error, records) => {
          let prompts = {};
          for (let record of records) {
            prompts[record["act"]] = record["prompt"];
          }
          console.log(prompts);
          await invoke("chatgpt_update_prompts", {
            key: "awesome",
            prompts: prompts,
          });
        }
      );
      toast.push("Downloaded awesome chatgpt prompt & stored in prompts.toml");
    } else {
      toast.push(`<span class='error'>error downloading prompts. ${response.status}</span>`);
    }
  }
</script>

<div>
  <Picker on:action={handle_action}>
    <div slot="message"><h1>Pick a command</h1></div>
    <svelte:fragment slot="entries">
	  <tr data-cmd="terminal"><td>Open terminal here</td></tr>
      <tr data-cmd="settings"><td>Edit settings</td></tr>
      <tr data-cmd="create_date_nodes"><td>Create date nodes</td></tr>
      <tr data-cmd="exit"><td>Exit the app</td></tr>
      <tr data-cmd="reload"><td>Reload data from disk</td></tr>
      <tr data-cmd="download_awesome_chatpgt_prompts"
        ><td>Update awesome chatgpt prompts</td></tr
      >
    </svelte:fragment>
  </Picker>
</div>
