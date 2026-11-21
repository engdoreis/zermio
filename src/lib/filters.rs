// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

// This filter does not have extra arguments
pub fn pascal_case<T: std::fmt::Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
    let s = s.to_string();
    let result = s
        .split(['_', ' '])
        .filter(|part| !part.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string()
                        + chars.as_str().to_string().to_ascii_lowercase().as_str()
                }
                None => String::new(),
            }
        })
        .collect();
    Ok(result)
}

pub fn hex<T>(num: T, _: &dyn askama::Values) -> askama::Result<String>
where
    T: std::fmt::Display + std::fmt::LowerHex,
{
    Ok(format!("{:#x}", num))
}

// Rust's strict + reserved keywords that can be escaped as raw identifiers
// (`r#...`). `self`, `Self`, `super` and `crate` are deliberately excluded:
// the language doesn't allow those four as raw identifiers at all, so a
// hardware register/field named e.g. "self" would need a real rename rather
// than this filter -- not expected to come up in practice.
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static",
    "struct", "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
];

/// Escapes an identifier that happens to collide with a Rust keyword (e.g. a
/// register named `MOD` or a field named `TYPE`) into a raw identifier, so
/// it can still be used as a module/field/function name in generated code.
pub fn kw<T: std::fmt::Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
    let s = s.to_string();
    if RUST_KEYWORDS.contains(&s.as_str()) {
        Ok(format!("r#{s}"))
    } else {
        Ok(s)
    }
}
