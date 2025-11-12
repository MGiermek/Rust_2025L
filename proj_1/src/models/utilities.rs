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