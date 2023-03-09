<script lang="ts">
  import { mode_store } from "../lib/mode_stack.ts";

  let mode = $mode_store;
</script>

<div class="wrapper">
  <div class="header" id="header">
    <slot name="header" />
  </div>
  <div class="main_content">
    <div class="sticky-spacer" />
    <div class="sticky-content">
      <slot name="content" />
    </div>
  </div>
  <div class="footer" id="footer">
    <slot name="footer" />
    {mode}
  </div>
</div>

<style>
  :global body,
  :global .footer,
  :global .header {
    background-color: #eeeeee;
    border-bottom: 2px dashed grey;
  }
</style>
