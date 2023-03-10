const many_cat_colors = [
  //todO: combine with the one in MailMessages
  "#1C86EE",
  "#E31A1C", // red
  "#008B00",
  "#6A3D9A", // purple
  "#FF7F00", // orange
  "#4D4D4D",
  "#FFD700",
  "#7EC0EE",
  "#FB9A99", // lt pink
  "#90EE90",
  "#FDBF6F", // lt orange
  "#B3B3B3",
  "#EEE685",
  "#B03060",
  "#FF83FA",
  "#FF1493",
  "#00008F",
  "#36648B",
  "#00CED1",
  "#008F00",
  "#8B8B00",
  "#CDCD00",
  "#A52A2A",
];

export function find_color(tag: string) {
  let first_letter = tag.slice(0, 1);
  let index = first_letter.charCodeAt(0) - 97;
  return many_cat_colors[index % many_cat_colors.length];
}
