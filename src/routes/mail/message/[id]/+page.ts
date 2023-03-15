import { invoke } from "@tauri-apps/api/tauri";
import PostalMime from "postal-mime";
import html2plaintext from "html2plaintext";

import { removeItemOnce } from "../../../../lib/util";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
  let mail_id = params.id;
  let msg = await invoke("get_mail_message", {
    id: mail_id,
  });
  let raw_message = msg.raw;
  let tags = msg.tags;

  if (tags.indexOf("unread") > -1) {
    console.log("unread");
    await invoke("mail_message_remove_tags", {
      id: mail_id,
      tags: ["unread"],
    });
    removeItemOnce(tags, "unread");
  }
  const parser = new PostalMime();
  const email = await parser.parse(raw_message);
  let res = {
    id: mail_id,
    raw: raw_message,
    parsed: email,
    tags: tags,
    filename: msg.filename,
    available_tags: await invoke("mail_get_tags", {}),
  };

  return res;
}
