//! wix-guid - GUID generator for WiX installers
//!
//! Generates GUIDs in various formats for use in WiX installer projects.
//! Supports both random (v4) and deterministic (hash-based) generation.
//!
//! # Example
//!
//! ```
//! use wix_init::guid::{Guid, GuidFormat, GuidGenerator};
//!
//! // Random GUID
//! let guid = Guid::random();
//! println!("{}", guid.format(GuidFormat::Braces));
//!
//! // Deterministic GUID from path (for components)
//! let gen = GuidGenerator::new("MyProduct", "1.0.0");
//! let guid = gen.component_guid("INSTALLDIR/bin/app.exe");
//! println!("{}", guid.format(GuidFormat::Braces));
//! ```

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuidError {
    #[error("Invalid GUID format: {0}")]
    InvalidFormat(String),

    #[error("Invalid GUID string: {0}")]
    ParseError(String),
}

/// GUID format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuidFormat {
    /// With braces and hyphens: {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}
    #[default]
    Braces,
    /// With hyphens, no braces: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    Hyphens,
    /// No hyphens or braces: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    Plain,
    /// Registry format: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    Registry,
    /// C# format: new Guid("...")
    CSharp,
    /// WiX Component format: *
    WixAuto,
}

impl GuidFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            GuidFormat::Braces => "braces",
            GuidFormat::Hyphens => "hyphens",
            GuidFormat::Plain => "plain",
            GuidFormat::Registry => "registry",
            GuidFormat::CSharp => "csharp",
            GuidFormat::WixAuto => "auto",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "braces" | "b" => Some(GuidFormat::Braces),
            "hyphens" | "h" | "d" => Some(GuidFormat::Hyphens),
            "plain" | "p" | "n" => Some(GuidFormat::Plain),
            "registry" | "reg" | "r" => Some(GuidFormat::Registry),
            "csharp" | "cs" | "c" => Some(GuidFormat::CSharp),
            "auto" | "a" | "*" => Some(GuidFormat::WixAuto),
            _ => None,
        }
    }
}

/// A GUID value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guid {
    bytes: [u8; 16],
}

impl Guid {
    /// Create a new random GUID (version 4)
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);

        // Set version to 4 (random)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self { bytes }
    }

    /// Create a GUID from raw bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Create a deterministic GUID from a string using a simple hash
    pub fn from_hash(input: &str) -> Self {
        let hash = simple_hash(input);
        let mut bytes = [0u8; 16];

        // Use hash to fill bytes
        for (i, chunk) in hash.chunks(8).enumerate() {
            if i < 2 {
                for (j, &b) in chunk.iter().enumerate() {
                    if i * 8 + j < 16 {
                        bytes[i * 8 + j] = b;
                    }
                }
            }
        }

        // Set version to 5 (name-based SHA-1, close enough)
        bytes[6] = (bytes[6] & 0x0f) | 0x50;
        // Set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self { bytes }
    }

    /// Create a deterministic GUID with a namespace
    pub fn from_namespace_and_name(namespace: &Guid, name: &str) -> Self {
        let mut input = Vec::with_capacity(16 + name.len());
        input.extend_from_slice(&namespace.bytes);
        input.extend_from_slice(name.as_bytes());

        let hash = simple_hash(&String::from_utf8_lossy(&input));
        let mut bytes = [0u8; 16];

        for (i, &b) in hash.iter().take(16).enumerate() {
            bytes[i] = b;
        }

        // Set version to 5
        bytes[6] = (bytes[6] & 0x0f) | 0x50;
        // Set variant to RFC 4122
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self { bytes }
    }

    /// Parse a GUID from a string
    pub fn parse(s: &str) -> Result<Self, GuidError> {
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();

        if cleaned.len() != 32 {
            return Err(GuidError::ParseError(format!(
                "Expected 32 hex digits, got {}",
                cleaned.len()
            )));
        }

        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&cleaned[i * 2..i * 2 + 2], 16)
                .map_err(|_| GuidError::ParseError("Invalid hex digit".into()))?;
        }

        Ok(Self { bytes })
    }

    /// Format the GUID as a string
    pub fn format(&self, format: GuidFormat) -> String {
        match format {
            GuidFormat::Braces => self.format_braces(false),
            GuidFormat::Hyphens => self.format_hyphens(false),
            GuidFormat::Plain => self.format_plain(false),
            GuidFormat::Registry => self.format_braces(true),
            GuidFormat::CSharp => format!("new Guid(\"{}\")", self.format_hyphens(false)),
            GuidFormat::WixAuto => "*".to_string(),
        }
    }

    fn format_braces(&self, uppercase: bool) -> String {
        if uppercase {
            format!("{{{}}}", self.format_hyphens(true))
        } else {
            format!("{{{}}}", self.format_hyphens(false))
        }
    }

    fn format_hyphens(&self, uppercase: bool) -> String {
        let hex: String = self.bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let formatted = format!(
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        );
        if uppercase {
            formatted.to_uppercase()
        } else {
            formatted
        }
    }

    fn format_plain(&self, uppercase: bool) -> String {
        let hex: String = self.bytes.iter().map(|b| format!("{:02x}", b)).collect();
        if uppercase {
            hex.to_uppercase()
        } else {
            hex
        }
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Get the nil (all zeros) GUID
    pub fn nil() -> Self {
        Self { bytes: [0u8; 16] }
    }

    /// Check if this is the nil GUID
    pub fn is_nil(&self) -> bool {
        self.bytes.iter().all(|&b| b == 0)
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(GuidFormat::Braces))
    }
}

/// Simple hash function for deterministic GUID generation
fn simple_hash(input: &str) -> Vec<u8> {
    // FNV-1a inspired hash, produces consistent results
    let mut hash = vec![0u8; 16];
    let prime: u64 = 0x100000001b3;
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x84222325cbf29ce4;

    for byte in input.bytes() {
        h1 ^= byte as u64;
        h1 = h1.wrapping_mul(prime);
        h2 ^= byte as u64;
        h2 = h2.wrapping_mul(prime.wrapping_add(2));
    }

    // Additional mixing
    h1 ^= h1 >> 33;
    h1 = h1.wrapping_mul(0xff51afd7ed558ccd);
    h2 ^= h2 >> 33;
    h2 = h2.wrapping_mul(0xc4ceb9fe1a85ec53);

    for (i, b) in h1.to_le_bytes().iter().enumerate() {
        hash[i] = *b;
    }
    for (i, b) in h2.to_le_bytes().iter().enumerate() {
        hash[8 + i] = *b;
    }

    hash
}

/// GUID generator with namespace support
#[derive(Debug, Clone)]
pub struct GuidGenerator {
    namespace: Guid,
}

impl GuidGenerator {
    /// Create a new generator with a namespace derived from product info
    pub fn new(product_name: &str, version: &str) -> Self {
        let namespace_input = format!("{}:{}", product_name, version);
        Self {
            namespace: Guid::from_hash(&namespace_input),
        }
    }

    /// Create a generator with a specific namespace GUID
    pub fn with_namespace(namespace: Guid) -> Self {
        Self { namespace }
    }

    /// Generate a component GUID from a file path
    pub fn component_guid(&self, path: &str) -> Guid {
        Guid::from_namespace_and_name(&self.namespace, &format!("component:{}", path))
    }

    /// Generate a feature GUID from a feature name
    pub fn feature_guid(&self, name: &str) -> Guid {
        Guid::from_namespace_and_name(&self.namespace, &format!("feature:{}", name))
    }

    /// Generate a product code GUID
    pub fn product_code(&self) -> Guid {
        Guid::from_namespace_and_name(&self.namespace, "product")
    }

    /// Generate an upgrade code GUID
    pub fn upgrade_code(&self) -> Guid {
        Guid::from_namespace_and_name(&self.namespace, "upgrade")
    }

    /// Generate a custom GUID from any input
    pub fn custom_guid(&self, input: &str) -> Guid {
        Guid::from_namespace_and_name(&self.namespace, input)
    }

    /// Get the namespace GUID
    pub fn namespace(&self) -> &Guid {
        &self.namespace
    }
}

impl Default for GuidGenerator {
    fn default() -> Self {
        Self::new("WixCraft", "1.0.0")
    }
}

/// Well-known namespace GUIDs
pub mod namespaces {
    use super::Guid;

    /// DNS namespace (RFC 4122)
    pub fn dns() -> Guid {
        Guid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ])
    }

    /// URL namespace (RFC 4122)
    pub fn url() -> Guid {
        Guid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ])
    }

    /// OID namespace (RFC 4122)
    pub fn oid() -> Guid {
        Guid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x12, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ])
    }

    /// X500 namespace (RFC 4122)
    pub fn x500() -> Guid {
        Guid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ])
    }

    /// WiX components namespace (custom)
    pub fn wix_components() -> Guid {
        Guid::from_bytes([
            0x57, 0x69, 0x78, 0x43, 0x6f, 0x6d, 0x70, 0x6f,
            0x6e, 0x65, 0x6e, 0x74, 0x73, 0x00, 0x00, 0x00,
        ])
    }
}

/// Batch GUID generation
pub struct GuidBatch {
    guids: Vec<Guid>,
}

impl GuidBatch {
    /// Generate multiple random GUIDs
    pub fn random(count: usize) -> Self {
        let guids = (0..count).map(|_| Guid::random()).collect();
        Self { guids }
    }

    /// Generate deterministic GUIDs from a list of inputs
    pub fn from_inputs(generator: &GuidGenerator, inputs: &[&str]) -> Self {
        let guids = inputs
            .iter()
            .map(|input| generator.custom_guid(input))
            .collect();
        Self { guids }
    }

    /// Get the generated GUIDs
    pub fn guids(&self) -> &[Guid] {
        &self.guids
    }

    /// Format all GUIDs
    pub fn format_all(&self, format: GuidFormat) -> Vec<String> {
        self.guids.iter().map(|g| g.format(format)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_guid() {
        let guid1 = Guid::random();
        let guid2 = Guid::random();
        assert_ne!(guid1, guid2);
    }

    #[test]
    fn test_guid_format_braces() {
        let guid = Guid::from_bytes([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        ]);
        let formatted = guid.format(GuidFormat::Braces);
        assert!(formatted.starts_with('{'));
        assert!(formatted.ends_with('}'));
        assert!(formatted.contains('-'));
    }

    #[test]
    fn test_guid_format_hyphens() {
        let guid = Guid::random();
        let formatted = guid.format(GuidFormat::Hyphens);
        assert!(!formatted.starts_with('{'));
        assert!(formatted.contains('-'));
        assert_eq!(formatted.len(), 36);
    }

    #[test]
    fn test_guid_format_plain() {
        let guid = Guid::random();
        let formatted = guid.format(GuidFormat::Plain);
        assert!(!formatted.contains('-'));
        assert!(!formatted.contains('{'));
        assert_eq!(formatted.len(), 32);
    }

    #[test]
    fn test_guid_format_registry() {
        let guid = Guid::random();
        let formatted = guid.format(GuidFormat::Registry);
        assert!(formatted.starts_with('{'));
        assert!(formatted.ends_with('}'));
        assert_eq!(formatted, formatted.to_uppercase());
    }

    #[test]
    fn test_guid_format_csharp() {
        let guid = Guid::random();
        let formatted = guid.format(GuidFormat::CSharp);
        assert!(formatted.starts_with("new Guid(\""));
        assert!(formatted.ends_with("\")"));
    }

    #[test]
    fn test_guid_format_auto() {
        let guid = Guid::random();
        let formatted = guid.format(GuidFormat::WixAuto);
        assert_eq!(formatted, "*");
    }

    #[test]
    fn test_deterministic_guid() {
        let guid1 = Guid::from_hash("test input");
        let guid2 = Guid::from_hash("test input");
        assert_eq!(guid1, guid2);

        let guid3 = Guid::from_hash("different input");
        assert_ne!(guid1, guid3);
    }

    #[test]
    fn test_guid_parse() {
        let original = Guid::random();
        let formatted = original.format(GuidFormat::Braces);
        let parsed = Guid::parse(&formatted).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_guid_parse_formats() {
        let guid = Guid::random();

        // Parse from braces format
        let braces = guid.format(GuidFormat::Braces);
        let parsed1 = Guid::parse(&braces).unwrap();
        assert_eq!(guid, parsed1);

        // Parse from hyphens format
        let hyphens = guid.format(GuidFormat::Hyphens);
        let parsed2 = Guid::parse(&hyphens).unwrap();
        assert_eq!(guid, parsed2);

        // Parse from plain format
        let plain = guid.format(GuidFormat::Plain);
        let parsed3 = Guid::parse(&plain).unwrap();
        assert_eq!(guid, parsed3);
    }

    #[test]
    fn test_nil_guid() {
        let nil = Guid::nil();
        assert!(nil.is_nil());

        let random = Guid::random();
        assert!(!random.is_nil());
    }

    #[test]
    fn test_generator_component_guid() {
        let gen = GuidGenerator::new("MyProduct", "1.0.0");

        let guid1 = gen.component_guid("bin/app.exe");
        let guid2 = gen.component_guid("bin/app.exe");
        assert_eq!(guid1, guid2);

        let guid3 = gen.component_guid("bin/other.exe");
        assert_ne!(guid1, guid3);
    }

    #[test]
    fn test_generator_different_products() {
        let gen1 = GuidGenerator::new("Product1", "1.0.0");
        let gen2 = GuidGenerator::new("Product2", "1.0.0");

        let guid1 = gen1.component_guid("file.txt");
        let guid2 = gen2.component_guid("file.txt");
        assert_ne!(guid1, guid2);
    }

    #[test]
    fn test_generator_different_versions() {
        let gen1 = GuidGenerator::new("Product", "1.0.0");
        let gen2 = GuidGenerator::new("Product", "2.0.0");

        let guid1 = gen1.product_code();
        let guid2 = gen2.product_code();
        assert_ne!(guid1, guid2);
    }

    #[test]
    fn test_generator_upgrade_code() {
        let gen = GuidGenerator::new("Product", "1.0.0");
        let upgrade = gen.upgrade_code();
        assert!(!upgrade.is_nil());
    }

    #[test]
    fn test_generator_feature_guid() {
        let gen = GuidGenerator::new("Product", "1.0.0");
        let guid = gen.feature_guid("MainFeature");
        assert!(!guid.is_nil());
    }

    #[test]
    fn test_guid_batch_random() {
        let batch = GuidBatch::random(5);
        assert_eq!(batch.guids().len(), 5);

        // All should be unique
        let formatted: std::collections::HashSet<_> = batch
            .format_all(GuidFormat::Plain)
            .into_iter()
            .collect();
        assert_eq!(formatted.len(), 5);
    }

    #[test]
    fn test_guid_batch_from_inputs() {
        let gen = GuidGenerator::new("Test", "1.0.0");
        let inputs = vec!["file1.txt", "file2.txt", "file3.txt"];
        let batch = GuidBatch::from_inputs(&gen, &inputs);

        assert_eq!(batch.guids().len(), 3);
    }

    #[test]
    fn test_guid_format_from_str() {
        assert_eq!(GuidFormat::from_str("braces"), Some(GuidFormat::Braces));
        assert_eq!(GuidFormat::from_str("hyphens"), Some(GuidFormat::Hyphens));
        assert_eq!(GuidFormat::from_str("plain"), Some(GuidFormat::Plain));
        assert_eq!(GuidFormat::from_str("registry"), Some(GuidFormat::Registry));
        assert_eq!(GuidFormat::from_str("csharp"), Some(GuidFormat::CSharp));
        assert_eq!(GuidFormat::from_str("auto"), Some(GuidFormat::WixAuto));
        assert_eq!(GuidFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_namespaces() {
        let dns = namespaces::dns();
        let url = namespaces::url();
        assert_ne!(dns, url);
        assert!(!dns.is_nil());
        assert!(!url.is_nil());
    }

    #[test]
    fn test_namespace_guid_generation() {
        let ns = namespaces::wix_components();
        let guid = Guid::from_namespace_and_name(&ns, "test");
        assert!(!guid.is_nil());
    }

    #[test]
    fn test_guid_display() {
        let guid = Guid::random();
        let display = format!("{}", guid);
        assert!(display.starts_with('{'));
        assert!(display.ends_with('}'));
    }

    #[test]
    fn test_parse_error() {
        let result = Guid::parse("invalid");
        assert!(result.is_err());

        let result = Guid::parse("not-a-guid");
        assert!(result.is_err());
    }

    #[test]
    fn test_guid_bytes() {
        let guid = Guid::random();
        let bytes = guid.as_bytes();
        assert_eq!(bytes.len(), 16);
    }
}
