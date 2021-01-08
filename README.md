# markdown-heading-id

Filter for the [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) of crate [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark)

This crate provides a filter of [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) which converts headings with custom ID into HTML.
It uses the syntax of headings IDs defined in [Extended Syntax of Markdown](https://www.markdownguide.org/extended-syntax/#heading-ids).

For example, if we have the following fragment of Markdown
```rust
## Heading {#heading-id}
```
then it is converted into a fragment of HTML below:
```rust
<h2 id="heading-id">Heading</h2>
```

### Usage

It is easy to use a filter provided by this crate.
`HeadingId` wraps an instance of [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) and it can be passed to [`push_html`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/html/fn.push_html.html) or [`write_html`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/html/fn.write_html.html),
because `HeadingId` implements the trait `Iterator<Item=Event<'a>>`.
An example is given below:
```rust
use pulldown_cmark::Parser;
use pulldown_cmark::html::push_html;
use markdown_heading_id::HeadingId;

let parser = Parser::new("## Heading {#heading-id}");
let parser = HeadingId::new(parser);
let mut buf = String::new();
push_html(&mut buf, parser);
assert_eq!(buf.trim_end(), r#"<h2 id="heading-id">Heading</h2>"#);
```

License: MIT
