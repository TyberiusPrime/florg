import { replaceAsync } from "$lib/util";
import { invoke } from "@tauri-apps/api/tauri";
import asciidoctor from "asciidoctor";

export async function render_text(text: string) {
  const Asciidoctor = asciidoctor();
  let rendered: string = Asciidoctor.convert(text, {
    attributes: {
      doctype: "article",
      showtitle: true,
      "source-highlighter": "highlight.js",
      "highlightjs-languages": "rust, swift",
    },
  });
  rendered = rendered.replace(/<a/g, '<a target="_blank"');
  rendered = await replaceAsync(
    rendered,
    /<a target="_blank" href="#([^"]+)">\[[^\]]+\]/g,
    async (_match, args) => {
      let path = args[0];
      console.log("found path", path);
      if (path.startsWith("mail:")) {
        let msg = await invoke("get_mail_message_brief", { id: path.slice(5) });
        let title = "";
        if (msg == null) {
          title = "Unknown email";
        } else {
			title = `${msg.from}: ${msg.subject}`;
		}


        let id = path.slice(5);
        return `<a href="/mail/message/${id}">mail:${title}`;
      } else {
        let title = await invoke("get_node_title", { path });
        return `<a href="/node/${path}">${path}:${title}`;
      }
    },
  );
  rendered = rendered.replaceAll("<p", "<p tabIndex=0 ");

  return rendered;
}
