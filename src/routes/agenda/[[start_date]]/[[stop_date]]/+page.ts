import { invoke } from "@tauri-apps/api/tauri";

export async function load({ params }: { params: any }) {
  let start_date = new Date(parseInt(params.start_date));
  let stop_date = new Date(parseInt(params.stop_date));

  let rg_results = await invoke("ripgrep_below_node", {
    queryPath: "",
    searchTerm: "<\\d{4}-\\d{1,2}-\\d{1,2}( \\d{2}:\\d{2})?>",
    onlyMatching: true,
  });

  let hits = [];
  for (let ii = 0; ii < rg_results.length; ii++) {
    let rg_result = rg_results[ii];
    for (let ll = 0; ll < rg_result.lines.length; ll++) {
      let str_date = rg_result.lines[ll][1].slice(1, -1);
      let date = new Date(str_date);
      if ((start_date <= date) && (date <= stop_date)) {
        hits.push({
          str_date,
          date,
          rg: rg_results[ii],
        });
      }
    }
	hits.sort((a, b) => a.date - b.date);
  }

  return {
    start_date: start_date,
    stop_date: stop_date,
    hits: hits,
  };
}
