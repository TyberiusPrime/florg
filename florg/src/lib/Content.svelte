<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import asciidoctor from "asciidoctor";
  import hljs from "highlight.js";
  export let text = "content...";
  import { onMount, onDestroy } from "svelte";
  import '../..//dist/highlight.js/styles/github.css'
  let Asciidoctor = asciidoctor();
  // {@html Asciidoctor.convert(text)}

  onMount(async () => {
    console.log("hh");
    document.querySelectorAll("pre code").forEach((el) => {
      hljs.highlightElement(el);
      console.log(el);
    });
  });
</script>

<div id="the_content">
  {@html Asciidoctor.convert(text, {
    attributes: {
      doctype: "article",
      showtitle: true,
      "source-highlighter": "highlight.js",
      "highlightjs-languages": "rust, swift",
    },
  })}
  <span id="bottom_of_content" />
</div>

<style>
  #the_content {
    margin: 0px;
    padding: 0px;
  }
</style>
