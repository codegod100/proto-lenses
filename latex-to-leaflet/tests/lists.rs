mod common;
use common::*;

#[test]
fn itemize_two_items() {
    let json = convert(r#"
\begin{document}
\begin{itemize}
\item First
\item Second
\end{itemize}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 1);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.unorderedList");
    let items = bs[0]["block"]["children"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["$type"], "pub.leaflet.blocks.unorderedList#listItem");
    assert_eq!(items[1]["$type"], "pub.leaflet.blocks.unorderedList#listItem");
}

#[test]
fn enumerate_three_items() {
    let json = convert(r#"
\begin{document}
\begin{enumerate}
\item One
\item Two
\item Three
\end{enumerate}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.orderedList");
    let items = bs[0]["block"]["children"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(bs[0]["block"]["startIndex"], 1);
}

#[test]
fn no_text_bleeding_between_items() {
    let json = convert(r#"
\begin{document}
\begin{itemize}
\item Alpha
\item Beta
\end{itemize}
Text after.
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(bs.len(), 2); // list + trailing text
    let items = bs[0]["block"]["children"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let first = items[0]["content"]["block"]["plaintext"].as_str().unwrap_or("");
    let second = items[1]["content"]["block"]["plaintext"].as_str().unwrap_or("");
    assert_eq!(first, "Alpha", "first item wrong");
    assert_eq!(second, "Beta", "second item wrong");
    assert!(
        !first.contains("Beta"),
        "text bled from second item into first: {first}"
    );
    assert!(
        !second.contains("Alpha"),
        "text bled from first item into second: {second}"
    );
    assert_eq!(plaintext(&bs[1]), "Text after.");
}

#[test]
fn nested_itemize() {
    let json = convert(r#"
\begin{document}
\begin{itemize}
\item Outer
  \begin{itemize}
  \item Inner
  \end{itemize}
\end{itemize}
\end{document}
"#);
    let bs = blocks(&json);
    assert_eq!(block_type(&bs[0]), "pub.leaflet.blocks.unorderedList");
    let items = bs[0]["block"]["children"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    // Inner list should appear as a child block of the outer item.
    // For now just assert the outer structure is valid.
    assert_eq!(items[0]["$type"], "pub.leaflet.blocks.unorderedList#listItem");
}
