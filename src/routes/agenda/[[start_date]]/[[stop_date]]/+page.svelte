<script lang="ts">
  import { toast } from "@zerodevx/svelte-toast";
  import Picker from "$lib/../components/Picker.svelte";
  import { goto, invalidateAll } from "$app/navigation";
  import View from "$lib/../components/View.svelte";
  import Help from "$lib/../components/Help.svelte";
  import { iso_date, render_tags, get_iso_week } from "$lib/util";
  export let data;
  let viewComponent;
  let overlay;
  let start_date = iso_date(data.start_date);
  let stop_date = iso_date(data.stop_date);

  let help_entries = [{ key: "Esc", text: "Go back" }];

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

  async function handle_keys() {
  }

  function extract_kw(str_date) {
	let date = new Date(Date.parse(str_date));	
	return get_iso_week(date);
  }

  function is_today(str_date) {
	let today = iso_date(new Date());		
	return str_date.indexOf(today) != -1;
  }
</script>

<View bind:this={viewComponent} bind:overlay>
  <div slot="header" class="header">
      <h1>Agenda</h1>
      <input type="date" on:change={handle_change} bind:value={start_date} />
      <input type="date" on:change={handle_change} bind:value={stop_date} />
    </div>
  <div slot="content" on:keyup={handle_keys}>

  <Picker on:action={handle_action} >
    <svelte:fragment slot="entries">
      {#each data.hits as hit, index}
	   {#if index === 0 || extract_kw(hit.str_date) !== extract_kw(data.hits[index - 1].str_date)}
      <tr data-skip="true">
        <td colspan="3">KW {extract_kw(hit.str_date)}</td>
      </tr>
    {/if}
        <tr data-cmd={hit.rg.path}>
          <td class="shrink">
		  {#if is_today(hit.str_date)}
		  <b>{hit.str_date}</b>
		  {:else}
		  {hit.str_date}
		  {/if}
		  </td>
          <td class="shrink">{hit.rg.path}</td>
		  <td><div style="float:left">{hit.rg.title.replace(/<\d{4}-\d{1,2}-\d{1,2}>/, "")}</div>
		  {@html render_tags(hit.rg.tags)}</td>
        </tr>
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
  .shrink {
    width: 0.1%;
    white-space: nowrap;
  }
</style>
