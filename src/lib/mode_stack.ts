import { writable } from "svelte/store";
import { error_toast } from "./util";

//wether the overlay handles the esc button
//or there is *another* layer
export let overlay_handles_escape = writable(true);

let last_path = "";

export function set_last_path(value: string) {
  last_path = value;
}
export function get_last_path() {
  return last_path;
}

let temp_history_store = null;
export function set_temp_history(value: any) {
  temp_history_store = value;
}
export function get_temp_history() {
  return temp_history_store;
}

let mode_ignore_enter = false;
export function check_and_reset_mode_ignore_enter() {
  if (mode_ignore_enter) {
    mode_ignore_enter = false;
    return true;
  }
  return false;
}

export function set_mode_ignore_enter() {
  mode_ignore_enter = true;
}
