import { writable } from "svelte/store";

let mode_stack = [{ mode: "node", args: { path: "" }, transient: false }];

export let mode = writable(null);
export let mode_args = writable(null);
export let mode_transient = writable(null);

export function enter_mode(mode: string, args: any, transient: boolean) {
  if (mode_stack.slice(-1)[0].transient) {
    mode_stack.pop();
  }
  let mm = { mode: mode, args: args, transient: transient };
  if (JSON.stringify(mode_stack.slice(-1)[0]) != JSON.stringify(mm)) {
    console.log("push");
    mode_stack.push(mm);
  }
  assign_last_mode();
}

export function leave_mode() {
  console.log("called leave mode");
  if (mode_stack.length > 1) {
    mode_stack.pop();
  }
  assign_last_mode();
}

export function assign_last_mode() {
  let m = mode_stack[mode_stack.length - 1];
  mode.update((_old) => m.mode);
  mode_args.update((_old) => m.args);
  mode_transient.update((_old) => m.transient);
}

export function get_last_path() {
  for (let i = mode_stack.length - 1; i >= 0; i--) {
    if (mode_stack[i].mode == "node") {
      return mode_stack[i].args.path;
    }
  }
}
