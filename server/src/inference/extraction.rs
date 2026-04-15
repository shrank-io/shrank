use crate::db::documents::{ExtractedEntity, ExtractionResult};
use crate::AppError;

/// Parse and validate the LLM's JSON extraction response.
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
        .or_else(|| sender.clone()); // fallback to sender if normalized not provided

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

    // Parse entities
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
