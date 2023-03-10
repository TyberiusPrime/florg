<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import asciidoctor from "asciidoctor";
  import hljs from "highlight.js";
  import "../styles/highlight.js/github.css";
  import { onMount, onDestroy } from "svelte";
  import { add_code_clipboards } from "./util.ts";

  // {@html Asciidoctor.convert(text)}

  export let rendered = "content...";

  function apply_mods(_ignored) {
    console.log("apply mods");
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
    });
    add_code_clipboards();
    return "";
  }

  onMount(() => {
  });
</script>

<div id="the_content">
  {@html rendered}
  <span id="bottom_of_content" />
  {apply_mods(rendered)}
</div>

<style>
  #the_content {
    margin: 0px;
    padding: 0px;
  }
</style>
