//! Currencies: ISO code, symbol, English name and the number of decimals shown.

pub struct Cur {
    pub code: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    /// Digits after the point when displayed (0 for JPY, 3 for KWD).
    pub dec: u32,
}

const fn c(code: &'static str, symbol: &'static str, name: &'static str, dec: u32) -> Cur {
    Cur { code, symbol, name, dec }
}

/// The three currencies shown first in every picker.
pub const TOP: [&str; 3] = ["UAH", "USD", "EUR"];

/// Every currency the app offers. The first three are [`TOP`]; the rest is alphabetical by code.
pub static ALL: &[Cur] = &[
    c("UAH", "₴", "Ukrainian Hryvnia", 2),
    c("USD", "$", "US Dollar", 2),
    c("EUR", "€", "Euro", 2),
    c("AED", "AED", "UAE Dirham", 2),
    c("ALL", "L", "Albanian Lek", 2),
    c("AMD", "֏", "Armenian Dram", 2),
    c("ARS", "ARS$", "Argentine Peso", 2),
    c("AUD", "A$", "Australian Dollar", 2),
    c("AZN", "₼", "Azerbaijani Manat", 2),
    c("BAM", "KM", "Bosnia-Herzegovina Mark", 2),
    c("BDT", "৳", "Bangladeshi Taka", 2),
    c("BGN", "лв", "Bulgarian Lev", 2),
    c("BHD", "BHD", "Bahraini Dinar", 3),
    c("BRL", "R$", "Brazilian Real", 2),
    c("CAD", "C$", "Canadian Dollar", 2),
    c("CHF", "CHF", "Swiss Franc", 2),
    c("CLP", "CLP$", "Chilean Peso", 0),
    c("CNY", "CN¥", "Chinese Yuan", 2),
    c("COP", "COL$", "Colombian Peso", 2),
    c("CZK", "Kč", "Czech Koruna", 2),
    c("DKK", "kr", "Danish Krone", 2),
    c("DZD", "DA", "Algerian Dinar", 2),
    c("EGP", "E£", "Egyptian Pound", 2),
    c("GBP", "£", "British Pound", 2),
    c("GEL", "₾", "Georgian Lari", 2),
    c("GHS", "GH₵", "Ghanaian Cedi", 2),
    c("HKD", "HK$", "Hong Kong Dollar", 2),
    c("HUF", "Ft", "Hungarian Forint", 2),
    c("IDR", "Rp", "Indonesian Rupiah", 2),
    c("ILS", "₪", "Israeli Shekel", 2),
    c("INR", "₹", "Indian Rupee", 2),
    c("IQD", "IQD", "Iraqi Dinar", 0),
    c("ISK", "kr", "Icelandic Krona", 0),
    c("JOD", "JOD", "Jordanian Dinar", 3),
    c("JPY", "¥", "Japanese Yen", 0),
    c("KES", "KSh", "Kenyan Shilling", 2),
    c("KGS", "сом", "Kyrgyzstani Som", 2),
    c("KRW", "₩", "South Korean Won", 0),
    c("KWD", "KWD", "Kuwaiti Dinar", 3),
    c("KZT", "₸", "Kazakhstani Tenge", 2),
    c("LBP", "L£", "Lebanese Pound", 2),
    c("LKR", "Rs", "Sri Lankan Rupee", 2),
    c("MAD", "MAD", "Moroccan Dirham", 2),
    c("MDL", "L", "Moldovan Leu", 2),
    c("MKD", "ден", "Macedonian Denar", 2),
    c("MXN", "MX$", "Mexican Peso", 2),
    c("MYR", "RM", "Malaysian Ringgit", 2),
    c("NGN", "₦", "Nigerian Naira", 2),
    c("NOK", "kr", "Norwegian Krone", 2),
    c("NZD", "NZ$", "New Zealand Dollar", 2),
    c("OMR", "OMR", "Omani Rial", 3),
    c("PEN", "S/", "Peruvian Sol", 2),
    c("PHP", "₱", "Philippine Peso", 2),
    c("PKR", "Rs", "Pakistani Rupee", 2),
    c("PLN", "zł", "Polish Zloty", 2),
    c("QAR", "QAR", "Qatari Riyal", 2),
    c("RON", "lei", "Romanian Leu", 2),
    c("RSD", "din", "Serbian Dinar", 2),
    c("SAR", "SAR", "Saudi Riyal", 2),
    c("SEK", "kr", "Swedish Krona", 2),
    c("SGD", "S$", "Singapore Dollar", 2),
    c("THB", "฿", "Thai Baht", 2),
    c("TND", "TND", "Tunisian Dinar", 3),
    c("TRY", "₺", "Turkish Lira", 2),
    c("TWD", "NT$", "New Taiwan Dollar", 2),
    c("UGX", "USh", "Ugandan Shilling", 0),
    c("UYU", "$U", "Uruguayan Peso", 2),
    c("UZS", "soʻm", "Uzbekistani Som", 2),
    c("VND", "₫", "Vietnamese Dong", 0),
    c("BBD", "Bds$", "Barbadian Dollar", 2),
    c("BND", "B$", "Brunei Dollar", 2),
    c("BOB", "Bs.", "Bolivian Boliviano", 2),
    c("BWP", "P", "Botswana Pula", 2),
    c("CRC", "₡", "Costa Rican Colon", 2),
    c("DOP", "RD$", "Dominican Peso", 2),
    c("ETB", "Br", "Ethiopian Birr", 2),
    c("FJD", "FJ$", "Fijian Dollar", 2),
    c("GTQ", "Q", "Guatemalan Quetzal", 2),
    c("JMD", "J$", "Jamaican Dollar", 2),
    c("KHR", "៛", "Cambodian Riel", 2),
    c("LAK", "₭", "Lao Kip", 2),
    c("MMK", "K", "Myanmar Kyat", 2),
    c("MNT", "₮", "Mongolian Tugrik", 2),
    c("MUR", "₨", "Mauritian Rupee", 2),
    c("NPR", "रू", "Nepalese Rupee", 2),
    c("PAB", "B/.", "Panamanian Balboa", 2),
    c("PYG", "₲", "Paraguayan Guarani", 0),
    c("TTD", "TT$", "Trinidad Dollar", 2),
    c("TZS", "TSh", "Tanzanian Shilling", 2),
    c("XAF", "FCFA", "Central African CFA Franc", 0),
    c("XOF", "CFA", "West African CFA Franc", 0),
    c("ZAR", "R", "South African Rand", 2),
];

/// Currencies that are never offered, converted or used as a default.
pub const EXCLUDED: [&str; 2] = ["RUB", "BYN"];

#[allow(dead_code)]
pub fn is_excluded(code: &str) -> bool {
    EXCLUDED.iter().any(|e| e.eq_ignore_ascii_case(code))
}

/// The frequent block of a picker: recently used currencies first, then USD, EUR and UAH (no repeats).
fn frequent_codes(recent: &[String]) -> Vec<&'static Cur> {
    let mut out: Vec<&'static Cur> = Vec::new();
    for code in recent.iter().map(String::as_str).chain(TOP) {
        if let Some(c) = find(code) {
            if !out.iter().any(|o| o.code == c.code) {
                out.push(c);
            }
        }
    }
    out
}

/// How many entries the frequent block of [`ordered`] has.
pub fn frequent(recent: &[String]) -> usize {
    frequent_codes(recent).len()
}

/// Every currency: the frequent block first, then the rest in list order.
pub fn ordered(recent: &[String]) -> Vec<&'static Cur> {
    let mut out = frequent_codes(recent);
    for c in ALL {
        if !out.iter().any(|o| o.code == c.code) {
            out.push(c);
        }
    }
    out
}

pub fn find(code: &str) -> Option<&'static Cur> {
    ALL.iter().find(|c| c.code.eq_ignore_ascii_case(code))
}

/// Symbol for a code, or the code itself when it is not in the list.
pub fn symbol(code: &str) -> String {
    find(code).map(|c| c.symbol.to_string()).unwrap_or_else(|| code.to_string())
}

/// Digits shown after the point for a currency (2 when unknown).
pub fn decimals(code: &str) -> u32 {
    find(code).map(|c| c.dec).unwrap_or(2)
}

/// Case-insensitive search over code, name and symbol; empty query matches everything.
pub fn matches(c: &Cur, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || c.code.to_lowercase().contains(&q) || c.name.to_lowercase().contains(&q) || c.symbol.to_lowercase().contains(&q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_three_come_first_and_codes_are_unique() {
        for (i, code) in TOP.iter().enumerate() {
            assert_eq!(ALL[i].code, *code);
        }
        let mut codes: Vec<_> = ALL.iter().map(|c| c.code).collect();
        codes.sort();
        let n = codes.len();
        codes.dedup();
        assert_eq!(codes.len(), n);
    }

    #[test]
    fn recent_currencies_come_first_without_repeats() {
        let recent = vec!["PLN".to_string(), "USD".to_string(), "RUB".to_string()];
        let o = ordered(&recent);
        let codes: Vec<&str> = o.iter().map(|c| c.code).take(4).collect();
        assert_eq!(codes, ["PLN", "USD", "UAH", "EUR"]);
        assert_eq!(frequent(&recent), 4);
        assert_eq!(o.len(), ALL.len());
        assert!(o.iter().all(|c| c.code != "RUB"));
    }

    #[test]
    fn excluded_currencies_never_appear() {
        for code in EXCLUDED {
            assert!(find(code).is_none());
            assert!(ALL.iter().all(|c| c.code != code));
            assert!(ALL.iter().all(|c| !matches(c, code) || c.code != code));
            assert!(is_excluded(&code.to_lowercase()));
        }
        for want in ["USD", "EUR", "UAH", "GBP", "PLN", "CZK", "CHF", "JPY", "CNY", "CAD", "AUD", "SEK", "NOK", "DKK", "TRY", "GEL", "KZT", "ILS", "INR", "BRL"] {
            assert!(find(want).is_some(), "{want}");
        }
    }

    #[test]
    fn lookup_and_search() {
        assert_eq!(symbol("uah"), "₴");
        assert_eq!(symbol("XYZ"), "XYZ");
        assert_eq!(decimals("JPY"), 0);
        assert_eq!(decimals("KWD"), 3);
        assert_eq!(decimals("XYZ"), 2);
        let eur = find("EUR").unwrap();
        assert!(matches(eur, "euro") && matches(eur, "€") && matches(eur, "eu") && matches(eur, ""));
        assert!(!matches(eur, "dollar"));
    }
}
