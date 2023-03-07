import { readText, writeText } from "@tauri-apps/api/clipboard";

export function format_date(date: any, br = false) {
  //todo :centralize / dry
  //return date as y-md<br />h:m:s
  let pdate: Date = null;
  if (typeof date === "string") {
    pdate = new Date();
    pdate.setTime(Date.parse(date));
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

export function escape_html(html: String) {
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
