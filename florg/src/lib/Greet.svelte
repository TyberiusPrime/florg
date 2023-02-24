<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri"

  export let name = "";
  let greetMsg = ""

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
  async function greet(){
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke("greet", { name })
	await new Promise(r => setTimeout(r, 2000));
	await sleep(2000);
  }

  async function handle(e) {
  if (e.key == "Enter") {
	  console.log(e);
	  await greet();
	  alert("hello");
  }
  }
</script>

<div>
  <div class="row">
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} on:keypress={handle}/>
    <button on:click={greet}>
      Greet
    </button>
  </div>
  <p>{greetMsg}</p>
</div>
