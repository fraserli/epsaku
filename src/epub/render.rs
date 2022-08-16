use crossterm::style::{ContentStyle, Stylize};
use roxmltree::{Node, NodeType};

#[derive(Default, Copy, Clone)]
pub struct RenderAttributes {
    body: bool,
    paragraph: bool,
    link: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    nodisplay: bool,
    heading: bool,
}

pub fn render_node(
    node: Node,
    images: &mut Vec<String>,
    mut attributes: RenderAttributes,
) -> String {
    let mut output = String::new();

    let mut newline = false;
    let mut linebreak = false;

    match node.node_type() {
        NodeType::Element => match node.tag_name().name() {
            "body" => attributes.body = true,
            "p" => {
                if !attributes.paragraph {
                    newline = true;
                }
                attributes.paragraph = true;
            }
            "div" => {
                if !attributes.paragraph {
                    newline = true;
                }
            }
            "a" => attributes.link = true,
            "b" | "strong" => attributes.bold = true,
            "i" | "em" => attributes.italic = true,
            "u" => attributes.underline = true,
            "script" | "style" => attributes.nodisplay = true,
            "br" => linebreak = true,
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if !attributes.paragraph {
                    newline = true;
                }
                attributes.heading = true;
            }
            "img" => {
                output.push_str(&format!("[IMG:{}]", images.len()).reverse().to_string());
                images.push(node.attribute("src").unwrap().to_string());
            }
            _ => {}
        },
        NodeType::Text => {
            if attributes.body
                && !attributes.nodisplay
                && (attributes.paragraph || attributes.heading)
            {
                let mut style = ContentStyle::new();

                if attributes.heading {
                    style = style.bold().reverse();
                }
                if attributes.italic {
                    style = style.italic();
                }
                if attributes.bold {
                    style = style.bold();
                }
                if attributes.underline || attributes.link {
                    style = style.underlined();
                }

                output.push_str(&style.apply(node.text().unwrap()).to_string());
            }
        }
        _ => {}
    }

    if linebreak {
        output.push('\n');
    }

    let mut buf = String::new();
    for child in node.children() {
        buf.push_str(&render_node(child, images, attributes));
    }

    if newline {
        output.push_str(buf.trim());
        if !buf.trim().is_empty() {
            output.push_str("\n\n");
        }
    } else {
        output.push_str(&buf);
    }

    output
}
