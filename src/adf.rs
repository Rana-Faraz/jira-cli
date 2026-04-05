use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde_json::{Map, Value, json};

pub fn markdown_to_adf(markdown: &str) -> Value {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let events = Parser::new_ext(markdown, options)
        .map(Event::into_static)
        .collect();

    let mut parser = AdfParser { events, pos: 0 };
    let content = parser.parse_blocks_until(None);
    json!({
        "version": 1,
        "type": "doc",
        "content": content,
    })
}

pub fn adf_to_plain_text(value: &Value) -> String {
    let mut output = String::new();
    render_adf_node(value, &mut output, 0);
    output.trim().to_owned()
}

struct AdfParser {
    events: Vec<Event<'static>>,
    pos: usize,
}

impl AdfParser {
    fn parse_blocks_until(&mut self, end: Option<TagEnd>) -> Vec<Value> {
        let mut blocks = Vec::new();

        while let Some(event) = self.peek().cloned() {
            if matches!(event, Event::End(tag_end) if Some(tag_end) == end) {
                self.pos += 1;
                break;
            }

            match event {
                Event::Start(Tag::Paragraph) => {
                    self.pos += 1;
                    let content = self.parse_inlines_until(TagEnd::Paragraph);
                    blocks.push(node_with_content("paragraph", content));
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    self.pos += 1;
                    let content = self.parse_inlines_until(TagEnd::Heading(level));
                    blocks.push(json!({
                        "type": "heading",
                        "attrs": { "level": heading_level(level) },
                        "content": content,
                    }));
                }
                Event::Start(Tag::BlockQuote(kind)) => {
                    self.pos += 1;
                    let content = self.parse_blocks_until(Some(TagEnd::BlockQuote(kind)));
                    blocks.push(node_with_content("blockquote", content));
                }
                Event::Start(Tag::List(start)) => {
                    self.pos += 1;
                    let list_end = TagEnd::List(start.is_some());
                    let items = self.parse_list_items(list_end);
                    let node = if let Some(order) = start {
                        json!({
                            "type": "orderedList",
                            "attrs": { "order": order },
                            "content": items,
                        })
                    } else {
                        json!({
                            "type": "bulletList",
                            "content": items,
                        })
                    };
                    blocks.push(node);
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    self.pos += 1;
                    blocks.push(self.parse_code_block(kind));
                }
                Event::Rule => {
                    self.pos += 1;
                    blocks.push(json!({ "type": "rule" }));
                }
                Event::Start(tag) => {
                    self.pos += 1;
                    let tag_end = tag.to_end();
                    let _ = self.parse_blocks_until(Some(tag_end));
                }
                Event::Text(text) => {
                    self.pos += 1;
                    blocks.push(node_with_content(
                        "paragraph",
                        vec![text_node(&text, Vec::new())],
                    ));
                }
                Event::Code(code) => {
                    self.pos += 1;
                    blocks.push(node_with_content(
                        "paragraph",
                        vec![text_node(&code, vec![mark("code")])],
                    ));
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.pos += 1;
                }
                Event::Html(_) | Event::InlineHtml(_) => {
                    self.pos += 1;
                }
                Event::End(_) => {
                    self.pos += 1;
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    self.pos += 1;
                    blocks.push(node_with_content(
                        "paragraph",
                        vec![text_node(&text, Vec::new())],
                    ));
                }
                Event::FootnoteReference(_) | Event::TaskListMarker(_) => {
                    self.pos += 1;
                }
            }
        }

        blocks
    }

    fn parse_list_items(&mut self, end: TagEnd) -> Vec<Value> {
        let mut items = Vec::new();

        while let Some(event) = self.peek().cloned() {
            if matches!(event, Event::End(tag_end) if tag_end == end) {
                self.pos += 1;
                break;
            }

            match event {
                Event::Start(Tag::Item) => {
                    self.pos += 1;
                    let content = self.parse_blocks_until(Some(TagEnd::Item));
                    items.push(node_with_content("listItem", content));
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        items
    }

    fn parse_code_block(&mut self, kind: CodeBlockKind<'static>) -> Value {
        let mut text = String::new();

        while let Some(event) = self.next() {
            match event {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(value)
                | Event::Code(value)
                | Event::Html(value)
                | Event::InlineHtml(value) => {
                    text.push_str(&value);
                }
                Event::SoftBreak | Event::HardBreak => text.push('\n'),
                _ => {}
            }
        }

        let mut node = Map::new();
        node.insert("type".into(), Value::String("codeBlock".into()));
        if let Some(language) = code_block_language(kind) {
            node.insert("attrs".into(), json!({ "language": language }));
        }
        node.insert(
            "content".into(),
            Value::Array(vec![text_node(&text, Vec::new())]),
        );
        Value::Object(node)
    }

    fn parse_inlines_until(&mut self, end: TagEnd) -> Vec<Value> {
        let mut inlines = Vec::new();

        while let Some(event) = self.next() {
            match event {
                Event::End(tag_end) if tag_end == end => break,
                Event::Text(text) => inlines.push(text_node(&text, Vec::new())),
                Event::Code(code) => inlines.push(text_node(&code, vec![mark("code")])),
                Event::SoftBreak => inlines.push(text_node(" ", Vec::new())),
                Event::HardBreak => inlines.push(json!({ "type": "hardBreak" })),
                Event::Start(Tag::Emphasis) => {
                    let mut nested = self.parse_inlines_until(TagEnd::Emphasis);
                    apply_mark(&mut nested, mark("em"));
                    inlines.extend(nested);
                }
                Event::Start(Tag::Strong) => {
                    let mut nested = self.parse_inlines_until(TagEnd::Strong);
                    apply_mark(&mut nested, mark("strong"));
                    inlines.extend(nested);
                }
                Event::Start(Tag::Strikethrough) => {
                    let mut nested = self.parse_inlines_until(TagEnd::Strikethrough);
                    apply_mark(&mut nested, mark("strike"));
                    inlines.extend(nested);
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let mut nested = self.parse_inlines_until(TagEnd::Link);
                    apply_mark(&mut nested, link_mark(&dest_url));
                    inlines.extend(nested);
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    let mut nested = self.parse_inlines_until(TagEnd::Image);
                    apply_mark(&mut nested, link_mark(&dest_url));
                    inlines.extend(nested);
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    inlines.push(text_node(&text, Vec::new()));
                }
                Event::Html(_) | Event::InlineHtml(_) => {}
                Event::Start(tag) => {
                    let tag_end = tag.to_end();
                    let nested = self.parse_inlines_until(tag_end);
                    inlines.extend(nested);
                }
                Event::Rule | Event::TaskListMarker(_) | Event::FootnoteReference(_) => {}
                Event::End(_) => {}
            }
        }

        inlines
    }

    fn peek(&self) -> Option<&Event<'static>> {
        self.events.get(self.pos)
    }

    fn next(&mut self) -> Option<Event<'static>> {
        let event = self.events.get(self.pos).cloned();
        if event.is_some() {
            self.pos += 1;
        }
        event
    }
}

fn node_with_content(node_type: &str, content: Vec<Value>) -> Value {
    json!({
        "type": node_type,
        "content": content,
    })
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_block_language(kind: CodeBlockKind<'static>) -> Option<String> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => info
            .split_whitespace()
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
    }
}

fn text_node(text: &str, marks: Vec<Value>) -> Value {
    let mut node = Map::new();
    node.insert("type".into(), Value::String("text".into()));
    node.insert("text".into(), Value::String(text.to_owned()));
    if !marks.is_empty() {
        node.insert("marks".into(), Value::Array(marks));
    }
    Value::Object(node)
}

fn mark(mark_type: &str) -> Value {
    json!({ "type": mark_type })
}

fn link_mark(url: &CowStr<'static>) -> Value {
    json!({
        "type": "link",
        "attrs": { "href": url.as_ref() },
    })
}

fn apply_mark(nodes: &mut [Value], mark: Value) {
    for node in nodes {
        if node_type(node) != Some("text") {
            continue;
        }

        let marks = node
            .as_object_mut()
            .expect("text nodes should be JSON objects")
            .entry("marks")
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Some(values) = marks.as_array_mut() {
            values.push(mark.clone());
        }
    }
}

fn node_type(node: &Value) -> Option<&str> {
    node.as_object()
        .and_then(|value| value.get("type"))
        .and_then(Value::as_str)
}

fn render_adf_node(node: &Value, output: &mut String, indent: usize) {
    let Some(node_type) = node_type(node) else {
        return;
    };

    match node_type {
        "doc" => render_children(node, output, indent),
        "text" => {
            if let Some(text) = node.get("text").and_then(Value::as_str) {
                output.push_str(text);
            }
        }
        "hardBreak" => output.push('\n'),
        "paragraph" => {
            render_children(node, output, indent);
            ensure_blank_line(output);
        }
        "heading" => {
            render_children(node, output, indent);
            ensure_blank_line(output);
        }
        "blockquote" => {
            let mut nested = String::new();
            render_children(node, &mut nested, indent);
            for line in nested.lines() {
                if line.trim().is_empty() {
                    output.push('\n');
                } else {
                    output.push_str("> ");
                    output.push_str(line);
                    output.push('\n');
                }
            }
            ensure_blank_line(output);
        }
        "bulletList" => render_list(node, output, indent, None),
        "orderedList" => {
            let start = node
                .get("attrs")
                .and_then(|attrs| attrs.get("order"))
                .and_then(Value::as_u64)
                .unwrap_or(1);
            render_list(node, output, indent, Some(start));
        }
        "listItem" => render_children(node, output, indent),
        "codeBlock" => {
            if let Some(content) = node.get("content").and_then(Value::as_array) {
                let text = content
                    .iter()
                    .filter_map(|child| child.get("text").and_then(Value::as_str))
                    .collect::<String>();
                output.push_str(&text);
            }
            ensure_blank_line(output);
        }
        "rule" => {
            output.push_str("---");
            ensure_blank_line(output);
        }
        _ => render_children(node, output, indent),
    }
}

fn render_children(node: &Value, output: &mut String, indent: usize) {
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            render_adf_node(child, output, indent);
        }
    }
}

fn render_list(node: &Value, output: &mut String, indent: usize, ordered_start: Option<u64>) {
    let Some(items) = node.get("content").and_then(Value::as_array) else {
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let marker = match ordered_start {
            Some(start) => format!("{}.", start + index as u64),
            None => "-".to_owned(),
        };

        let mut nested = String::new();
        render_children(item, &mut nested, indent + 2);
        let lines: Vec<&str> = nested.lines().collect();
        if lines.is_empty() {
            output.push_str(&" ".repeat(indent));
            output.push_str(&marker);
            output.push('\n');
            continue;
        }

        let prefix = format!("{}{} ", " ".repeat(indent), marker);
        let continuation = " ".repeat(prefix.len());
        for (line_index, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                output.push('\n');
                continue;
            }
            if line_index == 0 {
                output.push_str(&prefix);
            } else {
                output.push_str(&continuation);
            }
            output.push_str(line);
            output.push('\n');
        }
    }
    ensure_blank_line(output);
}

fn ensure_blank_line(output: &mut String) {
    while output.ends_with('\n') {
        output.pop();
    }
    output.push_str("\n\n");
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{adf_to_plain_text, markdown_to_adf};

    #[test]
    fn converts_common_markdown_subset_to_adf() {
        let doc = markdown_to_adf(
            "# Title\n\nA **bold** [link](https://example.com)\n\n- one\n- two\n\n```rust\nfn main() {}\n```",
        );

        let content = doc["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], Value::String("heading".into()));
        assert_eq!(content[1]["type"], Value::String("paragraph".into()));
        assert_eq!(content[2]["type"], Value::String("bulletList".into()));
        assert_eq!(content[3]["type"], Value::String("codeBlock".into()));
    }

    #[test]
    fn converts_hard_breaks() {
        let doc = markdown_to_adf("line one  \nline two");
        let paragraph = &doc["content"][0]["content"];
        assert!(
            paragraph
                .as_array()
                .unwrap()
                .iter()
                .any(|node| node["type"] == Value::String("hardBreak".into()))
        );
    }

    #[test]
    fn renders_adf_back_to_plain_text() {
        let doc = markdown_to_adf("# Title\n\n- one\n- two\n\n`code`");
        let text = adf_to_plain_text(&doc);
        assert!(text.contains("Title"));
        assert!(text.contains("- one"));
        assert!(text.contains("- two"));
        assert!(text.contains("code"));
    }
}
