import { invoke } from "@tauri-apps/api/tauri";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
	let convo: object = await invoke("chatgpt_get_conversation", { filename: params['filename'] });
    if (convo == null) {
      convo = await invoke("chatgpt_new_conversation", {});
    }
	convo.filename = params['filename'];
	return convo;

}
