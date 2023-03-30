<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { iso_date } from "$lib/util.ts";
  const dispatch = createEventDispatcher();

  export let value = iso_format(Date.now());
  export let valid = false;

  function sameDayLastMonth(date) {
    const year = date.getFullYear();
    const month = date.getMonth() - 1;
    const day = date.getDate();

    // If the current month is January, subtract one year and set the month to December
    if (month < 0) {
      return new Date(year - 1, 11, day);
    } else {
      return new Date(year, month, day);
    }
  }
  function sameDayNextMonth(date) {
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const day = date.getDate();
    const daysInNextMonth = new Date(year, month, 0).getDate();

    if (day > daysInNextMonth) {
      // If the day of the current month is greater than the number of days in the next month,
      // set the day to the last day of the next month
      return new Date(year, month, daysInNextMonth);
    } else {
      return new Date(year, month, day);
    }
  }

  function handle_key_down(ev) {
    let timestamp = Date.parse(value);
    valid = timestamp != null;
    let date = null;
    if (valid) {
      date = new Date(timestamp);
    }
    let offset = null;
    if (ev.key == "ArrowUp") {
      if (ev.shiftKey) {
        let sd = sameDayLastMonth(date);
        offset = (sd - date) / (24 * 60 * 60 * 1000);
      } else {
        offset = -7;
      }
    } else if (ev.key == "ArrowDown") {
      if (ev.shiftKey) {
        let sd = sameDayNextMonth(date);
        offset = (sd - date) / (24 * 60 * 60 * 1000);
      } else {
        offset = 7;
      }
    } else if (ev.key == "ArrowLeft") {
      offset = -1;
    } else if (ev.key == "ArrowRight") {
      offset = 1;
    }

    if (valid && offset != null) {
      timestamp = timestamp + offset * 24 * 60 * 60 * 1000;
      value = iso_date(timestamp);
      ev.preventDefault();
      ev.stopPropagation();
    }
	if (valid) {
        dispatch("change", value);
		
	}
  }

  function handle_key_up(ev) {
    if (ev.key == "Enter") {
      if (valid) {
        dispatch("action", value);
        ev.preventDefault();
        ev.stopPropagation();
      }
    } else if (ev.key == " ") {
      if (valid) {
        ev.preventDefault();
        ev.stopPropagation();
        dispatch("space_action", value);
      }
    } else if (ev.key == "Escape") {
      {
        ev.preventDefault();
        ev.stopPropagation();
        dispatch("leave", value);
      }
    }
  }
</script>

<input
  type="text"
  bind:value
  on:keydown={handle_key_down}
  on:keyup={handle_key_up}
/>
