use crate::ast::AstNode;

pub fn parse(source: &str) -> Result<Vec<AstNode>, String> {
    let mut lines = source.lines().enumerate().peekable();
    parse_block(&mut lines)
}

fn parse_block<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<Vec<AstNode>, String>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut nodes = Vec::new();

    while let Some((_, line)) = lines.peek() {
        let line = line.trim();

        if line.is_empty() {
            lines.next();
            continue;
        }

        if line == "}" {
            lines.next();
            break;
        }

        nodes.push(parse_line(lines)?);
    }

    Ok(nodes)
}

fn parse_line<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<AstNode, String>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let (_, raw) = lines.next().unwrap();
    let line = raw.trim();

    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts[0] {
        "function" => {
            let name = parts[1].to_string();
            expect_brace(raw, lines)?;
            let body = parse_block(lines)?;
            Ok(AstNode::Function { name, body })
        }

        "unsafe" => {
            expect_brace(raw,lines)?;
            let body = parse_block(lines)?;
            Ok(AstNode::Unsafe { body })
        }

        "region" => Ok(AstNode::Region {
            kind: parts[1].to_string(),
            name: parts[2].to_string(),
        }),

        "lifetime" => Ok(AstNode::Lifetime {
            name: parts[1].to_string(),
            scope: parts[4].to_string(),
        }),

        "let" => Ok(AstNode::Let {
            name: parts[1].to_string(),
            region: parts[3].to_string(),
        }),

        "capability" => Ok(AstNode::Capability {
            kind: parts[1].to_string(),
            value: parts[2].to_string(),
            lifetime: parts[4].to_string(),
        }),

        "drop" => Ok(AstNode::Drop {
            value: parts[1].to_string(),
        }),

        "assume" => Ok(AstNode::Assume {
            text: raw.split('"').nth(1).unwrap().to_string(),
        }),

        _ => Err(format!("Unknown syntax: {}", raw)),
    }
}

fn expect_brace<'a, I>(
    current_line: &str,
    lines: &mut std::iter::Peekable<I>,
) -> Result<(), String>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    // Case 1: brace is on the same line
    if current_line.trim_end().ends_with('{') {
        return Ok(());
    }

    // Case 2: brace is on the next line
    while let Some((_, next)) = lines.next() {
        let trimmed = next.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "{" {
            return Ok(());
        }
        return Err("Expected '{'".into());
    }

    Err("Expected '{'".into())
}

