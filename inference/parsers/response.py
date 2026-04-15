import json
import re


# All fields the extraction response must contain, with default values for
# missing ones so the backend always gets a complete object.
_DEFAULTS = {
    "language": None,
    "sender": None,
    "sender_normalized": None,
    "document_date": None,
    "document_type": None,
    "subject": None,
    "summary": None,
    "extracted_text": None,
    "amounts": [],
    "dates": [],
    "reference_ids": [],
    "tags": [],
    "entities": [],
    "related_references": [],
    "confidence": 0.0,
    "extraction_notes": None,
}


def parse_llm_json(raw_text: str) -> dict:
    """Extract a JSON object from raw LLM output.

    Handles common quirks:
    - Markdown code fences (```json ... ```)
    - Preamble text before the JSON
    - Trailing text after the closing brace
    - Partial/truncated JSON (best-effort brace repair)
    """
    text = raw_text.strip()

    # Strip markdown code fences
    text = _strip_fences(text)

    # Try direct parse first
    result = _try_parse(text)
    if result is not None:
        return _ensure_schema(result)

    # Find the outermost JSON object by locating first { and last }
    result = _extract_braced(text)
    if result is not None:
        return _ensure_schema(result)

    # Last resort: try to repair truncated JSON (missing closing braces/brackets)
    result = _repair_truncated(text)
    if result is not None:
        return _ensure_schema(result)

    raise ValueError(f"Could not extract valid JSON from LLM response: {raw_text[:200]}")


def _strip_fences(text: str) -> str:
    """Remove markdown code fences."""
    # Match ```json ... ``` or ``` ... ```
    m = re.search(r"```(?:json)?\s*\n?(.*?)```", text, re.DOTALL)
    if m:
        return m.group(1).strip()
    # Single opening fence without closing (truncated)
    m = re.match(r"```(?:json)?\s*\n?(.*)", text, re.DOTALL)
    if m:
        return m.group(1).strip()
    return text


def _try_parse(text: str) -> dict | None:
    try:
        obj = json.loads(text)
        if isinstance(obj, dict):
            return obj
    except (json.JSONDecodeError, ValueError):
        pass
    return None


def _extract_braced(text: str) -> dict | None:
    """Find the first top-level { ... } span and parse it."""
    start = text.find("{")
    if start == -1:
        return None
    end = text.rfind("}")
    if end == -1 or end <= start:
        return None
    return _try_parse(text[start : end + 1])


def _repair_truncated(text: str) -> dict | None:
    """Try to close unclosed braces/brackets for truncated output."""
    start = text.find("{")
    if start == -1:
        return None
    fragment = text[start:]

    # Count unclosed braces and brackets
    open_braces = 0
    open_brackets = 0
    in_string = False
    escape = False

    for ch in fragment:
        if escape:
            escape = False
            continue
        if ch == "\\":
            escape = True
            continue
        if ch == '"':
            in_string = not in_string
            continue
        if in_string:
            continue
        if ch == "{":
            open_braces += 1
        elif ch == "}":
            open_braces -= 1
        elif ch == "[":
            open_brackets += 1
        elif ch == "]":
            open_brackets -= 1

    # Only attempt repair if we're not too far off
    if open_braces < 0 or open_brackets < 0 or open_braces > 5 or open_brackets > 5:
        return None

    # Truncate any trailing partial string value
    # (e.g. the LLM was mid-sentence when it hit the token limit)
    repaired = fragment.rstrip()
    if in_string:
        # Close the open string
        repaired += '"'

    # Remove any trailing comma before we close containers
    repaired = re.sub(r",\s*$", "", repaired)

    repaired += "]" * open_brackets + "}" * open_braces
    return _try_parse(repaired)


def _ensure_schema(obj: dict) -> dict:
    """Fill in missing fields with defaults so the response always matches the contract."""
    for key, default in _DEFAULTS.items():
        if key not in obj:
            obj[key] = default
    return obj
