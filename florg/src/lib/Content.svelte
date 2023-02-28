<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  export let text = "content...";
</script>
<div id="the_content">
<pre>
{text}
</pre>
<span id="bottom_of_content" />
</div>

<style>
p { 
margin-top:0px;
}
</style>
