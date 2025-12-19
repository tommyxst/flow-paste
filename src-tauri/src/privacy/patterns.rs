use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PIIType {
    Phone,
    Email,
    IDCard,
    BankCard,
    IP,
    APIKey,
}

impl PIIType {
    pub fn placeholder_prefix(&self) -> &'static str {
        match self {
            PIIType::Phone => "PHONE",
            PIIType::Email => "EMAIL",
            PIIType::IDCard => "IDCARD",
            PIIType::BankCard => "BANKCARD",
            PIIType::IP => "IP",
            PIIType::APIKey => "APIKEY",
        }
    }
}

pub struct PIIPattern {
    pub pii_type: PIIType,
    pub regex: &'static Lazy<Regex>,
    pub priority: u8,
}

// CN Mobile: 1[3-9]\d{9}
static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b1[3-9]\d{9}\b").unwrap()
});

// Email: standard format
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap()
});

// CN ID Card: 18 digits (last can be X)
static IDCARD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]\b").unwrap()
});

// Bank Card: 13-19 digits
static BANKCARD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[3-6]\d{12,18}\b").unwrap()
});

// IPv4 Address
static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b").unwrap()
});

// API Keys: sk-..., pk-..., api-..., key-... patterns (stricter length/charset)
static APIKEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:sk|pk|api|key)-[A-Za-z0-9_-]{32,64}\b").unwrap()
});

pub static PII_PATTERNS: Lazy<Vec<PIIPattern>> = Lazy::new(|| {
    vec![
        // Higher priority = matched first when overlapping
        PIIPattern { pii_type: PIIType::IDCard, regex: &IDCARD_REGEX, priority: 100 },
        PIIPattern { pii_type: PIIType::APIKey, regex: &APIKEY_REGEX, priority: 90 },
        PIIPattern { pii_type: PIIType::Email, regex: &EMAIL_REGEX, priority: 80 },
        PIIPattern { pii_type: PIIType::BankCard, regex: &BANKCARD_REGEX, priority: 70 },
        PIIPattern { pii_type: PIIType::Phone, regex: &PHONE_REGEX, priority: 60 },
        PIIPattern { pii_type: PIIType::IP, regex: &IP_REGEX, priority: 50 },
    ]
});

pub fn luhn_check(card_number: &str) -> bool {
    let digits: Vec<u32> = card_number
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for &digit in digits.iter().rev() {
        let mut d = digit;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_pattern() {
        assert!(PHONE_REGEX.is_match("13800138000"));
        assert!(PHONE_REGEX.is_match("19912345678"));
        assert!(!PHONE_REGEX.is_match("12345678901"));
        assert!(!PHONE_REGEX.is_match("1380013800")); // 10 digits
    }

    #[test]
    fn test_email_pattern() {
        assert!(EMAIL_REGEX.is_match("test@example.com"));
        assert!(EMAIL_REGEX.is_match("user.name+tag@domain.co.uk"));
        assert!(!EMAIL_REGEX.is_match("invalid@"));
    }

    #[test]
    fn test_idcard_pattern() {
        assert!(IDCARD_REGEX.is_match("110101199003074518"));
        assert!(IDCARD_REGEX.is_match("11010119900307451X"));
        assert!(!IDCARD_REGEX.is_match("12345678901234567")); // Invalid format
    }

    #[test]
    fn test_ip_pattern() {
        assert!(IP_REGEX.is_match("192.168.1.1"));
        assert!(IP_REGEX.is_match("10.0.0.1"));
        assert!(!IP_REGEX.is_match("256.1.1.1"));
    }

    #[test]
    fn test_apikey_pattern() {
        assert!(APIKEY_REGEX.is_match("sk-abcdefghijklmnopqrstuvwx"));
        assert!(APIKEY_REGEX.is_match("api-1234567890abcdefghij"));
        assert!(!APIKEY_REGEX.is_match("sk-short"));
    }

    #[test]
    fn test_luhn_check() {
        assert!(luhn_check("4532015112830366")); // Valid test card
        assert!(!luhn_check("1234567890123456")); // Invalid
    }
}
