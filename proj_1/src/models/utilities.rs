pub fn split_preserving_quote_insides(input: &str, splitter: char) -> Vec<&str> {
    let mut result = Vec::<&str>::new();
    let mut start = 0;
    let mut in_quotes = false;

    for (i, c) in input.char_indices() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == splitter && !in_quotes {
            let output = &input[start..i].trim();
            if !output.is_empty() {
                result.push(output);
            }
            start = i + 1;
        }
    }
    
    let output = &input[start..].trim();
    if !output.is_empty() {
        result.push(output);
    }
    result
}

pub fn split_once_skipping_outside_quotes(input: &str, splitter: char) -> Option<(&str, &str)> {
    
    let Some((mut left, mut right)) = input.split_once(splitter) else {
        return None;
    };

    left = left.trim();
    right = right.trim();

    if left.starts_with('"') && left.ends_with('"') {
        left = &left[1..left.len()-1];
    }
    if right.starts_with('"') && right.ends_with('"') {
        right = &right[1..right.len()-1];
    }

    Some((left, right))
}

/// Operators have to be sorted by length descending!!!
pub fn split_by_operators_preserving_quotes<'a>(input: &'a str, operators: &[&str]) -> Vec<&'a str> {
    let mut result = Vec::new();
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut idx = 0usize; // current index in chars
    let mut start_byte = 0usize; // start byte index of the current pending token
    let mut in_quotes = false;

    while idx < chars.len() {
        let (byte_pos, ch) = chars[idx];
        if ch == '"' {
            in_quotes = !in_quotes;
            idx += 1;
            continue;
        }

        if !in_quotes {
            if let Some(op) = {
                let mut best: Option<&str> = None;
                for &o in operators {
                    if input[byte_pos..].starts_with(o) {
                        best = Some(o);
                        break;
                    }
                }
                best
            } {
                if start_byte < byte_pos {
                    let before = input[start_byte..byte_pos].trim();
                    if !before.is_empty() {
                        result.push(before);
                    }
                }

                let next_byte = byte_pos + op.len();
                result.push(&input[byte_pos..next_byte]);
                
                while idx < chars.len() && chars[idx].0 < next_byte {
                    idx += 1;
                }
                start_byte = next_byte;
                continue;
            }
        }

        idx += 1;
    }

    if start_byte < input.len() {
        let trailing = input[start_byte..].trim();
        if !trailing.is_empty() {
            result.push(trailing);
        }
    }
    result
}