pub const EXTRACTION_SYSTEM_PROMPT: &str = r#"You are a document analysis engine for a personal paper mail archive.
You receive photographs of physical mail documents (letters, invoices, statements, contracts, notices).
Your job is to extract all structured information from the document image.

CRITICAL RULES:
1. Detect the document language automatically. Do NOT assume any language.
2. Extract ALL visible text from the document (OCR).
3. Identify the sender/organization, dates, amounts, reference numbers.
4. Suggest descriptive tags for categorization. Tags should be in English for consistency,
   but include important domain-specific terms in the original language as additional tags.
5. Generate a one-sentence English summary regardless of document language.
6. Report your confidence (0.0-1.0) in the extraction quality.
7. If the image is blurry, partially cut off, or illegible, still extract what you can
   and set confidence accordingly.
8. Output ONLY valid JSON. No markdown, no explanation, no preamble."#;

pub fn build_extraction_prompt(existing_tags: &[String], existing_senders: &[String]) -> String {
    let mut prompt = String::from("Analyze this document image and extract all information.\n");

    if !existing_tags.is_empty() {
        prompt.push_str(
            "\nEXISTING TAGS IN ARCHIVE (reuse these when applicable, add new ones if needed):\n",
        );
        let limit = existing_tags.len().min(100);
        prompt.push_str(&existing_tags[..limit].join(", "));
        prompt.push('\n');
    }

    if !existing_senders.is_empty() {
        prompt.push_str(
            "\nKNOWN SENDERS (use exact spelling if this document is from one of these):\n",
        );
        let limit = existing_senders.len().min(50);
        prompt.push_str(&existing_senders[..limit].join(", "));
        prompt.push('\n');
    }

    prompt.push_str(
        r#"
Respond with a single JSON object in this exact schema:

{
  "language": "<ISO 639-1 code>",
  "sender": "<full sender name as printed>",
  "sender_normalized": "<cleaned/canonical sender name>",
  "document_date": "<YYYY-MM-DD or null if not visible>",
  "document_type": "<one of: invoice, letter, policy, statement, contract, receipt, notification, certificate, form, reminder, other>",
  "subject": "<one-line summary in the document's language>",
  "summary": "<one-sentence English summary of the document's purpose and key information>",
  "extracted_text": "<full OCR text, preserving line breaks>",
  "amounts": [
    {"value": <number>, "currency": "<ISO 4217>", "label": "<what this amount represents>"}
  ],
  "dates": [
    {"date": "<YYYY-MM-DD>", "label": "<what this date represents>"}
  ],
  "reference_ids": [
    {"type": "<policy|invoice|account|iban|tax_id|reference|customer_id|contract>", "value": "<the ID>"}
  ],
  "tags": ["<tag1>", "<tag2>", "..."],
  "entities": [
    {"type": "<organization|person|policy|vehicle|property|account>", "value": "<entity value>", "role": "<sender|recipient|referenced|beneficiary>"}
  ],
  "related_references": ["<any explicit references to other documents, e.g. 'Bezug auf Ihr Schreiben vom 15.03.2026'>"],
  "confidence": <0.0 to 1.0>,
  "extraction_notes": "<any issues: blurry text, cut-off sections, handwritten parts>"
}

IMPORTANT: Output ONLY the JSON object. No other text."#,
    );

    prompt
}
