use std::{fmt, str::FromStr};

use crate::error::{FormatEditError, Result};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsonPath {
    original: String,
    segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathSegment {
    Key(String),
    Index(i64),
    Wildcard,
}

impl JsonPath {
    pub fn root() -> Self {
        Self {
            original: "$".to_string(),
            segments: Vec::new(),
        }
    }

    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    pub(crate) fn from_segments(segments: Vec<PathSegment>) -> Self {
        let mut original = "$".to_string();
        for segment in &segments {
            original.push_str(&segment.to_string());
        }
        Self { original, segments }
    }

    pub fn is_deterministic(&self) -> bool {
        self.segments
            .iter()
            .all(|segment| !matches!(segment, PathSegment::Wildcard))
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.original)
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Key(key) if is_dot_key(key) => write!(formatter, ".{key}"),
            PathSegment::Key(key) => write!(formatter, "[{}]", quote_key(key)),
            PathSegment::Index(index) => write!(formatter, "[{index}]"),
            PathSegment::Wildcard => formatter.write_str("[*]"),
        }
    }
}

impl FromStr for JsonPath {
    type Err = FormatEditError;

    fn from_str(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Err(FormatEditError::InvalidPath(
                "path cannot be empty".to_string(),
            ));
        }

        let chars = input.chars().collect::<Vec<_>>();
        let mut cursor = 0;
        expect_char(&chars, &mut cursor, '$')?;

        let mut segments = Vec::new();
        while cursor < chars.len() {
            match chars[cursor] {
                '.' => {
                    cursor += 1;
                    if cursor >= chars.len() {
                        return Err(FormatEditError::InvalidPath(
                            "dot must be followed by a key".to_string(),
                        ));
                    }
                    if chars[cursor] == '*' {
                        cursor += 1;
                        segments.push(PathSegment::Wildcard);
                    } else {
                        let start = cursor;
                        while cursor < chars.len() && chars[cursor] != '.' && chars[cursor] != '[' {
                            cursor += 1;
                        }
                        if start == cursor {
                            return Err(FormatEditError::InvalidPath(
                                "dot must be followed by a key".to_string(),
                            ));
                        }
                        segments.push(PathSegment::Key(chars[start..cursor].iter().collect()));
                    }
                }
                '[' => {
                    cursor += 1;
                    if cursor >= chars.len() {
                        return Err(FormatEditError::InvalidPath("unclosed bracket".to_string()));
                    }

                    match chars[cursor] {
                        '\'' | '"' => {
                            let quote = chars[cursor];
                            cursor += 1;
                            let key = parse_quoted_key(&chars, &mut cursor, quote)?;
                            expect_char(&chars, &mut cursor, ']')?;
                            segments.push(PathSegment::Key(key));
                        }
                        '*' => {
                            cursor += 1;
                            expect_char(&chars, &mut cursor, ']')?;
                            segments.push(PathSegment::Wildcard);
                        }
                        _ => {
                            let start = cursor;
                            if chars[cursor] == '-' {
                                cursor += 1;
                            }
                            while cursor < chars.len() && chars[cursor].is_ascii_digit() {
                                cursor += 1;
                            }
                            if start == cursor || (chars[start] == '-' && start + 1 == cursor) {
                                return Err(FormatEditError::InvalidPath(
                                    "array index must be an integer".to_string(),
                                ));
                            }
                            let raw_index = chars[start..cursor].iter().collect::<String>();
                            expect_char(&chars, &mut cursor, ']')?;
                            let index = raw_index.parse::<i64>().map_err(|_| {
                                FormatEditError::InvalidPath(format!(
                                    "array index is too large: {raw_index}"
                                ))
                            })?;
                            segments.push(PathSegment::Index(index));
                        }
                    }
                }
                other => {
                    return Err(FormatEditError::InvalidPath(format!(
                        "unexpected character {other:?}"
                    )));
                }
            }
        }

        Ok(Self {
            original: input.to_string(),
            segments,
        })
    }
}

fn expect_char(chars: &[char], cursor: &mut usize, expected: char) -> Result<()> {
    match chars.get(*cursor) {
        Some(actual) if *actual == expected => {
            *cursor += 1;
            Ok(())
        }
        Some(actual) => Err(FormatEditError::InvalidPath(format!(
            "expected {expected:?}, found {actual:?}"
        ))),
        None => Err(FormatEditError::InvalidPath(format!(
            "expected {expected:?}, found end of input"
        ))),
    }
}

fn parse_quoted_key(chars: &[char], cursor: &mut usize, quote: char) -> Result<String> {
    let mut output = String::new();
    while let Some(current) = chars.get(*cursor) {
        *cursor += 1;
        match current {
            current if *current == quote => return Ok(output),
            '\\' => {
                let Some(escaped) = chars.get(*cursor) else {
                    return Err(FormatEditError::InvalidPath(
                        "escape sequence is incomplete".to_string(),
                    ));
                };
                *cursor += 1;
                match escaped {
                    '\\' => output.push('\\'),
                    '\'' => output.push('\''),
                    '"' => output.push('"'),
                    'n' => output.push('\n'),
                    'r' => output.push('\r'),
                    't' => output.push('\t'),
                    other => output.push(*other),
                }
            }
            other => output.push(*other),
        }
    }

    Err(FormatEditError::InvalidPath(
        "quoted key is not closed".to_string(),
    ))
}

fn is_dot_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|char| char.is_ascii_alphanumeric() || char == '_' || char == '-')
}

fn quote_key(key: &str) -> String {
    let escaped = key.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::{JsonPath, PathSegment};

    #[test]
    fn parses_dot_bracket_and_negative_indexes() {
        let path = "$.server.hosts[0]['display-name'][-1]"
            .parse::<JsonPath>()
            .unwrap();
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Key("server".to_string()),
                PathSegment::Key("hosts".to_string()),
                PathSegment::Index(0),
                PathSegment::Key("display-name".to_string()),
                PathSegment::Index(-1),
            ]
        );
    }

    #[test]
    fn parses_wildcards() {
        let path = "$.items[*].name".parse::<JsonPath>().unwrap();
        assert!(!path.is_deterministic());
    }
}
