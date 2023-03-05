<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import { createEventDispatcher } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import * as KeyPress from "../js/keypress-2.1.5.min.js";
  import { Fzf, byLengthAsc } from "fzf";

  import PickerTable from "./PickerTable.svelte";

  export let downstream_elements = [];
  export let focused = 0;
  export let view_mode = "threads";

  let unread_count;

  const dispatch = createEventDispatcher();

  const many_cat_colors = [
    "#1C86EE",
    "#E31A1C", // red
    "#008B00",
    "#6A3D9A", // purple
    "#FF7F00", // orange
    "#4D4D4D",
    "#FFD700",
    "#7EC0EE",
    "#FB9A99", // lt pink
    "#90EE90",
    "#FDBF6F", // lt orange
    "#B3B3B3",
    "#EEE685",
    "#B03060",
    "#FF83FA",
    "#FF1493",
    "#00008F",
    "#36648B",
    "#00CED1",
    "#008F00",
    "#8B8B00",
    "#CDCD00",
    "#A52A2A",
  ];

  function find_color(tag) {
    let first_letter = tag.slice(0, 1);
    let index = first_letter.charCodeAt(0) - 97;
    return many_cat_colors[index % many_cat_colors.length];
  }

  function latest_date(entry) {
    //go through all the messages, extract date, keep the largest one
    let latest = 0;
    for (let i = 0; i < entry.messages.length; i++) {
      let date = Date.parse(entry.messages[i].date);
      if (date > latest) {
        latest = date;
      }
    }
    return latest;
  }
  function format_date(date) {
    //return date as y-md<br />h:m:s
    let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(date);
    let mo = new Intl.DateTimeFormat("en", { month: "2-digit" }).format(date);
    let da = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
    let hh = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
    let mm = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
    return `${ye}&#8209;${mo}&#8209;${da}<br />${hh}:${mm}`;
  }
  function count_unread(entry) {
    let counter = 0;
    for (let i = 0; i < entry.messages.length; i++) {
      if (entry.messages[i].tags.indexOf("unread") > -1) {
        counter++;
      }
    }
    return counter;
  }

  function focus_filter() {
    document.getElementById("typebox").focus();
  }

  function focus_node(index) {
    if (index) {
      focused = index;
    }
    focus_filter();
  }

  function is_unread(msg) {
    return msg.tags.indexOf("unread") > -1;
  }

  function handle_double_click(ev) {
  console.log("double click");
        if (view_mode == "threads") {
        if (downstream_elements[focused].messages.length == 1) {
          dispatch("action", {
            id: downstream_elements[focused].messages[0].id,
            single_message: true,
          });
        } else {
          dispatch("action", {
            id: downstream_elements[focused].id,
            single_message: false,
          });
        }
      } else if (view_mode == "messages") {
        dispatch("action", {
          id: downstream_elements[focused].id,
          single_message: true,
        });
      }

  }
</script>

<div>
  <div style="overflow:scroll">
    <table id="mail_pick_table">
      {#if downstream_elements.length == 0}
        nothing found.
      {/if}
      {#if view_mode == "threads"}
        {#each downstream_elements as el, index}
          <tr
            class="{index == focused ? 'chosen' : ''}  "
            on:click={(e) => focus_node(index)}
            on:dblclick={handle_double_click}
          >
            <td class="index">{index}</td>
            <td class="unread_count">
              {#if count_unread(el) > 0}
                <span class="new">{count_unread(el)}/{el.messages.length}</span>
              {:else}
                {count_unread(el)}/{el.messages.length}
              {/if}
            </td>
            <td class="date">{@html format_date(latest_date(el))}</td>
            <td>
              <div class="fromsubject">
                <div class="subject {count_unread(el) > 0 ? 'new' : ''}">
                  {el.subject}
                </div>
                <div class="from">{el.authors}</div>
              </div>
              {#each el.tags as tag}
                <div class="tags" style="background-color:{find_color(tag)}">
                  {tag}
                </div>
              {/each}
            </td>
          </tr>
        {/each}
      {:else if view_mode == "messages"}
        {#each downstream_elements as el, index}
          <tr
            class="{index == focused ? 'chosen' : ''}  "
            on:click={(e) => focus_node(index)}
            on:dblclick={handle_double_click}
          >
			<td class="date">{@html format_date(Date.parse(el.date))}</td>
            <td>
              <div class="fromsubject">
                <div class="subject {is_unread(el) > 0 ? 'new' : ''}">
                  {el.subject}
                </div>
                <div class="from">{el.from}</div>
              </div>
              {#each el.tags as tag}
                <div class="tags" style="background-color:{find_color(tag)}">
                  {tag}
                </div>
              {/each}
            </td></tr
          >
        {/each}
      {/if}
    </table>
  </div>
</div>

<style>
  input {
    width: 50%;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }

  td,
  th {
    vertical-align: top;
    padding-right: 0.5em;
    border-bottom: 1px solid #ddd;
    padding-top: 5px;
  }

  .date {
    padding-top: 5px;
    text-align: left;
    font-size: 0.8em;
  }
  .index {
    width: 0.5em;
    font-size: 0.7em;
  }
  .unread_count {
    font-size: 0.7em;
  }

  td div {
    padding: 0px;
    margin: 0px;
  }

  .from {
    font-size: 14px;
    color: #555;
  }

  .subject {
    font-size: 1em;
    color: #111;
  }
  .fromsubject {
    float: left;
    padding-right: 1em;
  }

  .tags {
    display: inline-block;
    margin-left: 5px;
    padding: 3px 8px;
    border-radius: 10px;
    font-size: 12px;
    font-weight: bold;
    color: #eee;
    background-color: #3b9cff;
    float: left;
  }
  .new {
    font-weight: bold;
  }

  .chosen {
    background-color: #bfbfff;
  }
  .chosen:nth-child(odd) {
    background-color: #bfbfff;
  }

  tr:nth-child(odd) {
    background-color: #feffee;
  }
</style>
