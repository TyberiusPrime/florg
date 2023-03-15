import { invoke } from "@tauri-apps/api/tauri";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
  let res = await invoke("chatgpt_list_conversations", {});
  return { convos: res };
}
