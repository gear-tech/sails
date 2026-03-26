use alloc::collections::BTreeSet;
use alloc::string::String;

/// Trait for loading IDL content from a path.
pub trait IdlLoader {
    /// Unique identifier for the IDL content (e.g. hash or canonical path).
    type Id: Ord + Clone;

    /// Load the content and its unique identifier of the IDL file at the given path.
    fn load(&self, path: &str) -> Result<(String, Self::Id), String>;

    /// Resolve a relative include path based on the current file path.
    fn resolve(&self, base_path: &str, include_path: &str) -> Result<String, String>;
}

/// Preprocesses the IDL source, starting from the given path,
/// resolving `!@include` directives recursively.
///
/// Each file is included at most once
pub fn preprocess<L: IdlLoader>(path: &str, loader: &L) -> Result<String, String> {
    let mut visited = BTreeSet::new();
    preprocess_recursive(path, loader, &mut visited)
}

fn preprocess_recursive<L: IdlLoader>(
    path: &str,
    loader: &L,
    visited: &mut BTreeSet<L::Id>,
) -> Result<String, String> {
    let (src, unique_id) = loader.load(path)?;

    if visited.contains(&unique_id) {
        // If already visited, we return empty string to prevent duplication/cycle
        return Ok(String::new());
    }
    visited.insert(unique_id);

    let mut result = String::new();
    let mut brace_level: i32 = 0;

    for line in src.lines() {
        let trimmed = line.trim();

        if brace_level == 0 && trimmed.starts_with("!@include:") {
            let next_path_raw = trimmed.strip_prefix("!@include:").unwrap().trim();
            let next_path_raw = next_path_raw.trim_matches('"').trim_matches('\'');

            // Resolve path using loader
            let next_path = loader.resolve(path, next_path_raw)?;

            let processed_content = preprocess_recursive(&next_path, loader, visited)?;

            result.push_str(&processed_content);
            if !processed_content.is_empty() && !processed_content.ends_with('\n') {
                result.push('\n');
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');

        let change = calculate_brace_change(line);
        brace_level += change;
    }

    Ok(result)
}

fn calculate_brace_change(line: &str) -> i32 {
    let mut change = 0;
    let mut in_string = false; // inside "..."
    let mut in_char = false; // inside '...'
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_string {
            if c == '"' {
                in_string = false;
            } else if c == '\\' {
                // Skip escaped char
                chars.next();
            }
        } else if in_char {
            if c == '\'' {
                in_char = false;
            } else if c == '\\' {
                chars.next();
            }
        } else {
            match c {
                '{' => change += 1,
                '}' => change -= 1,
                '"' => in_string = true,
                '\'' => in_char = true,
                '/' => {
                    if let Some('/') = chars.peek() {
                        // Found comment start '//', ignore rest of line
                        break;
                    }
                }
                _ => {} // Ignore other characters
            }
        }
    }
    change
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::format;
    use keccak_const::Keccak256;

    struct MapLoader(BTreeMap<String, String>);

    impl IdlLoader for MapLoader {
        type Id = [u8; 32];

        fn load(&self, path: &str) -> Result<(String, Self::Id), String> {
            let content = self
                .0
                .get(path)
                .cloned()
                .ok_or_else(|| format!("File not found: {path}"))?;

            let hash_raw = Keccak256::new().update(content.as_bytes()).finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_raw);

            Ok((content, hash))
        }

        fn resolve(&self, base_path: &str, include_path: &str) -> Result<String, String> {
            if let Some(pos) = base_path.rfind('/') {
                Ok(format!("{}{}", &base_path[..pos + 1], include_path))
            } else {
                Ok(String::from(include_path))
            }
        }
    }

    #[test]
    fn test_preprocess_recursive() {
        let mut files = BTreeMap::new();
        files.insert("leaf.idl".into(), "service Leaf {}".into());
        files.insert(
            "middle.idl".into(),
            "!@include: leaf.idl\nservice Middle {}".into(),
        );
        files.insert(
            "main.idl".into(),
            "!@include: middle.idl\nservice Main {}".into(),
        );

        let loader = MapLoader(files);
        let result = preprocess("main.idl", &loader).unwrap();

        assert!(result.contains("service Leaf"));
        assert!(result.contains("service Middle"));
        assert!(result.contains("service Main"));
    }

    #[test]
    fn test_preprocess_duplicate_prevented() {
        let mut files = BTreeMap::new();
        files.insert("common.idl".into(), "struct Common {}".into());
        files.insert("a.idl".into(), "!@include: common.idl\nservice A {}".into());
        files.insert("b.idl".into(), "!@include: common.idl\nservice B {}".into());
        files.insert(
            "main.idl".into(),
            "!@include: a.idl\n!@include: b.idl".into(),
        );

        let loader = MapLoader(files);
        let result = preprocess("main.idl", &loader).unwrap();

        // Count occurrences of "struct Common"
        let count = result.matches("struct Common").count();
        assert_eq!(count, 1); // Should be included only once
    }

    #[test]
    fn test_brace_counting_robustness() {
        // Case 1: Braces in comments
        // { -> +1, // starts comment, rest ignored. Total 1.
        assert_eq!(calculate_brace_change("service { // { }"), 1);

        // Case 2: Braces in strings
        // { -> +1
        // " { " -> string, braces inside ignored
        // } -> -1
        // Total 0.
        assert_eq!(calculate_brace_change(r#"service { " { " }"#), 0);

        // Case 3: Escaped quotes in strings
        // { -> +1
        // " -> start string
        // \" -> escaped quote (ignored)
        // { -> inside string (ignored)
        // " -> end string
        // } -> -1
        // Total 0.
        assert_eq!(calculate_brace_change(r#"{ " \" { " }"#), 0);
    }
}
