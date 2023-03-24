import { invoke } from "@tauri-apps/api/tauri";
import html2plaintext from "html2plaintext";

import { removeItemOnce } from "../../../../lib/util";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
  let mail_id = params.id;
  let msg = await invoke("get_mail_message", {
    id: mail_id,
  });
  let tags = msg.tags;

  if (tags.indexOf("unread") > -1) {
    console.log("unread");
    await invoke("mail_message_remove_tags", {
      id: mail_id,
      tags: ["unread"],
    });
    removeItemOnce(tags, "unread");
  }
  let parsed = JSON.parse(msg.json);
  let headers = parsed.parts[0].headers;
  let text = "";
  for (let ii = 0; ii < parsed.parts.length; ii++) {
	if ("Text" in parsed.parts[ii].body){
	  text += parsed.parts[ii].body.Text;
	}
  }
  let html = "";
  for (let ii = 0; ii < parsed.parts.length; ii++) {
	if ("Html" in parsed.parts[ii].body){
	  html += parsed.parts[ii].body.Html;
	}
  }
  //console.log(msg.json);
  let res = {
    id: mail_id,
    parsed: parsed,
	headers: headers,
    tags: tags,
    filename: msg.filename,
    available_tags: await invoke("mail_get_tags", {}),
	text: text,
	html: html,
  };

  return res;
}
