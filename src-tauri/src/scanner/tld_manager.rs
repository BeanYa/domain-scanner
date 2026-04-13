/// TLD Manager - manages the list of top-level domains
/// Built-in list of common TLDs, with ability to load from ICANN source

/// Built-in TLD list (most common)
pub const BUILTIN_TLDS: &[&str] = &[
    ".com", ".net", ".org", ".io", ".co", ".dev", ".app", ".ai",
    ".xyz", ".me", ".info", ".biz", ".us", ".uk", ".de", ".fr",
    ".jp", ".cn", ".ru", ".ca", ".au", ".in", ".br", ".it",
    ".es", ".nl", ".se", ".no", ".ch", ".at", ".be", ".dk",
    ".pl", ".cz", ".eu", ".tv", ".cc", ".ws", ".mobi", ".name",
    ".pro", ".tel", ".travel", ".jobs", ".cat", ".asia", ".post",
    ".club", ".online", ".site", ".tech", ".store", ".fun",
    ".space", ".top", ".blog", ".shop", ".cloud", ".digital",
    ".guru", ".ninja", ".rocks", ".world", ".today", ".solutions",
    ".agency", ".company", ".ventures", ".technology", ".directory",
    ".academy", ".management", ".builders", ".systems", ".institute",
    ".catering", ".properties", ".international", ".photography",
    ".gallery", ".graphics", ".lighting", ".photography",
];

pub struct TldManager;

impl TldManager {
    pub fn new() -> Self {
        Self
    }

    /// Get all built-in TLDs
    pub fn get_all_tlds(&self) -> Vec<String> {
        BUILTIN_TLDS.iter().map(|s| s.to_string()).collect()
    }

    /// Check if a TLD is in the built-in list
    pub fn is_valid_tld(&self, tld: &str) -> bool {
        let normalized = Self::normalize_tld(tld);
        BUILTIN_TLDS.contains(&normalized.as_str())
    }

    /// Normalize a TLD string (ensure it starts with a dot)
    pub fn normalize_tld(tld: &str) -> String {
        let trimmed = tld.trim().to_lowercase();
        if trimmed.starts_with('.') {
            trimmed
        } else {
            format!(".{}", trimmed)
        }
    }

    /// Get common TLDs (top 20 by popularity)
    pub fn get_common_tlds(&self) -> Vec<String> {
        BUILTIN_TLDS[..20].iter().map(|s| s.to_string()).collect()
    }

    /// Estimate the number of domain candidates for a given pattern and TLD list
    pub fn estimate_scan_count(&self, prefix_count: u64, tld_count: usize) -> u64 {
        prefix_count * tld_count as u64
    }
}

impl Default for TldManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_tlds_not_empty() {
        let mgr = TldManager::new();
        let tlds = mgr.get_all_tlds();
        assert!(!tlds.is_empty());
        assert!(tlds.len() > 50);
    }

    #[test]
    fn test_is_valid_tld() {
        let mgr = TldManager::new();
        assert!(mgr.is_valid_tld(".com"));
        assert!(mgr.is_valid_tld("com")); // without dot should also work
        assert!(mgr.is_valid_tld(".IO")); // case insensitive
        assert!(!mgr.is_valid_tld(".nonexistent"));
    }

    #[test]
    fn test_normalize_tld() {
        assert_eq!(TldManager::normalize_tld("com"), ".com");
        assert_eq!(TldManager::normalize_tld(".com"), ".com");
        assert_eq!(TldManager::normalize_tld("  COM  "), ".com");
        assert_eq!(TldManager::normalize_tld(".IO"), ".io");
    }

    #[test]
    fn test_get_common_tlds() {
        let mgr = TldManager::new();
        let common = mgr.get_common_tlds();
        assert_eq!(common.len(), 20);
        assert!(common.contains(&".com".to_string()));
        assert!(common.contains(&".net".to_string()));
    }

    #[test]
    fn test_estimate_scan_count() {
        let mgr = TldManager::new();
        // 3-letter domains: 26^3 = 17576
        let count = mgr.estimate_scan_count(17576, 3);
        assert_eq!(count, 52728);
    }
}
