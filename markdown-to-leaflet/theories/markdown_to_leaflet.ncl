{
  id = "dev.panproto.markdown-to-leaflet",
  description = "Map Markdown block kinds to Leaflet block kinds",
  source = "markdown",
  target = "leaflet",

  rules = [
    { "match" = { name = "heading" }, replace = { name = "header" } },
    { "match" = { name = "paragraph" }, replace = { name = "text" } },
    { "match" = { name = "blockquote" }, replace = { name = "blockquote" } },
    { "match" = { name = "code_block" }, replace = { name = "code" } },
    { "match" = { name = "ordered_list" }, replace = { name = "orderedList" } },
    { "match" = { name = "unordered_list" }, replace = { name = "unorderedList" } },
    { "match" = { name = "list_item" }, replace = { name = "listItem" } },
    { "match" = { name = "thematic_break" }, replace = { name = "horizontalRule" } },
    { "match" = { name = "image" }, replace = { name = "image" } },
    { "match" = { name = "math_block" }, replace = { name = "math" } },
  ],
}
