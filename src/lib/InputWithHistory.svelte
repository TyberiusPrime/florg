<script>
  import { createPopper } from "@popperjs/core";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { overlay_handles_escape } from "../lib/mode_stack.ts";

  export let input_value = "";
  export let history = [];
  export let placement = "top-start";
  let selectedIndex = history.length - 1;
  let showHistory = false;

  function handleInput(event) {
    const { value } = event.target;
    input_value = value;
    if (event.key === "ArrowUp" && history.length > 0) {
      event.preventDefault();
      if (showHistory) {
        selectedIndex = Math.max(selectedIndex - 1, 0);
      } else {
        selectedIndex = history.length - 1;
      }
      showHistory = true;
      overlay_handles_escape.update((_) => false);
    } else if (event.key === "ArrowDown" && history.length > 0) {
      event.preventDefault();
      if (showHistory) {
        selectedIndex = Math.min(selectedIndex + 1, history.length - 1);
      } else {
        selectedIndex = history.length - 1;
      }
      showHistory = true;
      overlay_handles_escape.update((_) => false);
    } else if (event.key === "Enter") {
      if (showHistory && selectedIndex >= 0 && selectedIndex < history.length) {
        event.preventDefault();
        input_value = history[selectedIndex];
        showHistory = false;
        overlay_handles_escape.update((_) => true);
      } else if (input_value) {
        dispatch("accept", null);
      }
    } else if (event.key === "Escape") {
      event.preventDefault();
      if (showHistory) {
        showHistory = false;
        window.setTimeout(() => {
          overlay_handles_escape.update((_) => true);
        }, 10);
      } else {
        dispatch("leave", input_value);
      }
    } else {
      showHistory = false;
      overlay_handles_escape.update((_) => true);
    }
  }

  function popout() {
    window.setTimeout(() => {
      let el = document.getElementById("history_popcorn");
      console.log("el is", el);
      if (el != null) {
        document.getElementById("history_popcorn").style.width =
          window.getComputedStyle(
            document.getElementById("InputWithHistory")
          ).width;
        createPopper(
          document.querySelector("#InputWithHistory"),
          document.querySelector("#history_popcorn"),
          {
            placement: placement,
          }
        );
      }
    }, 10);
    return "";
  }
</script>

<svelet:element>
  {#if showHistory && history.length > 0}
    <div id="history_popcorn">
      History:
      <ul>
        {#each history as item, index}
          <li class:selected={selectedIndex === index}>{item}</li>
        {/each}
      </ul>
    </div>
    {popout()}
  {/if}

  <input
    id="InputWithHistory"
    type="text"
    placeholder="Type something..."
    value={input_value}
    on:keydown={handleInput}
    autofocus
  />
</svelet:element>

<style>
  #history_popcorn {
    border: 1px dashed grey;
    background-color: white;
    padding: 0.5em;
  }
  :global .selected {
    background-color: #bfbfff;
  }

  ul {
    list-style-type: none;
    margin-top: 0;
    margin-left: 0;
  }
  li {
    margin-left: 0;
    padding: 0.25em;
  }
</style>
