import { replaceAsync } from "$lib/util";
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
    /<a target="_blank" href="#([A-Z]+)">[[A-Z]+]/g,
    async (_match, args) => {
      let path = args[0];
      let title = await invoke("get_node_title", { path });
      return `<a href="#/node/${path}">${path}:${title}`;
    },
  );

  return rendered;
}
