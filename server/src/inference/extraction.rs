use crate::db::documents::{ExtractedEntity, ExtractionResult};
use crate::AppError;

/// Extract valid JSON from raw LLM output.
///
/// Handles markdown fences, preamble/trailing text, and truncated output.
pub fn parse_llm_json(raw: &str) -> Result<serde_json::Value, AppError> {
    let text = raw.trim();

    // 1. Strip markdown code fences
    let stripped = strip_fences(text);

    // 2. Try direct parse
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&stripped) {
        if v.is_object() {
            return Ok(v);
        }
    }

    // 3. Find outermost { ... }
    if let Some(json_str) = extract_braced(&stripped) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
            if v.is_object() {
                return Ok(v);
            }
        }
    }

    // 4. Try repairing truncated JSON
    if let Some(repaired) = repair_truncated(&stripped) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&repaired) {
            if v.is_object() {
                return Ok(v);
            }
        }
    }

    Err(AppError::Inference(format!(
        "could not extract valid JSON from LLM response: {}",
        &raw[..raw.len().min(200)]
    )))
}

fn strip_fences(text: &str) -> String {
    // Match ```json\n...\n``` or ```\n...\n```
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        // Skip optional language tag
        let content_start = after_fence
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let content = &after_fence[content_start..];

        if let Some(end) = content.find("```") {
            return content[..end].trim().to_string();
        }
        // Unclosed fence (truncated output)
        return content.trim().to_string();
    }
    text.to_string()
}

fn extract_braced(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

fn repair_truncated(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let fragment = &text[start..];

    let mut open_braces: i32 = 0;
    let mut open_brackets: i32 = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in fragment.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => open_braces += 1,
            '}' => open_braces -= 1,
            '[' => open_brackets += 1,
            ']' => open_brackets -= 1,
            _ => {}
        }
    }

    // Only repair if reasonably close
    if open_braces < 0 || open_brackets < 0 || open_braces > 5 || open_brackets > 5 {
        return None;
    }

    let mut repaired = fragment.trim_end().to_string();

    // Close open string
    if in_string {
        repaired.push('"');
    }

    // Remove trailing comma
    let trimmed = repaired.trim_end();
    if trimmed.ends_with(',') {
        repaired = trimmed[..trimmed.len() - 1].to_string();
    }

    // Close brackets then braces
    for _ in 0..open_brackets {
        repaired.push(']');
    }
    for _ in 0..open_braces {
        repaired.push('}');
    }

    Some(repaired)
}

/// Parse and validate the LLM's JSON extraction response into an ExtractionResult.
pub fn parse(raw: &serde_json::Value) -> Result<ExtractionResult, AppError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| AppError::Inference("extraction response is not a JSON object".into()))?;

    let language = obj.get("language").and_then(|v| v.as_str()).map(|s| s.to_string());

    let sender = obj.get("sender").and_then(|v| v.as_str()).map(|s| s.to_string());

    let sender_normalized = obj
        .get("sender_normalized")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| sender.clone());

    let document_date = obj
        .get("document_date")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "null")
        .map(|s| s.to_string());

    let document_type = obj
        .get("document_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase());

    let subject = obj.get("subject").and_then(|v| v.as_str()).map(|s| s.to_string());

    let extracted_text = obj
        .get("extracted_text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let amounts = obj.get("amounts").filter(|v| v.is_array()).cloned();
    let dates = obj.get("dates").filter(|v| v.is_array()).cloned();
    let reference_ids = obj.get("reference_ids").filter(|v| v.is_array()).cloned();

    // Normalize tags: lowercase, underscores
    let tags = obj.get("tags").and_then(|v| {
        if let Some(arr) = v.as_array() {
            let normalized: Vec<serde_json::Value> = arr
                .iter()
                .filter_map(|t| t.as_str())
                .map(|t| {
                    serde_json::Value::String(
                        t.to_lowercase()
                            .replace(' ', "_")
                            .replace('-', "_"),
                    )
                })
                .collect();
            Some(serde_json::Value::Array(normalized))
        } else {
            None
        }
    });

    let confidence = obj
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|c| c.clamp(0.0, 1.0));

    let entities = obj
        .get("entities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let entity_type = e.get("type")?.as_str()?.to_string();
                    let value = e.get("value")?.as_str()?.to_string();
                    let role = e
                        .get("role")
                        .and_then(|r| r.as_str())
                        .unwrap_or("referenced")
                        .to_string();
                    Some(ExtractedEntity {
                        entity_type,
                        value,
                        role,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ExtractionResult {
        language,
        sender,
        sender_normalized,
        document_date,
        document_type,
        subject,
        extracted_text,
        amounts,
        dates,
        reference_ids,
        tags,
        confidence,
        raw_response: serde_json::to_string(raw).unwrap_or_default(),
        entities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json() {
        let raw = r#"{"language": "de", "sender": "AOK"}"#;
        let v = parse_llm_json(raw).unwrap();
        assert_eq!(v["language"], "de");
        assert_eq!(v["sender"], "AOK");
    }

    #[test]
    fn test_markdown_fences() {
        let raw = "```json\n{\"language\": \"en\", \"confidence\": 0.9}\n```";
        let v = parse_llm_json(raw).unwrap();
        assert_eq!(v["language"], "en");
    }

    #[test]
    fn test_preamble_and_trailing() {
        let raw = "Here is the result:\n{\"sender\": \"DKB\"}\nHope this helps!";
        let v = parse_llm_json(raw).unwrap();
        assert_eq!(v["sender"], "DKB");
    }

    #[test]
    fn test_truncated_json() {
        let raw = r#"{"language": "de", "tags": ["tax", "invoice"]"#;
        let v = parse_llm_json(raw).unwrap();
        assert_eq!(v["language"], "de");
    }

    #[test]
    fn test_invalid_input() {
        let result = parse_llm_json("no json here");
        assert!(result.is_err());
    }
}
