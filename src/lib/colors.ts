
export function tag_class(tag: string) {
  let first_letter = tag.slice(0, 1);
  let index = first_letter.charCodeAt(0) - 97;
  let no = index % 8;
  return "tag_" + no;
}
