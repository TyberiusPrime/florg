<script lang="ts">
  export let entries;
</script>

<div>
  Help
  <div class="dacols">
    {#each entries as entry}
      <div>
        <span class="hotkey">{entry.key}</span
        >&nbsp;{@html entry.text.replaceAll(" ", "&nbsp;")}
      </div>
    {/each}
  </div>
  done
</div>

<style>
  .dacols {
    padding: 0.5em;
    columns: 200px 3;
  }
  .dacols p {
    text-align: left;
    padding: 0;
    margin: 0;
  }
</style>
