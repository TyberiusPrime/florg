<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
</script>

<div class="header">
  <table>
    <tr><td>A <a href=""> hello</a></td></tr>
    <tr
      ><td
        ><a
          href="http://www.heise.de"
          target="_blank"
          rel="noreferrer"
		><span class="hotkey">B</span>&nbsp;&nbsp; hello</a
        ></td
      ></tr
    >
  </table>
  <hr />
</div>

<style>
  .header {
/*    position: fixed;
    top: 0;
    width: 100%;
	*/
  }

  td {
    font-family: JetBrains Mono, monospace, monospace;
    text-color: red;
    font-weight: bold;
    text-align: left;
  }
</style>
