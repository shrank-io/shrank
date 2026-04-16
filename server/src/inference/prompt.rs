// ---------------------------------------------------------------------------
// Pass 1: OCR — image to markdown
// ---------------------------------------------------------------------------

pub const OCR_SYSTEM_PROMPT: &str = r#"You are a precise document OCR engine. You receive a photograph of a physical document (letter, invoice, statement, contract, notice).

Your ONLY job is to transcribe ALL visible text into clean Markdown.

Rules:
1. Transcribe EVERY piece of text visible in the image — headers, body, footers, fine print, stamps, handwriting.
2. Preserve the document's visual structure using Markdown: headings (#), lists (-), tables (|), bold (**), line breaks.
3. For tables, use Markdown table syntax. If alignment is unclear, do your best.
4. Preserve the original language exactly. Do NOT translate anything.
5. If text is partially illegible, transcribe what you can and mark unclear parts with [illegible] or [unclear: best guess].
6. Include amounts, dates, reference numbers, and IBANs exactly as printed — do not reformat them.
7. Output ONLY the Markdown transcription. No commentary, no metadata, no JSON."#;

pub const OCR_USER_PROMPT: &str = "Transcribe all text from this document image into Markdown.";

// ---------------------------------------------------------------------------
// Pass 2: Extraction — markdown text to structured JSON
// ---------------------------------------------------------------------------

pub const EXTRACTION_SYSTEM_PROMPT: &str = r#"You are a document analysis engine for a personal paper mail archive.
You receive the OCR text of a physical mail document (letter, invoice, statement, contract, notice).
Your job is to extract all structured information from this text.

CRITICAL RULES:
1. Detect the document language from the text content. Do NOT assume any language.
2. Identify the sender/organization, dates, amounts, reference numbers.
3. Suggest descriptive tags for categorization. Tags should be in English for consistency,
   but include important domain-specific terms in the original language as additional tags.
4. Generate a one-sentence English summary regardless of document language.
5. Report your confidence (0.0-1.0) in the extraction quality.
6. Output ONLY valid JSON. No markdown, no explanation, no preamble."#;

pub fn build_extraction_prompt(
    ocr_text: &str,
    existing_tags: &[String],
    existing_senders: &[String],
) -> String {
    let mut prompt = String::from(
        "Extract structured information from the following document text.\n\n",
    );

    prompt.push_str("--- DOCUMENT TEXT ---\n");
    // Limit to ~12k chars to leave room for the schema and response
    let limit = ocr_text.len().min(12_000);
    prompt.push_str(&ocr_text[..limit]);
    if ocr_text.len() > limit {
        prompt.push_str("\n[... truncated ...]\n");
    }
    prompt.push_str("\n--- END DOCUMENT TEXT ---\n");

    if !existing_tags.is_empty() {
        prompt.push_str(
            "\nEXISTING TAGS IN ARCHIVE (reuse these when applicable, add new ones if needed):\n",
        );
        let tag_limit = existing_tags.len().min(100);
        prompt.push_str(&existing_tags[..tag_limit].join(", "));
        prompt.push('\n');
    }

    if !existing_senders.is_empty() {
        prompt.push_str(
            "\nKNOWN SENDERS (use exact spelling if this document is from one of these):\n",
        );
        let sender_limit = existing_senders.len().min(50);
        prompt.push_str(&existing_senders[..sender_limit].join(", "));
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
  "extracted_text": "<the full document text from above, cleaned up>",
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
  "extraction_notes": "<any issues: unclear text, missing sections>"
}

IMPORTANT: Output ONLY the JSON object. No other text."#,
    );

    prompt
}
