<script lang="ts">
export let mode = "?";
export let mode_args;

</script>
<div class="wrapper">
  <div class="header" id="header" >
	<slot name="header"></slot>
  </div>
  <div class="main_content">
    <div class="sticky-spacer" />
    <div class="sticky-content" >
	<slot name="content"></slot>
	</div>
  </div>
  <div class="footer" >
	<slot name="footer"></slot>
	{mode}
  </div>
</div>



<style>
  :global body,
  :global .footer,
  :global .header {
    background-color: #eeeeee;
	border-bottom:2px dashed grey;
  }
</style>
