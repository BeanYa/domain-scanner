/// Prompt templates for LLM-based domain name generation

/// Build a prompt for generating domain name candidates
pub fn build_domain_generation_prompt(
    description: &str,
    tld: &str,
    count: usize,
    language: &str,
) -> String {
    format!(
        r#"You are a domain name expert. Generate {count} creative and brandable domain name candidates.

Requirements:
- Each domain name should be a single word or short phrase (2-12 characters)
- Target TLD: {tld}
- Language: {language}
- Focus on: {description}
- Output format: one domain name per line, no numbering, no explanations
- Only output the domain prefix (without the TLD)
- Avoid trademarked names
- Prefer memorable, pronounceable names

Generate {count} domain name prefixes:"#,
        count = count,
        tld = tld,
        language = language,
        description = description
    )
}

/// Build a prompt for semantic filtering of domain names
pub fn build_semantic_filter_prompt(domains: &[String], criteria: &str, tld: &str) -> String {
    let domain_list = domains.join("\n");
    format!(
        r#"You are evaluating domain names for a specific purpose.

Domain names (prefix only, TLD: {tld}):
{domain_list}

Filtering criteria: {criteria}

For each domain, determine if it matches the criteria.
Output format: one domain per line, only include domains that MATCH the criteria.
Do not include any explanations or numbering.
Only output the domain prefixes that match."#,
        tld = tld,
        domain_list = domain_list,
        criteria = criteria
    )
}

/// Build a prompt for domain name scoring
pub fn build_domain_scoring_prompt(domain: &str, criteria: &str) -> String {
    format!(
        r#"Rate this domain name on a scale of 1-10 for the given criteria.

Domain: {domain}
Criteria: {criteria}

Consider:
- Memorability (easy to remember)
- Pronounceability (easy to say)
- Brand potential (commercial value)
- Length (shorter is generally better)
- Relevance to criteria

Output ONLY a single number from 1-10."#,
        domain = domain,
        criteria = criteria
    )
}

/// Parse domain generation response into a list of domain prefixes
pub fn parse_domain_list(response: &str) -> Vec<String> {
    response
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .map(|line| {
            // Remove numbering like "1. " or "1) " or "- "
            let trimmed = line.trim_start();
            let cleaned = if trimmed.starts_with('-') || trimmed.starts_with('*') {
                trimmed[1..].trim_start().to_string()
            } else if let Some(rest) = strip_numbering(trimmed) {
                rest
            } else {
                trimmed.to_string()
            };
            cleaned
        })
        .filter(|line| !line.is_empty())
        .collect()
}

/// Strip leading numbering like "1. ", "1) ", "1 "
fn strip_numbering(s: &str) -> Option<String> {
    let digits_end = s
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, c)| i + c.len_utf8())?;

    if digits_end == 0 {
        return None;
    }

    let rest = &s[digits_end..];
    if rest.starts_with(". ") {
        Some(rest[2..].to_string())
    } else if rest.starts_with(") ") {
        Some(rest[2..].to_string())
    } else if rest.starts_with(".") || rest.starts_with(")") {
        Some(rest[1..].trim_start().to_string())
    } else if rest.starts_with(' ') {
        Some(rest.trim_start().to_string())
    } else {
        None
    }
}

/// Parse semantic filter response to get matched domains
pub fn parse_filter_response(response: &str) -> Vec<String> {
    parse_domain_list(response)
}

/// Parse scoring response to get a numeric score
pub fn parse_score(response: &str) -> Option<f32> {
    let trimmed = response.trim();
    // Try to extract a number from the response
    for part in trimmed.split_whitespace() {
        if let Ok(score) = part.parse::<f32>() {
            if (1.0..=10.0).contains(&score) {
                return Some(score);
            }
        }
    }
    // Try parsing the whole string
    trimmed
        .parse::<f32>()
        .ok()
        .filter(|s| (1.0..=10.0).contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_domain_generation_prompt() {
        let prompt = build_domain_generation_prompt("tech startup", ".com", 50, "English");
        assert!(prompt.contains("50"));
        assert!(prompt.contains(".com"));
        assert!(prompt.contains("tech startup"));
        assert!(prompt.contains("English"));
    }

    #[test]
    fn test_build_semantic_filter_prompt() {
        let domains = vec!["techworld".to_string(), "codehub".to_string()];
        let prompt = build_semantic_filter_prompt(&domains, "technology related", ".com");
        assert!(prompt.contains("techworld"));
        assert!(prompt.contains("codehub"));
        assert!(prompt.contains("technology related"));
    }

    #[test]
    fn test_build_domain_scoring_prompt() {
        let prompt = build_domain_scoring_prompt("techworld.com", "technology brand");
        assert!(prompt.contains("techworld.com"));
        assert!(prompt.contains("technology brand"));
    }

    #[test]
    fn test_parse_domain_list_basic() {
        let response = "techworld\ncodehub\ndevstream";
        let domains = parse_domain_list(response);
        assert_eq!(domains, vec!["techworld", "codehub", "devstream"]);
    }

    #[test]
    fn test_parse_domain_list_with_numbering() {
        let response = "1. techworld\n2. codehub\n3. devstream";
        let domains = parse_domain_list(response);
        assert_eq!(domains, vec!["techworld", "codehub", "devstream"]);
    }

    #[test]
    fn test_parse_domain_list_with_parentheses() {
        let response = "1) techworld\n2) codehub";
        let domains = parse_domain_list(response);
        assert_eq!(domains, vec!["techworld", "codehub"]);
    }

    #[test]
    fn test_parse_domain_list_skips_empty() {
        let response = "techworld\n\n\ncodehub\n";
        let domains = parse_domain_list(response);
        assert_eq!(domains, vec!["techworld", "codehub"]);
    }

    #[test]
    fn test_parse_filter_response() {
        let response = "techworld\ncodehub";
        let domains = parse_filter_response(response);
        assert_eq!(domains, vec!["techworld", "codehub"]);
    }

    #[test]
    fn test_parse_score_valid() {
        assert_eq!(parse_score("8"), Some(8.0));
        assert_eq!(parse_score("8.5"), Some(8.5));
        assert_eq!(parse_score(" 7 "), Some(7.0));
    }

    #[test]
    fn test_parse_score_invalid() {
        assert_eq!(parse_score("great"), None);
        assert_eq!(parse_score("0"), None);
        assert_eq!(parse_score("11"), None);
    }

    #[test]
    fn test_parse_score_extracts_from_text() {
        assert_eq!(parse_score("I'd give it 8 out of 10"), Some(8.0));
    }
}
