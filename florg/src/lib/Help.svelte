<script lang="ts">
export let entries;
</script>

<div>
  <table>
  Help<br />
  {#each entries as entry }
	<tr><td><span class="hotkey">{entry.key}</span></td><td>{entry.text}</td></tr>
  {/each}
  </table>
</div>

<style>
 </style>
