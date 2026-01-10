//! wix-license - License file generator for WiX installers
//!
//! Generates RTF license files and embedded license components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common open source licenses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    MIT,
    Apache2,
    GPL2,
    GPL3,
    LGPL21,
    LGPL3,
    BSD2Clause,
    BSD3Clause,
    ISC,
    MPL2,
    Unlicense,
    Proprietary,
    Custom,
}

impl LicenseType {
    pub fn spdx_id(&self) -> &'static str {
        match self {
            LicenseType::MIT => "MIT",
            LicenseType::Apache2 => "Apache-2.0",
            LicenseType::GPL2 => "GPL-2.0",
            LicenseType::GPL3 => "GPL-3.0",
            LicenseType::LGPL21 => "LGPL-2.1",
            LicenseType::LGPL3 => "LGPL-3.0",
            LicenseType::BSD2Clause => "BSD-2-Clause",
            LicenseType::BSD3Clause => "BSD-3-Clause",
            LicenseType::ISC => "ISC",
            LicenseType::MPL2 => "MPL-2.0",
            LicenseType::Unlicense => "Unlicense",
            LicenseType::Proprietary => "Proprietary",
            LicenseType::Custom => "Custom",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LicenseType::MIT => "MIT License",
            LicenseType::Apache2 => "Apache License 2.0",
            LicenseType::GPL2 => "GNU General Public License v2.0",
            LicenseType::GPL3 => "GNU General Public License v3.0",
            LicenseType::LGPL21 => "GNU Lesser General Public License v2.1",
            LicenseType::LGPL3 => "GNU Lesser General Public License v3.0",
            LicenseType::BSD2Clause => "BSD 2-Clause License",
            LicenseType::BSD3Clause => "BSD 3-Clause License",
            LicenseType::ISC => "ISC License",
            LicenseType::MPL2 => "Mozilla Public License 2.0",
            LicenseType::Unlicense => "The Unlicense",
            LicenseType::Proprietary => "Proprietary License",
            LicenseType::Custom => "Custom License",
        }
    }

    pub fn from_spdx(spdx: &str) -> Option<Self> {
        match spdx.to_uppercase().as_str() {
            "MIT" => Some(LicenseType::MIT),
            "APACHE-2.0" => Some(LicenseType::Apache2),
            "GPL-2.0" | "GPL-2.0-ONLY" => Some(LicenseType::GPL2),
            "GPL-3.0" | "GPL-3.0-ONLY" => Some(LicenseType::GPL3),
            "LGPL-2.1" | "LGPL-2.1-ONLY" => Some(LicenseType::LGPL21),
            "LGPL-3.0" | "LGPL-3.0-ONLY" => Some(LicenseType::LGPL3),
            "BSD-2-CLAUSE" => Some(LicenseType::BSD2Clause),
            "BSD-3-CLAUSE" => Some(LicenseType::BSD3Clause),
            "ISC" => Some(LicenseType::ISC),
            "MPL-2.0" => Some(LicenseType::MPL2),
            "UNLICENSE" => Some(LicenseType::Unlicense),
            _ => None,
        }
    }
}

/// License configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub license_type: LicenseType,
    pub copyright_holder: String,
    pub copyright_year: String,
    pub product_name: Option<String>,
    pub additional_terms: Option<String>,
    pub custom_text: Option<String>,
}

impl LicenseConfig {
    pub fn new(license_type: LicenseType, holder: &str, year: &str) -> Self {
        Self {
            license_type,
            copyright_holder: holder.to_string(),
            copyright_year: year.to_string(),
            product_name: None,
            additional_terms: None,
            custom_text: None,
        }
    }

    pub fn mit(holder: &str, year: &str) -> Self {
        Self::new(LicenseType::MIT, holder, year)
    }

    pub fn apache2(holder: &str, year: &str) -> Self {
        Self::new(LicenseType::Apache2, holder, year)
    }

    pub fn proprietary(holder: &str, year: &str) -> Self {
        Self::new(LicenseType::Proprietary, holder, year)
    }

    pub fn with_product(mut self, product: &str) -> Self {
        self.product_name = Some(product.to_string());
        self
    }

    pub fn with_additional_terms(mut self, terms: &str) -> Self {
        self.additional_terms = Some(terms.to_string());
        self
    }
}

/// RTF formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtfOptions {
    pub font_name: String,
    pub font_size: u32,
    pub title_size: u32,
    pub line_spacing: u32,
    pub margins: RtfMargins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtfMargins {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Default for RtfOptions {
    fn default() -> Self {
        Self {
            font_name: "Arial".to_string(),
            font_size: 10,
            title_size: 14,
            line_spacing: 240,
            margins: RtfMargins {
                left: 1440,
                right: 1440,
                top: 1440,
                bottom: 1440,
            },
        }
    }
}

/// License text generator
pub struct LicenseGenerator;

impl LicenseGenerator {
    /// Generate plain text license
    pub fn generate_text(config: &LicenseConfig) -> String {
        match config.license_type {
            LicenseType::MIT => Self::mit_text(config),
            LicenseType::Apache2 => Self::apache2_text(config),
            LicenseType::BSD2Clause => Self::bsd2_text(config),
            LicenseType::BSD3Clause => Self::bsd3_text(config),
            LicenseType::ISC => Self::isc_text(config),
            LicenseType::Proprietary => Self::proprietary_text(config),
            LicenseType::Custom => config.custom_text.clone().unwrap_or_default(),
            _ => Self::generic_text(config),
        }
    }

    fn mit_text(config: &LicenseConfig) -> String {
        format!(
            r#"MIT License

Copyright (c) {} {}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE."#,
            config.copyright_year, config.copyright_holder
        )
    }

    fn apache2_text(config: &LicenseConfig) -> String {
        format!(
            r#"Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

Copyright {} {}

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License."#,
            config.copyright_year, config.copyright_holder
        )
    }

    fn bsd2_text(config: &LicenseConfig) -> String {
        format!(
            r#"BSD 2-Clause License

Copyright (c) {}, {}
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."#,
            config.copyright_year, config.copyright_holder
        )
    }

    fn bsd3_text(config: &LicenseConfig) -> String {
        format!(
            r#"BSD 3-Clause License

Copyright (c) {}, {}
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."#,
            config.copyright_year, config.copyright_holder
        )
    }

    fn isc_text(config: &LicenseConfig) -> String {
        format!(
            r#"ISC License

Copyright (c) {} {}

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE."#,
            config.copyright_year, config.copyright_holder
        )
    }

    fn proprietary_text(config: &LicenseConfig) -> String {
        let product = config.product_name.as_deref().unwrap_or("this software");
        format!(
            r#"PROPRIETARY SOFTWARE LICENSE AGREEMENT

Copyright (c) {} {}. All Rights Reserved.

This software and associated documentation files ("{}") are proprietary
and confidential. Unauthorized copying, modification, distribution, or use
of this software, via any medium, is strictly prohibited.

This software is provided "AS IS" without warranty of any kind, express or
implied. In no event shall {} be liable for any damages arising from
the use of this software.

By installing or using this software, you agree to be bound by the terms
of this license agreement.

{}

For licensing inquiries, contact: {}"#,
            config.copyright_year,
            config.copyright_holder,
            product,
            config.copyright_holder,
            config.additional_terms.as_deref().unwrap_or(""),
            config.copyright_holder
        )
    }

    fn generic_text(config: &LicenseConfig) -> String {
        format!(
            r#"{}

Copyright (c) {} {}

This software is licensed under the {} license.
Please refer to the full license text for terms and conditions."#,
            config.license_type.name(),
            config.copyright_year,
            config.copyright_holder,
            config.license_type.name()
        )
    }

    /// Generate RTF license for WiX installer
    pub fn generate_rtf(config: &LicenseConfig, options: &RtfOptions) -> String {
        let text = Self::generate_text(config);
        Self::text_to_rtf(&text, options)
    }

    /// Convert plain text to RTF format
    pub fn text_to_rtf(text: &str, options: &RtfOptions) -> String {
        let mut rtf = String::new();

        // RTF header
        rtf.push_str("{\\rtf1\\ansi\\deff0\n");

        // Font table
        rtf.push_str(&format!("{{\\fonttbl{{\\f0 {};}}}}\\n", options.font_name));

        // Document formatting
        rtf.push_str(&format!(
            "\\margl{}\\margr{}\\margt{}\\margb{}\n",
            options.margins.left,
            options.margins.right,
            options.margins.top,
            options.margins.bottom
        ));

        // Font size (in half-points)
        rtf.push_str(&format!("\\fs{}\n", options.font_size * 2));

        // Line spacing
        rtf.push_str(&format!("\\sl{}\n", options.line_spacing));

        // Content with escaped special characters
        for line in text.lines() {
            let escaped = Self::escape_rtf(line);
            rtf.push_str(&escaped);
            rtf.push_str("\\par\n");
        }

        rtf.push_str("}\n");
        rtf
    }

    fn escape_rtf(text: &str) -> String {
        let mut result = String::new();
        for c in text.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '{' => result.push_str("\\{"),
                '}' => result.push_str("\\}"),
                '\n' => result.push_str("\\par "),
                '\t' => result.push_str("\\tab "),
                c if c.is_ascii() => result.push(c),
                c => {
                    // Unicode escape
                    result.push_str(&format!("\\u{}?", c as i32));
                }
            }
        }
        result
    }
}

/// WiX license component generator
pub struct WixLicenseComponent;

impl WixLicenseComponent {
    /// Generate WiX XML for embedding license
    pub fn generate_embedded(rtf_path: &str) -> String {
        format!(
            r#"<WixVariable Id="WixUILicenseRtf" Value="{}" />"#,
            rtf_path
        )
    }

    /// Generate component for installing license file
    pub fn generate_file_component(source_path: &str, component_id: &str) -> String {
        format!(
            r#"<Component Id="{}" Guid="*">
    <File Id="LicenseFile" Source="{}" Name="LICENSE.txt" />
</Component>"#,
            component_id, source_path
        )
    }
}

/// License detector from project files
pub struct LicenseDetector;

impl LicenseDetector {
    /// Detect license from various project files
    pub fn detect_from_files(files: &HashMap<String, String>) -> Option<LicenseType> {
        // Check package.json
        if let Some(content) = files.get("package.json") {
            if let Some(license) = Self::from_package_json(content) {
                return Some(license);
            }
        }

        // Check Cargo.toml
        if let Some(content) = files.get("Cargo.toml") {
            if let Some(license) = Self::from_cargo_toml(content) {
                return Some(license);
            }
        }

        // Check LICENSE file content
        if let Some(content) = files.get("LICENSE") {
            return Self::from_license_text(content);
        }

        None
    }

    fn from_package_json(content: &str) -> Option<LicenseType> {
        // Simple extraction - look for "license": "..."
        if let Some(start) = content.find("\"license\"") {
            let rest = &content[start..];
            if let Some(colon) = rest.find(':') {
                let after_colon = &rest[colon + 1..];
                let trimmed = after_colon.trim();
                if trimmed.starts_with('"') {
                    let end = trimmed[1..].find('"')?;
                    let license = &trimmed[1..=end];
                    return LicenseType::from_spdx(license);
                }
            }
        }
        None
    }

    fn from_cargo_toml(content: &str) -> Option<LicenseType> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("license") && line.contains('=') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let license = parts[1].trim().trim_matches('"');
                    return LicenseType::from_spdx(license);
                }
            }
        }
        None
    }

    fn from_license_text(content: &str) -> Option<LicenseType> {
        let lower = content.to_lowercase();

        if lower.contains("mit license") || lower.contains("permission is hereby granted, free of charge") {
            return Some(LicenseType::MIT);
        }
        if lower.contains("apache license") && lower.contains("version 2.0") {
            return Some(LicenseType::Apache2);
        }
        if lower.contains("gnu general public license") {
            if lower.contains("version 3") {
                return Some(LicenseType::GPL3);
            }
            if lower.contains("version 2") {
                return Some(LicenseType::GPL2);
            }
        }
        if lower.contains("bsd 2-clause") {
            return Some(LicenseType::BSD2Clause);
        }
        if lower.contains("bsd 3-clause") {
            return Some(LicenseType::BSD3Clause);
        }
        if lower.contains("isc license") {
            return Some(LicenseType::ISC);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_type_spdx() {
        assert_eq!(LicenseType::MIT.spdx_id(), "MIT");
        assert_eq!(LicenseType::Apache2.spdx_id(), "Apache-2.0");
    }

    #[test]
    fn test_license_type_name() {
        assert_eq!(LicenseType::MIT.name(), "MIT License");
        assert_eq!(LicenseType::GPL3.name(), "GNU General Public License v3.0");
    }

    #[test]
    fn test_license_type_from_spdx() {
        assert_eq!(LicenseType::from_spdx("MIT"), Some(LicenseType::MIT));
        assert_eq!(LicenseType::from_spdx("Apache-2.0"), Some(LicenseType::Apache2));
        assert_eq!(LicenseType::from_spdx("invalid"), None);
    }

    #[test]
    fn test_license_config_new() {
        let config = LicenseConfig::new(LicenseType::MIT, "Test Corp", "2024");
        assert_eq!(config.license_type, LicenseType::MIT);
        assert_eq!(config.copyright_holder, "Test Corp");
    }

    #[test]
    fn test_license_config_mit() {
        let config = LicenseConfig::mit("Test Corp", "2024");
        assert_eq!(config.license_type, LicenseType::MIT);
    }

    #[test]
    fn test_license_config_with_product() {
        let config = LicenseConfig::mit("Test", "2024").with_product("MyApp");
        assert_eq!(config.product_name, Some("MyApp".to_string()));
    }

    #[test]
    fn test_generate_mit_text() {
        let config = LicenseConfig::mit("Test Corp", "2024");
        let text = LicenseGenerator::generate_text(&config);
        assert!(text.contains("MIT License"));
        assert!(text.contains("Test Corp"));
        assert!(text.contains("2024"));
    }

    #[test]
    fn test_generate_apache2_text() {
        let config = LicenseConfig::apache2("Test Corp", "2024");
        let text = LicenseGenerator::generate_text(&config);
        assert!(text.contains("Apache License"));
        assert!(text.contains("Version 2.0"));
    }

    #[test]
    fn test_generate_proprietary_text() {
        let config = LicenseConfig::proprietary("Test Corp", "2024");
        let text = LicenseGenerator::generate_text(&config);
        assert!(text.contains("PROPRIETARY"));
        assert!(text.contains("Test Corp"));
    }

    #[test]
    fn test_rtf_options_default() {
        let options = RtfOptions::default();
        assert_eq!(options.font_name, "Arial");
        assert_eq!(options.font_size, 10);
    }

    #[test]
    fn test_generate_rtf() {
        let config = LicenseConfig::mit("Test Corp", "2024");
        let options = RtfOptions::default();
        let rtf = LicenseGenerator::generate_rtf(&config, &options);
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("MIT License"));
    }

    #[test]
    fn test_escape_rtf() {
        let escaped = LicenseGenerator::escape_rtf("Test {with} \\special\\ chars");
        assert!(escaped.contains("\\{"));
        assert!(escaped.contains("\\}"));
        assert!(escaped.contains("\\\\"));
    }

    #[test]
    fn test_wix_embedded_component() {
        let xml = WixLicenseComponent::generate_embedded("License.rtf");
        assert!(xml.contains("WixUILicenseRtf"));
        assert!(xml.contains("License.rtf"));
    }

    #[test]
    fn test_wix_file_component() {
        let xml = WixLicenseComponent::generate_file_component("License.txt", "LicenseComponent");
        assert!(xml.contains("LicenseComponent"));
        assert!(xml.contains("LICENSE.txt"));
    }

    #[test]
    fn test_detect_from_package_json() {
        let mut files = HashMap::new();
        files.insert("package.json".to_string(), r#"{"license": "MIT"}"#.to_string());
        assert_eq!(LicenseDetector::detect_from_files(&files), Some(LicenseType::MIT));
    }

    #[test]
    fn test_detect_from_cargo_toml() {
        let mut files = HashMap::new();
        files.insert("Cargo.toml".to_string(), "license = \"Apache-2.0\"".to_string());
        assert_eq!(LicenseDetector::detect_from_files(&files), Some(LicenseType::Apache2));
    }

    #[test]
    fn test_detect_from_license_text() {
        let mut files = HashMap::new();
        files.insert("LICENSE".to_string(), "MIT License\n\nPermission is hereby granted...".to_string());
        assert_eq!(LicenseDetector::detect_from_files(&files), Some(LicenseType::MIT));
    }
}
