import { readText, writeText } from "@tauri-apps/api/clipboard";
import { invoke } from "@tauri-apps/api/tauri";
import { toast } from "@zerodevx/svelte-toast";

export function format_date(date: any, br = false) {
  //todo :centralize / dry
  //return date as y-md<br />h:m:s
  let pdate: Date = null;
  if (typeof date === "string") {
    pdate = new Date();
    pdate.setTime(Date.parse(date));
  } else if (typeof date == "number") {
    pdate = new Date();
    pdate.setTime(date);
  } else {
    pdate = date;
  }
  /* let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(date);
  let mo = pdate.getMonth().padStart(2,'0'); //new Intl.DateTimeFormat("en", { month: "2-digit" }).format(date);
  let da = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
  let hh = new Intl.DateTimeFormat("de", { hour: "numeric" }).format(date);
  let mm = new Intl.DateTimeFormat("en", { minute: "2-digit" }).format(date); */
  let ye = pdate.getFullYear();
  let mo = String(pdate.getMonth() + 1).padStart(2, "0");
  let da = String(pdate.getDay()).padStart(2, "0");
  let hh = String(pdate.getHours()).padStart(2, "0");
  let mm = String(pdate.getMinutes()).padStart(2, "0");

  if (br) {
    return `${ye}&#8209;${mo}&#8209;${da}<br />${hh}:${mm}`;
  } else {
    return `${ye}&#8209;${mo}&#8209;${da} ${hh}:${mm}`;
  }
}

export function add_code_clipboards() {
  for (let tag of ["code", "pre", "p"]) {
    const codeBlocks = document.getElementsByTagName(tag);

    for (let i = 0; i < codeBlocks.length; i++) {
      const copyButton = document.createElement("button");
      copyButton.classList.add("copy-button");
      copyButton.innerText = "📋";
      copyButton.onclick = async () => {
        await writeText(codeBlocks[i].innerText);
      };
      codeBlocks[i].parentElement.style.position = "relative";
      codeBlocks[i].parentElement.appendChild(copyButton);
    }
  }
}

export function escape_html(html: string) {
  return document.createElement("div").appendChild(
    document.createTextNode(html),
  ).parentNode.innerHTML;
}

export function iso_date(date: Date): String {
  let ye = new Intl.DateTimeFormat("en", { year: "numeric" }).format(date);
  let mo = new Intl.DateTimeFormat("en", { month: "2-digit" }).format(date);
  let da = new Intl.DateTimeFormat("en", { day: "2-digit" }).format(date);
  return `${ye}-${mo}-${da}`;
}
export function no_text_inputs_focused(): Boolean {
  return (
    document.activeElement.tagName != "INPUT" &&
    document.activeElement.tagName != "TEXTAREA"
  );
}
export async function get_node(path: string) {
  return await invoke("get_node", { path });
}

export function isElementInViewport(el): boolean {

  var rect = el.getBoundingClientRect();
  let header_height = document.getElementById("header").offsetHeight;
  let footer_height = document.getElementById("footer").offsetHeight;

  return (
    rect.top >= header_height &&
    rect.left >= 0 &&
    rect.bottom <=
      (window.innerHeight || document.documentElement.clientHeight) -
        footer_height &&
    rect.right <=
      (window.innerWidth ||
        document.documentElement.clientWidth) /* or $(window).width() */
  );
}

export function error_toast(msg) {
  toast.push(`<span class="error">${msg}</span>`);
}

export function removeItemOnce(arr, value) {
  var index = arr.indexOf(value);
  if (index > -1) {
    arr.splice(index, 1);
  }
  return arr;
}
export const replaceAsync = async (
  str: string,
  regex: RegExp,
  asyncFn: (match: any, ...args: any) => Promise<any>,
) => {
  const promises: Promise<any>[] = [];
  str.replace(regex, (match, ...args) => {
    promises.push(asyncFn(match, args));
    return match;
  });
  const data = await Promise.all(promises);
  return str.replace(regex, () => data.shift());
};


export function trim_eol(multi_lines: string) {
  return multi_lines.replace(/[ \t]+$/gm, "");
}
