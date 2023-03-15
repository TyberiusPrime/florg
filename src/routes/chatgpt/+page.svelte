
<script lang="ts">
import  Picker  from '../../components/Picker.svelte';
import {
  goto,
} from '$app/navigation';
export let data;

  async function handle_action(ev) {
    let filename = ev.detail.cmd;
    if (filename == "") {
      filename = new Date().toISOString() + ".json";
    }
	goto(`/chatgpt/${filename}`);
  }


</script>

<div>
  <Picker on:action={handle_action}>
    <div slot="message"><h1>Pick a ChatGPT conversation</h1></div>
    <svelte:fragment slot="entries">
      <tr data-cmd=""><td>New conversation</td></tr>
        {#each data.convos as convo}
          <tr data-cmd={convo.filename}><td>{convo.title}</td></tr>
        {/each}
    </svelte:fragment>
  </Picker>

</div>
