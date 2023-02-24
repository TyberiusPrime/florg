<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import TopTree from "./lib/TopTree.svelte";
  import Content from "./lib/Content.svelte";
  import Footer from "./lib/Footer.svelte";
  import * as KeyPress from "../dist/keypress-2.1.5.min.js";

  function handleKeypress(event) {
    console.log(event);
  }
  let show_help = false;
  let mode = "nomal";
  let footer_msg = "";

  var listener_normal = new window.keypress.Listener();
  listener_normal.reset();
  listener_normal.stop_listening();

  var listener_nav = new window.keypress.Listener();
  listener_nav.stop_listening();

  listener_normal.simple_combo("f", (e, count, repeated) => {
    console.log("entering nav mode");
    listener_normal.stop_listening();
    listener_nav.listen();
	footer_msg = "Nav mode activated. <span class='hotkey'>Escape</span> to abort. <span class='hotkey'>Space</span> to start at root";
    mode = "nav";
  });
  listener_normal.register_combo({
    keys: "h",
    is_unordered: true,
    on_keydown: (e, count, repeated) => {
      if (!repeated) {
        console.log("help mode");
        show_help = !show_help;
      }
    },
  });

  listener_nav.simple_combo("esc", () => {
    mode = "normal";
    footer_msg = "";
	listener_nav.stop_listening();
	listener_normal.listen();
  });

  listener_normal.listen();
</script>

<svelte:window />

<div class="wrapper">
  <div class="header"><TopTree /></div>
  <!-- <div class="sidebar-1">
    <div class="sticky-spacer" />
    <div class="sticky-content">Sidebar 1 Absolute position, Fixed width</div>
  </div> -->
  <div class="content">
    <div class="sticky-spacer" />
    <div class="sticky-content">
      <Content />
      end of content
    </div>
  </div>
  <!-- <div class="sidebar-2">
    <div class="sticky-spacer" />
    <div class="sticky-content" />
  </div> -->
  <div class="footer">
    <Footer bind:show_help bind:mode bind:msg={footer_msg} />
  </div>
</div>

<!-- <header> <TopTree /> </header>
  <div class="content">
  <Content />
   </div>
<footer> <Footer bind:show_help /></footer>
  -->
<style>
</style>
