import { invoke } from "@tauri-apps/api/tauri";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
  let query = params.query;
  let res = {
    query: query,
    tags: await invoke("mail_get_tags", {}),
    messages: [],
    more_mail: false,
    mode: "",
	focused: 0,
  };
  if (query.startsWith("thread:")) {
    res.mode = "messages";
    let t: [[object], boolean] = await invoke("query_mail", { query: query });
    let threads = t[0];
    let messages = [];
    for (let thread of threads) {
      for (let msg of thread.messages) {
        messages.push(msg);
      }
    }
    messages.reverse();
    res.messages = messages;
    res.more_mail = t[1];
  } else {
    res.mode = "threads";
    let t = await invoke("query_mail", { query: query });
    res.messages = t[0];
    res.more_mail = t[1];
  }

  return res;
}
