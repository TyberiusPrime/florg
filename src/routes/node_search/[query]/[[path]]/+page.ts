import { invoke } from "@tauri-apps/api/tauri";

/** @type {import('./$types').PageLoad} */
export async function load({ params }: { params: any }) {
  let path = params.path|| "";
  let search_term = params.query;
  console.log(params)

  console.log("ripgrep below", path);
  let rg_results = await invoke("ripgrep_below_node", {
    queryPath: path,
    searchTerm: search_term,
  });
  let translated_results = [];
  for (let result of rg_results) {
    let pretty_path = result.path;
    if (pretty_path == "") {
      pretty_path = "(root)";
    }
    let text = `${pretty_path}: <strong>${result.title}</strong>`;
    for (let pt of result.parent_titles) {
      text += " / " + pt;
    }

    text += "<br />";
    let counter = 0;
    for (let line of result.lines) {
      text += line[0] + ": " + line[1] + "<br />";
      counter += 1;
      if (counter > 7) {
        text += "...";
        break;
      }
    }
    translated_results.push({
      cmd: result.path,
      text: text,
	  tags: result.tags,
    });
  }
  return {
    search_results: translated_results,
  };
}
