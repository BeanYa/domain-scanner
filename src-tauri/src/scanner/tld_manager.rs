/// TLD Manager - manages the list of top-level domains
/// Built-in list of common TLDs, with ability to load from ICANN source

/// Built-in TLD list — comprehensive 230+ entries organized by category.
///
/// Categories: gtld, new_gtld, cctld, other
/// Popular entries (most commonly scanned) are listed first within each section.
///
/// ── Section 1: Popular gTLDs (generic) ──────────────────────────
/// ── Section 2: New gTLDs (popular modern extensions) ────────────
/// ── Section 3: Country-code TLDs (ccTLDs) ──────────────────────
/// ── Section 4: Sponsored / Other ────────────────────────────────
pub const BUILTIN_TLDS: &[&str] = &[
    // ── Popular gTLDs ───────────────────────────────────────────
    ".com",
    ".net",
    ".org",
    ".info",
    ".biz",
    ".name",
    ".pro",
    ".mobi",
    ".tel",
    // ── New gTLDs — tech / developer ────────────────────────────
    ".app",
    ".dev",
    ".ai", // Anguilla ccTLD, widely used for AI/tech
    ".io", // British Indian Ocean, widely used for tech
    ".co", // Colombia ccTLD, widely used as .com alternative
    ".me", // Montenegro ccTLD, widely used for personal branding
    ".xyz",
    ".tech",
    ".cloud",
    ".digital",
    ".systems",
    ".network",
    ".code",
    ".bot",
    ".computer",
    ".software",
    ".engineering",
    ".science",
    ".technology",
    ".directory",
    ".tools",
    ".top",
    // ── New gTLDs — business / commerce ────────────────────────
    ".club",
    ".online",
    ".site",
    ".store",
    ".shop",
    ".market",
    ".marketing",
    ".finance",
    ".financial",
    ".capital",
    ".exchange",
    ".bank",
    ".money",
    ".cash",
    ".fund",
    ".trade",
    ".bid",
    ".auction",
    ".discount",
    ".deals",
    // ── New gTLDs — creative / media ───────────────────────────
    ".fun",
    ".space",
    ".blog",
    ".media",
    ".news",
    ".press",
    ".live",
    ".studio",
    ".design",
    ".photography",
    ".gallery",
    ".graphics",
    ".photo",
    ".pics",
    ".art",
    ".video",
    ".film",
    ".music",
    ".radio",
    ".games",
    ".gaming",
    ".play",
    // ── New gTLDs — lifestyle / social ─────────────────────────
    ".guru",
    ".ninja",
    ".rocks",
    ".world",
    ".today",
    ".life",
    ".love",
    ".cool",
    ".lol",
    ".wow",
    ".pink",
    ".blue",
    ".red",
    ".green",
    ".black",
    ".gold",
    ".silver",
    // ── New gTLDs — professional / services ────────────────────
    ".solutions",
    ".agency",
    ".company",
    ".ventures",
    ".academy",
    ".management",
    ".builders",
    ".institute",
    ".consulting",
    ".expert",
    ".services",
    ".support",
    ".tips",
    ".guide",
    ".zone",
    ".works",
    ".place",
    ".foundation",
    ".center",
    ".community",
    ".partners",
    ".associates",
    ".international",
    ".properties",
    ".catering",
    ".restaurant",
    ".menu",
    ".coffee",
    ".beer",
    ".wine",
    ".food",
    // ── New gTLDs — geography / web / communication ────────────
    ".city",
    ".earth",
    ".global",
    ".land",
    ".website",
    ".domains",
    ".host",
    ".server",
    ".link",
    ".click",
    ".download",
    ".email",
    ".contact",
    // ── New gTLDs — identity / branding ────────────────────────
    ".moe",
    ".one",
    ".zero",
    ".vip",
    ".pet",
    ".dog",
    ".baby",
    ".kids",
    ".best",
    ".win",
    ".bet",
    ".bond",
    ".rip",
    // ── Country-code TLDs (ccTLDs) — Americas ─────────────────
    ".us",
    ".ca",
    ".mx",
    ".br",
    ".ar",
    ".cl",
    ".pe",
    ".ve",
    ".ec",
    ".uy",
    ".py",
    ".bo",
    ".cr",
    ".pa",
    ".gt",
    ".hn",
    ".sv",
    ".ni",
    ".cu",
    ".do",
    ".ht",
    ".jm",
    ".tt",
    ".bb",
    ".gd",
    ".lc",
    ".vc",
    ".ag",
    ".bs",
    ".bz",
    ".dm",
    ".kn",
    ".ms",
    ".tc",
    ".vg",
    // ── Country-code TLDs — Europe ────────────────────────────
    ".uk",
    ".de",
    ".fr",
    ".it",
    ".es",
    ".nl",
    ".se",
    ".no",
    ".ch",
    ".at",
    ".be",
    ".dk",
    ".pl",
    ".cz",
    ".eu",
    ".ie",
    ".pt",
    ".fi",
    ".gr",
    ".hu",
    ".ro",
    ".bg",
    ".hr",
    ".sk",
    ".si",
    ".lt",
    ".lv",
    ".ee",
    ".lu",
    ".mt",
    ".cy",
    ".is",
    ".li",
    ".mc",
    ".va",
    ".sm",
    ".ad",
    ".fo",
    ".gi",
    ".gg",
    ".je",
    ".im",
    ".ax",
    // ── Country-code TLDs — Asia-Pacific ──────────────────────
    ".jp",
    ".cn",
    ".kr",
    ".in",
    ".au",
    ".nz",
    ".sg",
    ".hk",
    ".tw",
    ".my",
    ".th",
    ".id",
    ".ph",
    ".vn",
    ".pk",
    ".bd",
    ".lk",
    ".mm",
    ".kh",
    ".la",
    ".np",
    ".bn",
    ".tl",
    ".mv",
    ".mn",
    ".kz",
    ".uz",
    ".kg",
    ".tj",
    ".tm",
    ".af",
    ".ir",
    ".iq",
    ".ps",
    // ── Country-code TLDs — Middle East / Africa ──────────────
    ".il",
    ".ae",
    ".sa",
    ".tr",
    ".qa",
    ".kw",
    ".bh",
    ".om",
    ".jo",
    ".lb",
    ".eg",
    ".za",
    ".ng",
    ".ke",
    ".ma",
    ".tn",
    ".dz",
    ".gh",
    ".tz",
    ".ug",
    ".et",
    ".mg",
    ".mu",
    ".rw",
    ".sn",
    ".cm",
    ".ci",
    ".ly",
    ".sd",
    ".cd",
    ".ao",
    ".mz",
    ".zw",
    ".bw",
    ".na",
    // ── Popular ccTLDs used as gTLDs (not listed elsewhere)
    ".tv",
    ".cc",
    ".ws",
    ".fm",
    ".am",
    ".dj",
    ".to",
    ".nu",
    // ── Sponsored / Other TLDs ────────────────────────────────
    ".edu",
    ".gov",
    ".mil",
    ".int",
    ".museum",
    ".aero",
    ".coop",
    ".travel",
    ".jobs",
    ".cat",
    ".post",
    ".asia",
    ".arpa",
    ".example",
    ".test",
    ".localhost",
    ".invalid",
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

    /// Get common TLDs (top 30 by popularity)
    pub fn get_common_tlds(&self) -> Vec<String> {
        BUILTIN_TLDS[..30].iter().map(|s| s.to_string()).collect()
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
        assert!(tlds.len() > 200);
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
        assert_eq!(common.len(), 30);
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
