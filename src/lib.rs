//! Filter for the [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) of crate [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark)
//!
//! This crate provides a filter of [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) which converts headings with custom ID into HTML.
//! It uses the syntax of headings IDs defined in [Extended Syntax of Markdown](https://www.markdownguide.org/extended-syntax/#heading-ids).
//!
//! For example, if we have the following fragment of Markdown
//! ```ignore
//! ## Heading {#heading-id}
//! ```
//! then it is converted into a fragment of HTML below:
//! ```ignore
//! <h2 id="heading-id">Heading</h2>
//! ```
//!
//! ## Usage
//!
//! It is easy to use a filter provided by this crate.
//! `HeadingId` wraps an instance of [`Parser`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/struct.Parser.html) and it can be passed to [`push_html`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/html/fn.push_html.html) or [`write_html`](https://docs.rs/pulldown-cmark/0.8.0/pulldown_cmark/html/fn.write_html.html),
//! because `HeadingId` implements the trait `Iterator<Item=Event<'a>>`.
//! An example is given below:
//! ```
//! use pulldown_cmark::Parser;
//! use pulldown_cmark::html::push_html;
//! use markdown_heading_id::HeadingId;
//!
//! let parser = Parser::new("## Heading {#heading-id}");
//! let parser = HeadingId::new(parser);
//! let mut buf = String::new();
//! push_html(&mut buf, parser);
//! assert_eq!(buf.trim_end(), r#"<h2 id="heading-id">Heading</h2>"#);
//! ```

use std::marker::PhantomData;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark::escape::{StrWrite, escape_html, escape_href};
use pulldown_cmark::html::push_html;

fn find_custom_id(s: &str) -> (&str, Option<&str>) {
    let (before_brace, after_brace) = match s.find("{#") {
        Some(pos) => (&s[..pos], &s[pos+2..]),
        None => return (s, None),
    };

    let (inner_brace, _after_brace) = match after_brace.find('}') {
        Some(pos) => (&after_brace[..pos], &after_brace[pos+1..]),
        None => return (s, None),
    };

    (before_brace.trim_end(), Some(inner_brace))
}

/// Converts headings with ID into HTML
///
/// An iterator `HeadingId` converts a heading with ID into an HTML event.
/// Heading IDs are written in an [extended syntax](https://www.markdownguide.org/extended-syntax/#heading-ids) of Markdown.
/// This iterator acts as a filter of the `Parser` of `pulldown-cmark`.
/// `Event`s between a start of `Tag::Heading` and end thereof are converted into one
/// `Event::HTML`.
/// It buffers those events because the heading id is positioned at the tail of heading line.
pub struct HeadingId<'a, P> {
    parser: P,
    _marker: PhantomData<&'a P>,
}

impl<'a, P> HeadingId<'a, P>
where
    P: Iterator<Item=Event<'a>>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser: parser,
            _marker: PhantomData,
        }
    }

    fn convert_heading(&mut self, level: u32) -> Event<'a> {
        // Read events until the end of heading comes.
        let mut buffer = Vec::new();

        while let Some(event) = self.parser.next() {
            match event {
                Event::End(Tag::Heading(n)) if n == level => break,
                _ => {},
            }
            buffer.push(event.clone());
        }

        // Convert the events into an HTML
        let mut html = String::new();
        let mut start_tag = String::new();

        if let Some((last, events)) = buffer.split_last() {
            push_html(&mut html, events.iter().cloned());

            match last {
                Event::Text(text) => {
                    let (text, id) = find_custom_id(text);
                    escape_html(&mut html, text).unwrap();

                    if let Some(id) = id {
                        write!(&mut start_tag, "<h{} id=\"", level).unwrap();
                        escape_href(&mut start_tag, id).unwrap();
                        write!(&mut start_tag, "\">").unwrap();
                    } else {
                        write!(&mut start_tag, "<h{}>", level).unwrap();
                    }
                },
                event => {
                    push_html(&mut html, vec![event.clone()].into_iter());
                },
            }
        } else {
            write!(&mut start_tag, "<h{}>", level).unwrap();
        }

        writeln!(&mut html, "</h{}>", level).unwrap();

        start_tag += &html;
        let html = start_tag;
        
        Event::Html(html.into())
    }
}

impl<'a, P> Iterator for HeadingId<'a, P>
where
    P: Iterator<Item=Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.next() {
            Some(Event::Start(Tag::Heading(level))) => Some(self.convert_heading(level)),
            Some(event) => Some(event),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pulldown_cmark::Parser;

    fn convert(s: &str) -> String {
        let mut buf = String::new();
        let parser = Parser::new(s);
        let parser = HeadingId::new(parser);
        pulldown_cmark::html::push_html(&mut buf, parser);
        buf
    }

    #[test]
    fn heading_id() {
        let s = "## Heading {#heading-id}";
        assert_eq!(convert(s).trim_end(), r#"<h2 id="heading-id">Heading</h2>"#);
    }

    #[test]
    fn normal() {
        let s = "## Heading";
        assert_eq!(convert(s).trim_end(), r#"<h2>Heading</h2>"#);
    }

    #[test]
    fn inline_code() {
        let s = "# `source code` heading {#source}";
        assert_eq!(convert(s).trim_end(),
            r#"<h1 id="source"><code>source code</code> heading</h1>"#);
    }

    #[test]
    fn em_strong() {
        let s = "## *Italic* __BOLD__ heading {#italic-bold}";
        assert_eq!(convert(s).trim_end(),
            r#"<h2 id="italic-bold"><em>Italic</em> <strong>BOLD</strong> heading</h2>"#);
    }

    #[test]
    fn whitespace() {
        let s = "## ID with space {#id with space}";
        assert_eq!(convert(s).trim_end(),
            r#"<h2 id="id%20with%20space">ID with space</h2>"#);
    }

    #[test]
    fn empty() {
        assert_eq!(convert("##").trim_end(), "<h2></h2>");
    }

    #[test]
    fn with_link() {
        let s = "### [Link](https://example.com/) {#example}";
        assert_eq!(convert(s).trim_end(),
            r#"<h3 id="example"><a href="https://example.com/">Link</a></h3>"#);
    }

    #[test]
    fn to_be_escaped() {
        let s = "## ><";
        assert_eq!(convert(s).trim_end(), "<h2>&gt;&lt;</h2>");
    }
}
