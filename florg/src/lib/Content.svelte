<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import asciidoctor from "asciidoctor";
  import hljs from "highlight.js";
  import { onMount, onDestroy } from "svelte";
  import '../..//dist/highlight.js/styles/github.css'
  // {@html Asciidoctor.convert(text)}

  export let rendered = "content...";

  onMount(async () => {
    console.log("hh");
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
      console.log(el);
    });
  });
</script>

<div id="the_content">
  {@html rendered}
  <span id="bottom_of_content" />
</div>

<style>
  #the_content {
    margin: 0px;
    padding: 0px;
  }
</style>
