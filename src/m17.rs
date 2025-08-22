// Simple M17 parser - extracts only the essential information

/// Parses an M17 protocol line and extracts destination, source, type, and metadata
pub fn parse_m17_line(line: &str) -> Option<String> {
    if !line.contains("LSF_CRC_OK") {
        return None;
    }
    
    let mut result = String::new();
    
    if let Some(dst) = extract_field(line, "DST:") {
        result.push_str(&format!("To: {}\n", dst));
    }
    
    if let Some(src) = extract_field(line, "SRC:") {
        result.push_str(&format!("From: {}\n", src));
    }
    
    if let Some(typ) = extract_field(line, "TYPE:") {
        result.push_str(&format!("Type: {}\n", typ));
    }
    
    if let Some(meta) = extract_field(line, "META:") {
        result.push_str(&format!("Meta: {}", meta));
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn extract_field(line: &str, field: &str) -> Option<String> {
    if let Some(start) = line.find(field) {
        let value_start = start + field.len();
        let remaining = &line[value_start..];
        
        let end = remaining.find(" DST:")
            .or_else(|| remaining.find(" SRC:"))
            .or_else(|| remaining.find(" TYPE:"))
            .or_else(|| remaining.find(" META:"))
            .or_else(|| remaining.find(" NONCE:"))
            .or_else(|| remaining.find(" CRC:"))
            .or_else(|| remaining.find(" LSF_CRC"))
            .unwrap_or(remaining.len());
        
        let value = remaining[..end].trim();
        
        if !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    } else {
        None
    }
}