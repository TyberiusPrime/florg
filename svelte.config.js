import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: preprocess(),

  kit: {
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html",
    }),
    prerender: {
      entries: [
        "*",
        "/agenda/[[start_date]]/[[stop_date]]",
        "/chatgpt/[filename]",
        "/mail/message/[id]",
        "/mail/query/[query]",
        "/nav/[[path]]",
        "/node_search/[query]/[[path]]",
        "/node/[[path]]",
        "/tree/[[path]",
      ],
    },
  },
};

export default config;
