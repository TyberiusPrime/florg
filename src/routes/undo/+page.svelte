<script lang="ts">
  import Picker from "$lib/../components/Picker.svelte";
  import Help from "$lib/../components/Help.svelte";
  import Expander from "$lib/../components/Expander.svelte";
  import { goto, invalidateAll } from "$app/navigation";
  import Search from "$lib/../components/Search.svelte";
  import View from "$lib/../components/View.svelte";
  import { toast } from "@zerodevx/svelte-toast";
  import { invoke } from "@tauri-apps/api/tauri";
  import { no_text_inputs_focused, dispatch_keyup } from "$lib/util.ts";
  export let data;
  let viewComponent;
  let overlay = "";
  let focused;
  let help_entries = [{ key: "Esc", text: "Go back" }];

  let keys = {
    Escape: () => {
      if (overlay != "") {
        viewComponent.leave_overlay();
        let chosen = document.getElementsByClassName("chosen");
        console.log(chosen);
        if (chosen.length > 0) {
          console.log("chosen is", chosen);
          chosen[0].scrollIntoView();
        }
        return true;
      }
    },
    h: () => {
      if (no_text_inputs_focused()) {
        viewComponent.enter_overlay("help");
        return true;
      }
    },
  };
  async function handle_action(ev) {
    let hash = ev.detail.cmd;
    toast.push("undo " + hash);
    try {
      await invoke("git_undo", { hash: hash });
      goto("/tree");
    } catch (e) {
      toast.push("Error: " + e);
    }
  }
  function handle_keys(ev) {
    dispatch_keyup(keys)(ev);
  }
</script>

<View bind:this={viewComponent} bind:overlay>
  <div slot="header" class="header">
    <h1>Undo history</h1>
  </div>
  <div slot="content" on:keyup={handle_keys}>
    <Picker on:action={handle_action} bind:focused>
      <svelte:fragment slot="entries">
        {#each data.entries as el, index}
          {#if index != data.entries.length - 1}
            <tr data-cmd={data.entries[index + 1].hash} class="msg_entry">
              <td>
                <Expander text={el.hash} threshold="10" />
              </td>
              <td style="min-width:9em;">
                {@html el.date.replace(" ", "<br />")}
              </td><td>
                {el.message}
              </td>
            </tr>
          {/if}
        {/each}
      </svelte:fragment>
    </Picker>
  </div>
  <svelte:fragment slot="overlays">
    {#if overlay == "help"}
      <Help bind:entries={help_entries} />
    {/if}
  </svelte:fragment>
</View>

<style>
  td,
  th {
    vertical-align: top;
    padding-right: 0.5em;
    border-bottom: 1px solid #ddd;
    padding-top: 5px;
  }
</style>
