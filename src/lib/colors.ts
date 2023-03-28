
export function tag_class(tag: string) {
  let first_letter = tag.slice(0, 1);
  let index = first_letter.toLowerCase().charCodeAt(0) - 97;
  let no = index % 9;
  return "tag_" + no;
}
