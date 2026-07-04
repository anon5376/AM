use crate::beval::corpus::{Matcher, Task};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScoreOutcome {
    pub matched: bool,
    pub answer: Option<String>,
    pub reasons: Vec<String>,
    pub stale_repeated: bool,
}

pub fn score_response(task: &Task, response: &str) -> ScoreOutcome {
    let Some(answer) = extract_answer(response) else {
        return ScoreOutcome {
            matched: false,
            answer: None,
            reasons: vec!["NoAnswerEnvelope".to_string()],
            stale_repeated: contains_any(response, &task.answer_key.stale_markers),
        };
    };

    let mut matched = true;
    let mut reasons = Vec::new();
    for matcher in &task.answer_key.matchers {
        if let Some(reason) = matcher_failure(matcher, &answer) {
            matched = false;
            reasons.push(reason);
        }
    }
    let stale_repeated = contains_any(&answer, &task.answer_key.stale_markers);
    ScoreOutcome {
        matched,
        answer: Some(answer),
        reasons,
        stale_repeated,
    }
}

pub fn extract_answer(response: &str) -> Option<String> {
    for line in response.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("ANSWER:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn matcher_failure(matcher: &Matcher, answer: &str) -> Option<String> {
    match matcher {
        Matcher::Exact { value } => (answer != value).then(|| "ExactMismatch".to_string()),
        Matcher::Regex { pattern } => {
            (!mini_regex_match(pattern, answer)).then(|| format!("RegexMismatch:{pattern}"))
        }
        Matcher::MustContainAll { values } => values
            .iter()
            .find(|value| !answer.contains(value.as_str()))
            .map(|value| format!("Missing:{value}")),
        Matcher::MustContainNone { values } => values
            .iter()
            .find(|value| answer.contains(value.as_str()))
            .map(|value| format!("Forbidden:{value}")),
    }
}

fn contains_any(text: &str, markers: &[String]) -> bool {
    markers.iter().any(|marker| text.contains(marker.as_str()))
}

fn mini_regex_match(pattern: &str, answer: &str) -> bool {
    let anchored_start = pattern.starts_with('^');
    let anchored_end = pattern.ends_with('$');
    let body = pattern
        .strip_prefix('^')
        .unwrap_or(pattern)
        .strip_suffix('$')
        .unwrap_or_else(|| pattern.strip_prefix('^').unwrap_or(pattern));
    if !body.contains(".*") {
        return mini_segment_match(body, answer, anchored_start, anchored_end);
    }

    let parts: Vec<&str> = body.split(".*").collect();
    let mut cursor = 0_usize;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        let Some(found) = find_segment(part, &answer[cursor..]) else {
            return false;
        };
        if anchored_start && idx == 0 && found.start != 0 {
            return false;
        }
        cursor += found.end;
    }
    if anchored_end {
        if let Some(last) = parts.iter().rev().find(|part| !part.is_empty()) {
            return ends_with_segment(answer, last);
        }
    }
    true
}

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

fn mini_segment_match(
    pattern: &str,
    answer: &str,
    anchored_start: bool,
    anchored_end: bool,
) -> bool {
    if anchored_start && anchored_end {
        return segment_full_match(pattern, answer);
    }
    if anchored_start {
        return prefix_segment_match(pattern, answer).is_some();
    }
    if anchored_end {
        return ends_with_segment(answer, pattern);
    }
    find_segment(pattern, answer).is_some()
}

fn find_segment(pattern: &str, answer: &str) -> Option<Span> {
    for start in answer
        .char_indices()
        .map(|(idx, _)| idx)
        .chain([answer.len()])
    {
        if let Some(end) = prefix_segment_match(pattern, &answer[start..]) {
            return Some(Span {
                start,
                end: start + end,
            });
        }
    }
    None
}

fn ends_with_segment(answer: &str, pattern: &str) -> bool {
    for start in answer
        .char_indices()
        .map(|(idx, _)| idx)
        .chain([answer.len()])
    {
        if segment_full_match(pattern, &answer[start..]) {
            return true;
        }
    }
    false
}

fn segment_full_match(pattern: &str, answer: &str) -> bool {
    prefix_segment_match(pattern, answer) == Some(answer.len())
}

fn prefix_segment_match(pattern: &str, answer: &str) -> Option<usize> {
    let mut pat = pattern;
    let mut cursor = 0_usize;
    while !pat.is_empty() {
        if let Some(rest) = pat.strip_prefix("\\d+") {
            let consumed = answer[cursor..]
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .map(char::len_utf8)
                .sum::<usize>();
            if consumed == 0 {
                return None;
            }
            cursor += consumed;
            pat = rest;
            continue;
        }
        let next_special = pat.find("\\d+").unwrap_or(pat.len());
        let literal = &pat[..next_special];
        if !answer[cursor..].starts_with(literal) {
            return None;
        }
        cursor += literal.len();
        pat = &pat[next_special..];
    }
    Some(cursor)
}

#[cfg(test)]
mod tests {
    use super::{extract_answer, mini_regex_match};

    #[test]
    fn answer_envelope_is_extracted() {
        assert_eq!(
            extract_answer("noise\nANSWER: ok\nignored"),
            Some("ok".to_string())
        );
    }

    #[test]
    fn mini_regex_supports_anchors_wildcard_and_digit_plus() {
        assert!(mini_regex_match("^run-\\d+$", "run-42"));
        assert!(mini_regex_match("^alpha.*omega$", "alpha middle omega"));
        assert!(!mini_regex_match("^run-\\d+$", "xrun-42"));
    }
}
