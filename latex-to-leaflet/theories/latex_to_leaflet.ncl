{
  id = "dev.panproto.latex-to-leaflet",
  description = "Map LaTeX block kinds to Leaflet block kinds",
  source = "latex",
  target = "leaflet",

  rules = [
    { "match" = { name = "section" }, replace = { name = "header" } },
    { "match" = { name = "paragraph" }, replace = { name = "text" } },
    { "match" = { name = "quote" }, replace = { name = "blockquote" } },
    { "match" = { name = "verbatim" }, replace = { name = "code" } },
    { "match" = { name = "code_listing" }, replace = { name = "code" } },
    { "match" = { name = "itemize" }, replace = { name = "unorderedList" } },
    { "match" = { name = "enumerate" }, replace = { name = "orderedList" } },
    { "match" = { name = "item" }, replace = { name = "listItem" } },
    { "match" = { name = "equation" }, replace = { name = "math" } },
    { "match" = { name = "figure" }, replace = { name = "image" } },
    { "match" = { name = "image" }, replace = { name = "image" } },
  ],
}
