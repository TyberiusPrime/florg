<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import Picker from "$lib/../components/Picker.svelte";
  import { goto, invalidateAll } from "$app/navigation";
  import View from "$lib/../components/View.svelte";
  import { iso_date } from "$lib/util";
  export let data;
  let viewComponent;
  let overlay;
  let enable_filter = false;
  let start_date = iso_date(data.start_date);
  let stop_date = iso_date(data.stop_date);

  function handle_action(ev) {
  goto("/tree/" + ev.detail.cmd);
  }

  async function handle_change(ev) {
    let start = Date.parse(start_date);
    let stop = Date.parse(stop_date);
    await goto("/agenda/" + start + "/" + stop, {
      replaceState: true,
    });
  }
</script>

<div>
  <Picker on:action={handle_action} bind:enable_filter>
    <div slot="message">
      <h1>Agenda</h1>
      <input type="date" on:change={handle_change} bind:value={start_date} />
      <input type="date" on:change={handle_change} bind:value={stop_date} />
    </div>
    <svelte:fragment slot="entries">
      {#each data.hits as hit}
        <tr data-cmd={hit.rg.path}>
          <td class="shrink">{hit.str_date}</td>
          <td class="shrink">{hit.rg.path}</td>
          <td>{hit.rg.title.replace(/<\d{4}-\d{1,2}-\d{1,2}>/, "")}</td>
        </tr>
      {/each}
    </svelte:fragment>
  </Picker>
</div>

<style>
  .shrink {
    width: 0.1%;
    white-space: nowrap;
  }
</style>
