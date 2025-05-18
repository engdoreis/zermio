// Copyright (c) 2025 Douglas Reis.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

// This filter does not have extra arguments
pub fn pascal_case<T: std::fmt::Display>(s: T, _: &dyn askama::Values) -> askama::Result<String> {
    let s = s.to_string();
    let result = s
        .split('_')
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
