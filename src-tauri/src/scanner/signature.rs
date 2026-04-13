use sha2::{Sha256, Digest};
use crate::models::task::ScanMode;

/// Generate a unique signature for a scan task based on mode + TLD
pub fn generate_signature(scan_mode: &ScanMode, tld: &str) -> String {
    let raw = match scan_mode {
        ScanMode::Regex { pattern } => format!("regex:{pattern}:tld:{tld}"),
        ScanMode::Wildcard { pattern } => format!("wildcard:{pattern}:tld:{tld}"),
        ScanMode::Llm { config_id, prompt } => {
            let prompt_hash = sha256_hash(prompt.as_bytes());
            format!("llm:{config_id}:{prompt_hash}:tld:{tld}")
        }
        ScanMode::Manual { domains } => {
            let domains_hash = sha256_hash(domains.join(",").as_bytes());
            format!("manual:{domains_hash}:tld:{tld}")
        }
    };
    sha256_hash(raw.as_bytes())
}

fn sha256_hash(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_deterministic() {
        let mode = ScanMode::Regex { pattern: "^[a-z]{4}$".to_string() };
        let sig1 = generate_signature(&mode, ".com");
        let sig2 = generate_signature(&mode, ".com");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_different_tlds_different_signatures() {
        let mode = ScanMode::Regex { pattern: "^[a-z]{4}$".to_string() };
        let sig_com = generate_signature(&mode, ".com");
        let sig_net = generate_signature(&mode, ".net");
        assert_ne!(sig_com, sig_net);
    }

    #[test]
    fn test_different_modes_different_signatures() {
        let regex_mode = ScanMode::Regex { pattern: "^[a-z]{4}$".to_string() };
        let wildcard_mode = ScanMode::Wildcard { pattern: "????".to_string() };
        let sig1 = generate_signature(&regex_mode, ".com");
        let sig2 = generate_signature(&wildcard_mode, ".com");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_llm_signature_uses_prompt_hash() {
        let mode1 = ScanMode::Llm { config_id: "c1".to_string(), prompt: "AI domains".to_string() };
        let mode2 = ScanMode::Llm { config_id: "c1".to_string(), prompt: "AI domains".to_string() };
        let sig1 = generate_signature(&mode1, ".com");
        let sig2 = generate_signature(&mode2, ".com");
        assert_eq!(sig1, sig2);

        let mode3 = ScanMode::Llm { config_id: "c1".to_string(), prompt: "Different prompt".to_string() };
        let sig3 = generate_signature(&mode3, ".com");
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_manual_signature_uses_domains_hash() {
        let mode1 = ScanMode::Manual { domains: vec!["test".to_string(), "demo".to_string()] };
        let mode2 = ScanMode::Manual { domains: vec!["test".to_string(), "demo".to_string()] };
        assert_eq!(
            generate_signature(&mode1, ".com"),
            generate_signature(&mode2, ".com")
        );

        let mode3 = ScanMode::Manual { domains: vec!["other".to_string()] };
        assert_ne!(
            generate_signature(&mode1, ".com"),
            generate_signature(&mode3, ".com")
        );
    }
}
