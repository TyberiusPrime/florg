<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import {
    push as push_mode,
    replace as replace_mode,
  } from "svelte-spa-router";

  export let params = {};

  async function go_and_edit() {
    console.log("node_edit");
    let path = params.path || "";
    await invoke("edit_node", { path: path });
    replace_mode("/node/" + path);
  }
</script>

<div>
  {#await go_and_edit()}{/await}
</div>
