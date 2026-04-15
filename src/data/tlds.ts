/**
 * Comprehensive TLD (Top-Level Domain) database for domain scanning.
 *
 * Sources: IANA Root Zone Database, Namecheap 2025 Domain Insights,
 * GoDaddy, Cloudflare Registrar, TLD-List.com
 *
 * Total: 280+ entries covering gTLDs, new gTLDs, ccTLDs, and sponsored/other TLDs.
 *
 * Category types:
 *   "gtld"     — Classic generic TLDs (.com, .net, .org, etc.)
 *   "new_gtld" — New generic TLDs introduced since 2012 ICANN expansion
 *   "cctld"    — Country-code TLDs (two-letter codes per ISO 3166-1)
 *   "other"    — Sponsored, infrastructure, and reserved TLDs
 */

export type TldCategory = "gtld" | "new_gtld" | "cctld" | "other";

export interface TldEntry {
  tld: string;
  popular: boolean;
  category: TldCategory;
}

export const TLD_LIST: TldEntry[] = [
  // ═══════════════════════════════════════════════════════════════
  // POPULAR gTLDs (Classic Generic Top-Level Domains)
  // ═══════════════════════════════════════════════════════════════
  { tld: ".com", popular: true, category: "gtld" },
  { tld: ".net", popular: true, category: "gtld" },
  { tld: ".org", popular: true, category: "gtld" },
  { tld: ".info", popular: true, category: "gtld" },
  { tld: ".biz", popular: true, category: "gtld" },
  { tld: ".name", popular: false, category: "gtld" },
  { tld: ".pro", popular: false, category: "gtld" },
  { tld: ".mobi", popular: false, category: "gtld" },
  { tld: ".tel", popular: false, category: "gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Tech / Developer
  // ═══════════════════════════════════════════════════════════════
  { tld: ".app", popular: true, category: "new_gtld" },
  { tld: ".dev", popular: true, category: "new_gtld" },
  { tld: ".ai", popular: true, category: "cctld" },   // Anguilla ccTLD, used as tech gTLD
  { tld: ".io", popular: true, category: "cctld" },   // British Indian Ocean, used as tech gTLD
  { tld: ".co", popular: true, category: "cctld" },   // Colombia ccTLD, used as gTLD
  { tld: ".xyz", popular: true, category: "new_gtld" },
  { tld: ".tech", popular: true, category: "new_gtld" },
  { tld: ".cloud", popular: false, category: "new_gtld" },
  { tld: ".digital", popular: false, category: "new_gtld" },
  { tld: ".systems", popular: false, category: "new_gtld" },
  { tld: ".network", popular: false, category: "new_gtld" },
  { tld: ".code", popular: false, category: "new_gtld" },
  { tld: ".bot", popular: false, category: "new_gtld" },
  { tld: ".computer", popular: false, category: "new_gtld" },
  { tld: ".software", popular: false, category: "new_gtld" },
  { tld: ".engineering", popular: false, category: "new_gtld" },
  { tld: ".science", popular: false, category: "new_gtld" },
  { tld: ".technology", popular: false, category: "new_gtld" },
  { tld: ".directory", popular: false, category: "new_gtld" },
  { tld: ".tools", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Business / Commerce
  // ═══════════════════════════════════════════════════════════════
  { tld: ".club", popular: true, category: "new_gtld" },
  { tld: ".online", popular: true, category: "new_gtld" },
  { tld: ".site", popular: true, category: "new_gtld" },
  { tld: ".store", popular: true, category: "new_gtld" },
  { tld: ".shop", popular: false, category: "new_gtld" },
  { tld: ".market", popular: false, category: "new_gtld" },
  { tld: ".marketing", popular: false, category: "new_gtld" },
  { tld: ".finance", popular: false, category: "new_gtld" },
  { tld: ".financial", popular: false, category: "new_gtld" },
  { tld: ".capital", popular: false, category: "new_gtld" },
  { tld: ".exchange", popular: false, category: "new_gtld" },
  { tld: ".bank", popular: false, category: "new_gtld" },
  { tld: ".money", popular: false, category: "new_gtld" },
  { tld: ".cash", popular: false, category: "new_gtld" },
  { tld: ".fund", popular: false, category: "new_gtld" },
  { tld: ".trade", popular: false, category: "new_gtld" },
  { tld: ".bid", popular: false, category: "new_gtld" },
  { tld: ".auction", popular: false, category: "new_gtld" },
  { tld: ".discount", popular: false, category: "new_gtld" },
  { tld: ".deals", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Creative / Media
  // ═══════════════════════════════════════════════════════════════
  { tld: ".fun", popular: false, category: "new_gtld" },
  { tld: ".space", popular: false, category: "new_gtld" },
  { tld: ".top", popular: false, category: "new_gtld" },
  { tld: ".blog", popular: false, category: "new_gtld" },
  { tld: ".media", popular: false, category: "new_gtld" },
  { tld: ".news", popular: false, category: "new_gtld" },
  { tld: ".press", popular: false, category: "new_gtld" },
  { tld: ".live", popular: false, category: "new_gtld" },
  { tld: ".studio", popular: false, category: "new_gtld" },
  { tld: ".design", popular: false, category: "new_gtld" },
  { tld: ".photography", popular: false, category: "new_gtld" },
  { tld: ".gallery", popular: false, category: "new_gtld" },
  { tld: ".graphics", popular: false, category: "new_gtld" },
  { tld: ".photo", popular: false, category: "new_gtld" },
  { tld: ".pics", popular: false, category: "new_gtld" },
  { tld: ".art", popular: false, category: "new_gtld" },
  { tld: ".video", popular: false, category: "new_gtld" },
  { tld: ".film", popular: false, category: "new_gtld" },
  { tld: ".music", popular: false, category: "new_gtld" },
  { tld: ".radio", popular: false, category: "new_gtld" },
  { tld: ".games", popular: false, category: "new_gtld" },
  { tld: ".gaming", popular: false, category: "new_gtld" },
  { tld: ".play", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Lifestyle / Social
  // ═══════════════════════════════════════════════════════════════
  { tld: ".guru", popular: false, category: "new_gtld" },
  { tld: ".ninja", popular: false, category: "new_gtld" },
  { tld: ".rocks", popular: false, category: "new_gtld" },
  { tld: ".world", popular: false, category: "new_gtld" },
  { tld: ".today", popular: false, category: "new_gtld" },
  { tld: ".life", popular: false, category: "new_gtld" },
  { tld: ".love", popular: false, category: "new_gtld" },
  { tld: ".cool", popular: false, category: "new_gtld" },
  { tld: ".lol", popular: false, category: "new_gtld" },
  { tld: ".wow", popular: false, category: "new_gtld" },
  { tld: ".pink", popular: false, category: "new_gtld" },
  { tld: ".blue", popular: false, category: "new_gtld" },
  { tld: ".red", popular: false, category: "new_gtld" },
  { tld: ".green", popular: false, category: "new_gtld" },
  { tld: ".black", popular: false, category: "new_gtld" },
  { tld: ".gold", popular: false, category: "new_gtld" },
  { tld: ".silver", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Professional / Services
  // ═══════════════════════════════════════════════════════════════
  { tld: ".solutions", popular: false, category: "new_gtld" },
  { tld: ".agency", popular: false, category: "new_gtld" },
  { tld: ".company", popular: false, category: "new_gtld" },
  { tld: ".ventures", popular: false, category: "new_gtld" },
  { tld: ".academy", popular: false, category: "new_gtld" },
  { tld: ".management", popular: false, category: "new_gtld" },
  { tld: ".builders", popular: false, category: "new_gtld" },
  { tld: ".institute", popular: false, category: "new_gtld" },
  { tld: ".consulting", popular: false, category: "new_gtld" },
  { tld: ".expert", popular: false, category: "new_gtld" },
  { tld: ".services", popular: false, category: "new_gtld" },
  { tld: ".support", popular: false, category: "new_gtld" },
  { tld: ".tips", popular: false, category: "new_gtld" },
  { tld: ".guide", popular: false, category: "new_gtld" },
  { tld: ".zone", popular: false, category: "new_gtld" },
  { tld: ".works", popular: false, category: "new_gtld" },
  { tld: ".place", popular: false, category: "new_gtld" },
  { tld: ".foundation", popular: false, category: "new_gtld" },
  { tld: ".center", popular: false, category: "new_gtld" },
  { tld: ".community", popular: false, category: "new_gtld" },
  { tld: ".partners", popular: false, category: "new_gtld" },
  { tld: ".associates", popular: false, category: "new_gtld" },
  { tld: ".international", popular: false, category: "new_gtld" },
  { tld: ".properties", popular: false, category: "new_gtld" },
  { tld: ".catering", popular: false, category: "new_gtld" },
  { tld: ".restaurant", popular: false, category: "new_gtld" },
  { tld: ".menu", popular: false, category: "new_gtld" },
  { tld: ".coffee", popular: false, category: "new_gtld" },
  { tld: ".beer", popular: false, category: "new_gtld" },
  { tld: ".wine", popular: false, category: "new_gtld" },
  { tld: ".food", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Geography / Web / Communication
  // ═══════════════════════════════════════════════════════════════
  { tld: ".city", popular: false, category: "new_gtld" },
  { tld: ".earth", popular: false, category: "new_gtld" },
  { tld: ".global", popular: false, category: "new_gtld" },
  { tld: ".land", popular: false, category: "new_gtld" },
  { tld: ".website", popular: false, category: "new_gtld" },
  { tld: ".domains", popular: false, category: "new_gtld" },
  { tld: ".host", popular: false, category: "new_gtld" },
  { tld: ".server", popular: false, category: "new_gtld" },
  { tld: ".link", popular: false, category: "new_gtld" },
  { tld: ".click", popular: false, category: "new_gtld" },
  { tld: ".download", popular: false, category: "new_gtld" },
  { tld: ".email", popular: false, category: "new_gtld" },
  { tld: ".contact", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // NEW gTLDs — Identity / Branding
  // ═══════════════════════════════════════════════════════════════
  { tld: ".me", popular: true, category: "cctld" },    // Montenegro ccTLD, used as gTLD
  { tld: ".moe", popular: false, category: "new_gtld" },
  { tld: ".one", popular: false, category: "new_gtld" },
  { tld: ".zero", popular: false, category: "new_gtld" },
  { tld: ".vip", popular: false, category: "new_gtld" },
  { tld: ".pet", popular: false, category: "new_gtld" },
  { tld: ".dog", popular: false, category: "new_gtld" },
  { tld: ".baby", popular: false, category: "new_gtld" },
  { tld: ".kids", popular: false, category: "new_gtld" },
  { tld: ".best", popular: false, category: "new_gtld" },
  { tld: ".win", popular: false, category: "new_gtld" },
  { tld: ".bet", popular: false, category: "new_gtld" },
  { tld: ".bond", popular: false, category: "new_gtld" },
  { tld: ".rip", popular: false, category: "new_gtld" },

  // ═══════════════════════════════════════════════════════════════
  // COUNTRY-CODE TLDs (ccTLDs) — Americas
  // ═══════════════════════════════════════════════════════════════
  { tld: ".us", popular: true, category: "cctld" },
  { tld: ".ca", popular: true, category: "cctld" },
  { tld: ".mx", popular: false, category: "cctld" },
  { tld: ".br", popular: true, category: "cctld" },
  { tld: ".ar", popular: false, category: "cctld" },
  { tld: ".cl", popular: false, category: "cctld" },
  { tld: ".pe", popular: false, category: "cctld" },
  { tld: ".ve", popular: false, category: "cctld" },
  { tld: ".ec", popular: false, category: "cctld" },
  { tld: ".uy", popular: false, category: "cctld" },
  { tld: ".py", popular: false, category: "cctld" },
  { tld: ".bo", popular: false, category: "cctld" },
  { tld: ".cr", popular: false, category: "cctld" },
  { tld: ".pa", popular: false, category: "cctld" },
  { tld: ".gt", popular: false, category: "cctld" },
  { tld: ".hn", popular: false, category: "cctld" },
  { tld: ".sv", popular: false, category: "cctld" },
  { tld: ".ni", popular: false, category: "cctld" },
  { tld: ".cu", popular: false, category: "cctld" },
  { tld: ".do", popular: false, category: "cctld" },
  { tld: ".ht", popular: false, category: "cctld" },
  { tld: ".jm", popular: false, category: "cctld" },
  { tld: ".tt", popular: false, category: "cctld" },
  { tld: ".bb", popular: false, category: "cctld" },
  { tld: ".gd", popular: false, category: "cctld" },
  { tld: ".lc", popular: false, category: "cctld" },
  { tld: ".vc", popular: false, category: "cctld" },
  { tld: ".ag", popular: false, category: "cctld" },
  { tld: ".bs", popular: false, category: "cctld" },
  { tld: ".bz", popular: false, category: "cctld" },
  { tld: ".dm", popular: false, category: "cctld" },
  { tld: ".kn", popular: false, category: "cctld" },
  { tld: ".ms", popular: false, category: "cctld" },
  { tld: ".tc", popular: false, category: "cctld" },
  { tld: ".vg", popular: false, category: "cctld" },

  // ═══════════════════════════════════════════════════════════════
  // COUNTRY-CODE TLDs — Europe
  // ═══════════════════════════════════════════════════════════════
  { tld: ".uk", popular: true, category: "cctld" },
  { tld: ".de", popular: true, category: "cctld" },
  { tld: ".fr", popular: true, category: "cctld" },
  { tld: ".it", popular: true, category: "cctld" },
  { tld: ".es", popular: true, category: "cctld" },
  { tld: ".nl", popular: false, category: "cctld" },
  { tld: ".se", popular: false, category: "cctld" },
  { tld: ".no", popular: false, category: "cctld" },
  { tld: ".ch", popular: false, category: "cctld" },
  { tld: ".at", popular: false, category: "cctld" },
  { tld: ".be", popular: false, category: "cctld" },
  { tld: ".dk", popular: false, category: "cctld" },
  { tld: ".pl", popular: false, category: "cctld" },
  { tld: ".cz", popular: false, category: "cctld" },
  { tld: ".eu", popular: false, category: "cctld" },
  { tld: ".ie", popular: false, category: "cctld" },
  { tld: ".pt", popular: false, category: "cctld" },
  { tld: ".fi", popular: false, category: "cctld" },
  { tld: ".gr", popular: false, category: "cctld" },
  { tld: ".hu", popular: false, category: "cctld" },
  { tld: ".ro", popular: false, category: "cctld" },
  { tld: ".bg", popular: false, category: "cctld" },
  { tld: ".hr", popular: false, category: "cctld" },
  { tld: ".sk", popular: false, category: "cctld" },
  { tld: ".si", popular: false, category: "cctld" },
  { tld: ".lt", popular: false, category: "cctld" },
  { tld: ".lv", popular: false, category: "cctld" },
  { tld: ".ee", popular: false, category: "cctld" },
  { tld: ".lu", popular: false, category: "cctld" },
  { tld: ".mt", popular: false, category: "cctld" },
  { tld: ".cy", popular: false, category: "cctld" },
  { tld: ".is", popular: false, category: "cctld" },
  { tld: ".li", popular: false, category: "cctld" },
  { tld: ".mc", popular: false, category: "cctld" },
  { tld: ".va", popular: false, category: "cctld" },
  { tld: ".sm", popular: false, category: "cctld" },
  { tld: ".ad", popular: false, category: "cctld" },
  { tld: ".fo", popular: false, category: "cctld" },
  { tld: ".gi", popular: false, category: "cctld" },
  { tld: ".gg", popular: false, category: "cctld" },
  { tld: ".je", popular: false, category: "cctld" },
  { tld: ".im", popular: false, category: "cctld" },
  { tld: ".ax", popular: false, category: "cctld" },

  // ═══════════════════════════════════════════════════════════════
  // COUNTRY-CODE TLDs — Asia-Pacific
  // ═══════════════════════════════════════════════════════════════
  { tld: ".jp", popular: true, category: "cctld" },
  { tld: ".cn", popular: true, category: "cctld" },
  { tld: ".kr", popular: false, category: "cctld" },
  { tld: ".in", popular: true, category: "cctld" },
  { tld: ".au", popular: true, category: "cctld" },
  { tld: ".nz", popular: false, category: "cctld" },
  { tld: ".sg", popular: false, category: "cctld" },
  { tld: ".hk", popular: false, category: "cctld" },
  { tld: ".tw", popular: false, category: "cctld" },
  { tld: ".my", popular: false, category: "cctld" },
  { tld: ".th", popular: false, category: "cctld" },
  { tld: ".id", popular: false, category: "cctld" },
  { tld: ".ph", popular: false, category: "cctld" },
  { tld: ".vn", popular: false, category: "cctld" },
  { tld: ".pk", popular: false, category: "cctld" },
  { tld: ".bd", popular: false, category: "cctld" },
  { tld: ".lk", popular: false, category: "cctld" },
  { tld: ".mm", popular: false, category: "cctld" },
  { tld: ".kh", popular: false, category: "cctld" },
  { tld: ".la", popular: false, category: "cctld" },
  { tld: ".np", popular: false, category: "cctld" },
  { tld: ".bn", popular: false, category: "cctld" },
  { tld: ".tl", popular: false, category: "cctld" },
  { tld: ".mv", popular: false, category: "cctld" },
  { tld: ".mn", popular: false, category: "cctld" },
  { tld: ".kz", popular: false, category: "cctld" },
  { tld: ".uz", popular: false, category: "cctld" },
  { tld: ".kg", popular: false, category: "cctld" },
  { tld: ".tj", popular: false, category: "cctld" },
  { tld: ".tm", popular: false, category: "cctld" },
  { tld: ".af", popular: false, category: "cctld" },
  { tld: ".ir", popular: false, category: "cctld" },
  { tld: ".iq", popular: false, category: "cctld" },
  { tld: ".ps", popular: false, category: "cctld" },

  // ═══════════════════════════════════════════════════════════════
  // COUNTRY-CODE TLDs — Middle East / Africa
  // ═══════════════════════════════════════════════════════════════
  { tld: ".il", popular: false, category: "cctld" },
  { tld: ".ae", popular: false, category: "cctld" },
  { tld: ".sa", popular: false, category: "cctld" },
  { tld: ".tr", popular: false, category: "cctld" },
  { tld: ".qa", popular: false, category: "cctld" },
  { tld: ".kw", popular: false, category: "cctld" },
  { tld: ".bh", popular: false, category: "cctld" },
  { tld: ".om", popular: false, category: "cctld" },
  { tld: ".jo", popular: false, category: "cctld" },
  { tld: ".lb", popular: false, category: "cctld" },
  { tld: ".eg", popular: false, category: "cctld" },
  { tld: ".za", popular: false, category: "cctld" },
  { tld: ".ng", popular: false, category: "cctld" },
  { tld: ".ke", popular: false, category: "cctld" },
  { tld: ".ma", popular: false, category: "cctld" },
  { tld: ".tn", popular: false, category: "cctld" },
  { tld: ".dz", popular: false, category: "cctld" },
  { tld: ".gh", popular: false, category: "cctld" },
  { tld: ".tz", popular: false, category: "cctld" },
  { tld: ".ug", popular: false, category: "cctld" },
  { tld: ".et", popular: false, category: "cctld" },
  { tld: ".mg", popular: false, category: "cctld" },
  { tld: ".mu", popular: false, category: "cctld" },
  { tld: ".rw", popular: false, category: "cctld" },
  { tld: ".sn", popular: false, category: "cctld" },
  { tld: ".cm", popular: false, category: "cctld" },
  { tld: ".ci", popular: false, category: "cctld" },
  { tld: ".ly", popular: false, category: "cctld" },
  { tld: ".sd", popular: false, category: "cctld" },
  { tld: ".cd", popular: false, category: "cctld" },
  { tld: ".ao", popular: false, category: "cctld" },
  { tld: ".mz", popular: false, category: "cctld" },
  { tld: ".zw", popular: false, category: "cctld" },
  { tld: ".bw", popular: false, category: "cctld" },
  { tld: ".na", popular: false, category: "cctld" },

  // ═══════════════════════════════════════════════════════════════
  // POPULAR ccTLDs commonly used as generic TLDs
  // ═══════════════════════════════════════════════════════════════
  { tld: ".tv", popular: true, category: "cctld" },    // Tuvalu — used for video/TV
  { tld: ".cc", popular: true, category: "cctld" },    // Cocos Islands — used as generic
  { tld: ".ws", popular: true, category: "cctld" },    // Samoa — "website"
  { tld: ".fm", popular: false, category: "cctld" },   // Micronesia — used for radio/music
  { tld: ".am", popular: false, category: "cctld" },   // Armenia — used for radio/music
  { tld: ".dj", popular: false, category: "cctld" },   // Djibouti — used for DJ/music
  { tld: ".to", popular: false, category: "cctld" },   // Tonga — used as generic
  { tld: ".nu", popular: false, category: "cctld" },   // Niue — popular in Scandinavia ("nu" = "now")

  // ═══════════════════════════════════════════════════════════════
  // RU & CIS
  // ═══════════════════════════════════════════════════════════════
  { tld: ".ru", popular: true, category: "cctld" },
  { tld: ".ua", popular: false, category: "cctld" },
  { tld: ".by", popular: false, category: "cctld" },
  { tld: ".ge", popular: false, category: "cctld" },
  { tld: ".az", popular: false, category: "cctld" },
  { tld: ".md", popular: false, category: "cctld" },

  // ═══════════════════════════════════════════════════════════════
  // SPONSORED / OTHER TLDs
  // ═══════════════════════════════════════════════════════════════
  { tld: ".edu", popular: false, category: "other" },
  { tld: ".gov", popular: false, category: "other" },
  { tld: ".mil", popular: false, category: "other" },
  { tld: ".int", popular: false, category: "other" },
  { tld: ".museum", popular: false, category: "other" },
  { tld: ".aero", popular: false, category: "other" },
  { tld: ".coop", popular: false, category: "other" },
  { tld: ".travel", popular: false, category: "other" },
  { tld: ".jobs", popular: false, category: "other" },
  { tld: ".cat", popular: false, category: "other" },
  { tld: ".post", popular: false, category: "other" },
  { tld: ".asia", popular: false, category: "other" },
  { tld: ".arpa", popular: false, category: "other" },
  { tld: ".example", popular: false, category: "other" },
  { tld: ".test", popular: false, category: "other" },
  { tld: ".localhost", popular: false, category: "other" },
  { tld: ".invalid", popular: false, category: "other" },
];

/** Get only TLDs marked as popular */
export const POPULAR_TLDS = TLD_LIST.filter((t) => t.popular);

/** Get TLDs grouped by category */
export const TLDS_BY_CATEGORY = {
  gtld: TLD_LIST.filter((t) => t.category === "gtld"),
  new_gtld: TLD_LIST.filter((t) => t.category === "new_gtld"),
  cctld: TLD_LIST.filter((t) => t.category === "cctld"),
  other: TLD_LIST.filter((t) => t.category === "other"),
} as const;

/** Total count */
export const TLD_COUNT = TLD_LIST.length;
