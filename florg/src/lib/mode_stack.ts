import { writable } from "svelte/store";
import { error_toast } from "./util";

let mode_stack = [{ mode: "node", args: { path: "" }, transient: false }];

let enter_mode_callbacks = {};

export let mode_store = writable(null);
export let mode_args_store = writable(null);
export let mode_transient_store = writable(null);
export let overlay_handles_escape = writable(true);

export function enter_mode(
  mode: string,
  args: any,
  transient: boolean = false,
) {
  if (mode_stack.slice(-1)[0].transient) {
    mode_stack.pop();
  }

  for (let mode in enter_mode_callbacks) {
    let updated = enter_mode_callbacks[mode](
      mode_stack[mode_stack.length - 1]["args"],
    );
    if (updated === undefined || updated === null) {
      error_toast(
        "(enter_mode) mode callback returned null, not updating mode",
      );
    } else {
      mode_stack[mode_stack.length - 1]["args"] = updated;
    }
  }
  let mm = { mode: mode, args: args, transient: transient };
  if (JSON.stringify(mode_stack.slice(-1)[0]) != JSON.stringify(mm)) {
    mode_stack.push(mm);
  }
  assign_last_mode();
}

export function leave_mode() {
  if (mode_stack.length > 1) {
    mode_stack.pop();
  }
  assign_last_mode();
}

export function assign_last_mode() {
  let m = mode_stack[mode_stack.length - 1];
  mode_store.update((_old) => m.mode);
  mode_args_store.update((_old) => m.args);
  mode_transient_store.update((_old) => m.transient);
}

export function get_last_path() {
  for (let i = mode_stack.length - 1; i >= 0; i--) {
    if (mode_stack[i].mode == "node") {
      return mode_stack[i].args.path;
    }
  }
}

export function register_enter_mode(mode: string, callback: any) {
  enter_mode_callbacks[mode] = callback;
}

export function unregister_enter_mode(mode: string) {
  delete enter_mode_callbacks[mode];
}
