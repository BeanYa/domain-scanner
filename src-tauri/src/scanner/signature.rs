use crate::models::task::ScanMode;
use sha2::{Digest, Sha256};

/// Generate a unique signature for a scan task based on mode + TLDs
pub fn generate_signature(scan_mode: &ScanMode, tlds: &[String]) -> String {
    // Sort TLDs to ensure order-independent signatures
    let mut sorted_tlds: Vec<&str> = tlds.iter().map(|t| t.as_str()).collect();
    sorted_tlds.sort_unstable();
    let tlds_key = sorted_tlds.join(",");

    let raw = match scan_mode {
        ScanMode::Regex { pattern } => format!("regex:{pattern}:tlds:[{tlds_key}]"),
        ScanMode::Wildcard { pattern } => format!("wildcard:{pattern}:tlds:[{tlds_key}]"),
        ScanMode::Llm { config_id, prompt } => {
            let prompt_hash = sha256_hash(prompt.as_bytes());
            format!("llm:{config_id}:{prompt_hash}:tlds:[{tlds_key}]")
        }
        ScanMode::Manual { domains } => {
            let domains_hash = sha256_hash(domains.join(",").as_bytes());
            format!("manual:{domains_hash}:tlds:[{tlds_key}]")
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
    fn test_signature_deterministic_single_tld() {
        let mode = ScanMode::Regex {
            pattern: "^[a-z]{4}$".to_string(),
        };
        let sig1 = generate_signature(&mode, &[".com".to_string()]);
        let sig2 = generate_signature(&mode, &[".com".to_string()]);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_deterministic_multi_tld() {
        let mode = ScanMode::Regex {
            pattern: "^[a-z]{4}$".to_string(),
        };
        let tlds = vec![".com".to_string(), ".net".to_string(), ".org".to_string()];
        let sig1 = generate_signature(&mode, &tlds);
        let sig2 = generate_signature(&mode, &tlds);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_tld_order_independent() {
        let mode = ScanMode::Regex {
            pattern: "^[a-z]{3}$".to_string(),
        };
        let sig_a = generate_signature(&mode, &vec![".com".to_string(), ".net".to_string()]);
        let sig_b = generate_signature(&mode, &vec![".net".to_string(), ".com".to_string()]);
        assert_eq!(sig_a, sig_b, "TLD order should not affect signature");
    }

    #[test]
    fn test_different_tlds_different_signatures() {
        let mode = ScanMode::Regex {
            pattern: "^[a-z]{4}$".to_string(),
        };
        let sig_com = generate_signature(&mode, &[".com".to_string()]);
        let sig_net = generate_signature(&mode, &[".net".to_string()]);
        assert_ne!(sig_com, sig_net);

        let sig_multi = generate_signature(&mode, &vec![".com".to_string(), ".net".to_string()]);
        assert_ne!(sig_com, sig_multi);
    }

    #[test]
    fn test_signature_single_tld_compat_with_old_format() {
        // Single TLD should still produce a unique signature
        let mode = ScanMode::Regex {
            pattern: "^[a-z]{3}$".to_string(),
        };
        let sig = generate_signature(&mode, &[".com".to_string()]);
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64); // SHA-256 hex string
    }

    #[test]
    fn test_different_modes_different_signatures() {
        let regex_mode = ScanMode::Regex {
            pattern: "^[a-z]{4}$".to_string(),
        };
        let wildcard_mode = ScanMode::Wildcard {
            pattern: "????".to_string(),
        };
        let tlds = vec![".com".to_string()];
        let sig1 = generate_signature(&regex_mode, &tlds);
        let sig2 = generate_signature(&wildcard_mode, &tlds);
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_llm_signature_uses_prompt_hash() {
        let mode1 = ScanMode::Llm {
            config_id: "c1".to_string(),
            prompt: "AI domains".to_string(),
        };
        let mode2 = ScanMode::Llm {
            config_id: "c1".to_string(),
            prompt: "AI domains".to_string(),
        };
        let tlds = vec![".com".to_string()];
        assert_eq!(
            generate_signature(&mode1, &tlds),
            generate_signature(&mode2, &tlds)
        );

        let mode3 = ScanMode::Llm {
            config_id: "c1".to_string(),
            prompt: "Different prompt".to_string(),
        };
        assert_ne!(
            generate_signature(&mode1, &tlds),
            generate_signature(&mode3, &tlds)
        );
    }

    #[test]
    fn test_manual_signature_uses_domains_hash() {
        let mode1 = ScanMode::Manual {
            domains: vec!["test".to_string(), "demo".to_string()],
        };
        let mode2 = ScanMode::Manual {
            domains: vec!["test".to_string(), "demo".to_string()],
        };
        let tlds = vec![".com".to_string(), ".io".to_string()];
        assert_eq!(
            generate_signature(&mode1, &tlds),
            generate_signature(&mode2, &tlds)
        );

        let mode3 = ScanMode::Manual {
            domains: vec!["other".to_string()],
        };
        assert_ne!(
            generate_signature(&mode1, &tlds),
            generate_signature(&mode3, &tlds)
        );
    }
}
