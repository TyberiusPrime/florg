<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import * as chrono from "chrono-node";
  import Flatpickr from "svelte-flatpickr";
  import "flatpickr/dist/flatpickr.css";
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import * as KeyPress from "../../dist/keypress-2.1.5.min.js";

  export let message = "";
  export let action = "";

  let natural_date_input = "";
  let output_date = "";
  let date_input;
var listener_date = new window.keypress.Listener();
  listener_date.reset();
  listener_date.stop_listening();


  function iso_date(date: Date): String {
    let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(date);
    let mo = new Intl.DateTimeFormat("en", { month: "2-digit" }).format(date);
    let da = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
    return `${ye}-${mo}-${da}`;
  }

  function parse_text_do_date() {
    let input = natural_date_input;
    if (input == "t") {
      input = "today";
    } else if (input == "to" || input == "tom") {
      input = "tomorrow";
    } else if (input == "y") {
      input = "yesterday";
    }
    let date: Date = chrono.parseDate(input);
    if (date !== null) {
      output_date = iso_date(date);
    } else {
      output_date = "";
    }
  }

  function text_key_up(ev) {
    if (ev.key == "ArrowDown") {
      ev.preventDefault();
      if (output_date != "") {
        let date = new Date(output_date);
        date.setDate(date.getDate() - 1);
        output_date = date;
        let new_date = iso_date(date);
        natural_date_input = new_date;
      } else {
        natural_date_input = "yesterday";
        parse_text_do_date();
      }
    } else if (ev.key == "ArrowUp") {
      ev.preventDefault();
      if (output_date != "") {
        let date = output_date;
        date.setDate(date.getDate() + 1);
        output_date = date;
        let new_date = iso_date(date);
        natural_date_input = new_date;
      } else {
        natural_date_input = "tomorrow";
        parse_text_do_date();
      }
    } else if (ev.key == "Enter" && natural_date_input == "") {
      natural_date_input = "today";
      parse_text_do_date();
    } else if (ev.key == "Escape") {
		cancel_date_select();

    } else {
      parse_text_do_date();
    }
  }

  function cancel_date_select() {
      dispatch("date_chosen", { action: null });
  }

  function handleSubmit(ev) {
    if (ev !== null) {
      ev.preventDefault();
    }
    if (output_date !== null) {
      dispatch("date_chosen", { date: iso_date(output_date), action: action });
    }
  }
  const date_input_options = {
    inline: true,
    onChange(selectedDates, dateStr, instance) {
      natural_date_input = dateStr;

      let old_date = new Date();
      [...instance.calendarContainer.querySelectorAll(".flatpickr-day")].map(
        (x) =>
          x.addEventListener("mousedown", function (e) {
            let new_date = new Date();
            let diff = new_date - old_date; // in ms < 500ms => double click
            if (diff <= 500) {
              handleSubmit(null);
            }
            old_date = new_date;
          })
      );
    },
  };
</script>

<div class="form">
  {message}
  <form on:submit={handleSubmit} id="myform">
    <table>
      <tr>
        <td />
        <td>
          <Flatpickr
            options={date_input_options}
            bind:value={output_date}
            name="date"
          />
          <!-- bind:formattedValue on:change={handleChange}  -->
        </td>
      </tr>
      <tr>
        <td>
          <label for="natural_language_date"> text date:</label>
        </td>
        <td>
          <input
            id="natural_language_date"
            type="text"
            autofocus
            on:change={parse_text_do_date}
            on:keyup={text_key_up}
            on:paste={parse_text_do_date}
            on:input={parse_text_do_date}
            bind:value={natural_date_input}
          />
        </td></tr
      >
      <tr>
        <td />
        <td>
          <button on:click={cancel_date_select}>Cancel</button>
          <button type="submit" style="float:right">{action}</button>
        </td></tr
      >
    </table>
  </form>
</div>

<style>
  .form {
    text-align: left;
  }
  div.row {
    display: flex-cell;
    align-items: end;
  }
</style>
