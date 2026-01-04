//! Condition expression evaluator for lint rules
//!
//! Evaluates JavaScript-like condition expressions from rule definitions.
//! Supports a subset of expressions commonly used in lint rules.
//!
//! # Supported Expressions
//!
//! ## Attribute Access
//! - `attributes.Name` - Check if attribute exists and is truthy
//! - `!attributes.Name` - Check if attribute is missing
//! - `attributes.Name.startsWith('prefix')` - String prefix check
//! - `attributes.Name.endsWith('suffix')` - String suffix check
//! - `attributes.Name.toUpperCase()` - Convert to uppercase (for comparisons)
//!
//! ## Parent Access
//! - `parent.name` - Access parent element name
//! - `parent.countChildren('Element')` - Count children of parent
//!
//! ## Functions
//! - `countChildren()` - Count all direct children
//! - `countChildren('Element')` - Count children of specific type
//! - `hasChild('Element')` - Check if element has child of type
//! - `isValidGuid(attributes.Guid)` - Validate GUID format
//! - `isStandardDirectory(attributes.Id)` - Check against standard Windows directories
//! - `isSensitivePropertyName(attributes.Id)` - Check for password/secret patterns
//! - `getDepth()` - Get element nesting depth
//!
//! ## Comparisons
//! - `expr === value` - Strict equality
//! - `expr !== value` - Strict inequality
//! - `expr > value` - Greater than (numeric)
//!
//! ## Logical Operators
//! - `!expr` - Negation
//! - `expr1 && expr2` - Logical AND
//! - `expr1 || expr2` - Logical OR
//! - `(expr)` - Grouping with parentheses
//!
//! ## Regex
//! - `/pattern/.test(attributes.Value)` - Regex test
//!
//! # Limitations
//!
//! - No arithmetic operators (`+`, `-`, `*`, `/`)
//! - No ternary operator (`? :`)
//! - No array methods (`.map()`, `.filter()`, etc.)
//! - No variable declarations or assignments
//! - No `<`, `<=`, `>=` comparisons (only `>` is supported)
//! - Regex patterns are compiled on every evaluation (no caching)
//! - `&&` and `||` operators must have spaces around them
//! - Method chaining beyond single method calls is not supported

use super::helpers::Helpers;
use crate::parser::{WixDocument, WixElement};

/// Evaluates rule conditions against WiX elements
pub struct ConditionEvaluator<'a> {
    doc: &'a WixDocument,
    element: &'a WixElement,
    element_idx: usize,
}

impl<'a> ConditionEvaluator<'a> {
    /// Create a new evaluator for an element
    pub fn new(doc: &'a WixDocument, element: &'a WixElement, element_idx: usize) -> Self {
        Self {
            doc,
            element,
            element_idx,
        }
    }

    /// Evaluate a condition expression
    ///
    /// Returns true if the condition matches (i.e., the rule should trigger)
    pub fn evaluate(&self, condition: &str) -> bool {
        // Handle compound conditions with &&
        if condition.contains(" && ") {
            return condition
                .split(" && ")
                .all(|part| self.evaluate_single(part.trim()));
        }

        // Handle compound conditions with ||
        if condition.contains(" || ") {
            return condition
                .split(" || ")
                .any(|part| self.evaluate_single(part.trim()));
        }

        self.evaluate_single(condition)
    }

    /// Evaluate a single condition (no compound operators)
    fn evaluate_single(&self, condition: &str) -> bool {
        let condition = condition.trim();

        // Handle negation: !expression
        if let Some(inner) = condition.strip_prefix('!') {
            return !self.evaluate_single(inner.trim());
        }

        // Handle parentheses
        if condition.starts_with('(') && condition.ends_with(')') {
            return self.evaluate(&condition[1..condition.len() - 1]);
        }

        // Handle comparisons FIRST (before function calls, since comparisons may contain function calls)
        if condition.contains(" === ") {
            let parts: Vec<&str> = condition.split(" === ").collect();
            if parts.len() == 2 {
                let left = self.eval_value(parts[0].trim());
                let right = self.eval_value(parts[1].trim());
                return left == right;
            }
        }

        if condition.contains(" !== ") {
            let parts: Vec<&str> = condition.split(" !== ").collect();
            if parts.len() == 2 {
                let left = self.eval_value(parts[0].trim());
                let right = self.eval_value(parts[1].trim());
                return left != right;
            }
        }

        if condition.contains(" > ") {
            let parts: Vec<&str> = condition.split(" > ").collect();
            if parts.len() == 2 {
                let left = self.eval_numeric(parts[0].trim());
                let right = self.eval_numeric(parts[1].trim());
                return left > right;
            }
        }

        // Handle attribute checks: attributes.X
        if condition.starts_with("attributes.") {
            return self.eval_attribute_expr(condition);
        }

        // Handle parent attribute checks: parent.X
        if condition.starts_with("parent.") {
            return self.eval_parent_expr(condition);
        }

        // Handle function calls (standalone, not in comparisons)
        if condition.contains('(') {
            return self.eval_function_call(condition);
        }

        // Default: treat as truthy check
        !self.eval_value(condition).is_empty()
    }

    /// Evaluate an attribute expression
    fn eval_attribute_expr(&self, expr: &str) -> bool {
        // attributes.X - check if attribute exists and is truthy
        if let Some(attr_name) = expr.strip_prefix("attributes.") {
            // Handle method calls on attributes
            if attr_name.contains('.') {
                let parts: Vec<&str> = attr_name.splitn(2, '.').collect();
                let attr = parts[0];
                let method = parts[1];
                return self.eval_attribute_method(attr, method);
            }

            // Simple attribute existence/truthy check
            return self.element.has_attr(attr_name);
        }

        false
    }

    /// Evaluate a method call on an attribute value
    fn eval_attribute_method(&self, attr: &str, method: &str) -> bool {
        let value = self.element.attr(attr).unwrap_or("");

        // Handle startsWith('x')
        if let Some(rest) = method.strip_prefix("startsWith(") {
            if let Some(arg) = rest.strip_suffix(')') {
                let arg = arg.trim_matches(|c| c == '\'' || c == '"');
                return value.starts_with(arg);
            }
        }

        // Handle endsWith('x')
        if let Some(rest) = method.strip_prefix("endsWith(") {
            if let Some(arg) = rest.strip_suffix(')') {
                let arg = arg.trim_matches(|c| c == '\'' || c == '"');
                return value.ends_with(arg);
            }
        }

        // Handle toUpperCase() comparison (usually in !== comparison)
        if method.contains("toUpperCase()") {
            // This is typically used in: attributes.Id !== attributes.Id.toUpperCase()
            return value == value.to_uppercase();
        }

        false
    }

    /// Evaluate a parent expression
    fn eval_parent_expr(&self, expr: &str) -> bool {
        if let Some(parent) = self.element.parent.and_then(|idx| self.doc.get(idx)) {
            if let Some(rest) = expr.strip_prefix("parent.") {
                // parent.countChildren('File') === 1
                if rest.starts_with("countChildren(") {
                    if let Some(inner) = rest.strip_prefix("countChildren(") {
                        if let Some(arg) = inner.strip_suffix(')') {
                            let name = arg.trim_matches(|c| c == '\'' || c == '"');
                            let count = self.doc.count_children(parent, Some(name));
                            // Check if there's a comparison
                            return count > 0; // Default: check if any exist
                        }
                    }
                }

                // parent.name
                if rest == "name" {
                    return !parent.name.is_empty();
                }
            }
        }

        false
    }

    /// Evaluate a function call
    fn eval_function_call(&self, expr: &str) -> bool {
        // countChildren('X') or countChildren()
        if expr.starts_with("countChildren(") {
            if let Some(inner) = expr
                .strip_prefix("countChildren(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let name = if inner.is_empty() {
                    None
                } else {
                    Some(inner.trim_matches(|c| c == '\'' || c == '"'))
                };
                let count = self.doc.count_children(self.element, name);
                // If used alone (not in comparison), check if > 0
                return count > 0;
            }
        }

        // hasChild('X')
        if expr.starts_with("hasChild(") {
            if let Some(inner) = expr
                .strip_prefix("hasChild(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let name = inner.trim_matches(|c| c == '\'' || c == '"');
                return self.doc.has_child(self.element, name);
            }
        }

        // isValidGuid(attributes.X)
        if expr.starts_with("isValidGuid(") {
            if let Some(inner) = expr
                .strip_prefix("isValidGuid(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let value = self.eval_value(inner);
                return Helpers::is_valid_guid(&value);
            }
        }

        // isStandardDirectory(attributes.X)
        if expr.starts_with("isStandardDirectory(") {
            if let Some(inner) = expr
                .strip_prefix("isStandardDirectory(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let value = self.eval_value(inner);
                return Helpers::is_standard_directory(&value);
            }
        }

        // isStandardDirectoryId(attributes.X)
        if expr.starts_with("isStandardDirectoryId(") {
            if let Some(inner) = expr
                .strip_prefix("isStandardDirectoryId(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let value = self.eval_value(inner);
                return Helpers::is_standard_directory(&value);
            }
        }

        // isSensitivePropertyName(attributes.X)
        if expr.starts_with("isSensitivePropertyName(") {
            if let Some(inner) = expr
                .strip_prefix("isSensitivePropertyName(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let value = self.eval_value(inner);
                return Helpers::is_sensitive_property_name(&value);
            }
        }

        // getDepth()
        if expr == "getDepth()" {
            return self.get_depth() > 0;
        }

        // Regex test: /pattern/.test(value)
        if expr.contains(".test(") {
            return self.eval_regex_test(expr);
        }

        false
    }

    /// Evaluate a regex test expression
    fn eval_regex_test(&self, expr: &str) -> bool {
        // Pattern: /regex/.test(attributes.X)
        if let Some(start) = expr.find('/') {
            if let Some(end) = expr[start + 1..].find('/') {
                let pattern = &expr[start + 1..start + 1 + end];
                if let Some(test_start) = expr.find(".test(") {
                    if let Some(arg) = expr[test_start + 6..].strip_suffix(')') {
                        let value = self.eval_value(arg);
                        if let Ok(re) = regex::Regex::new(pattern) {
                            return re.is_match(&value);
                        }
                    }
                }
            }
        }
        false
    }

    /// Evaluate a value expression to a string
    fn eval_value(&self, expr: &str) -> String {
        let expr = expr.trim();

        // String literal
        if (expr.starts_with('\'') && expr.ends_with('\''))
            || (expr.starts_with('"') && expr.ends_with('"'))
        {
            return expr[1..expr.len() - 1].to_string();
        }

        // Attribute access
        if let Some(attr) = expr.strip_prefix("attributes.") {
            // Handle method chains like attributes.Id.toUpperCase()
            let attr_name = attr.split('.').next().unwrap_or(attr);
            return self.element.attr(attr_name).unwrap_or("").to_string();
        }

        // Numeric literal
        if expr.parse::<i64>().is_ok() {
            return expr.to_string();
        }

        expr.to_string()
    }

    /// Evaluate a numeric expression
    fn eval_numeric(&self, expr: &str) -> i64 {
        let expr = expr.trim();

        // Function call returning number
        if expr.starts_with("countChildren(") {
            if let Some(inner) = expr
                .strip_prefix("countChildren(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let name = if inner.is_empty() {
                    None
                } else {
                    Some(inner.trim_matches(|c| c == '\'' || c == '"'))
                };
                return self.doc.count_children(self.element, name) as i64;
            }
        }

        if expr == "getDepth()" {
            return self.get_depth() as i64;
        }

        // Numeric literal
        expr.parse().unwrap_or(0)
    }

    /// Get the nesting depth of the current element
    fn get_depth(&self) -> usize {
        let mut depth = 0;
        let mut current_idx = Some(self.element_idx);

        while let Some(idx) = current_idx {
            if let Some(elem) = self.doc.get(idx) {
                current_idx = elem.parent;
                depth += 1;
            } else {
                break;
            }
        }

        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_doc() -> WixDocument {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*" Id="TestComponent">
      <File Source="test.exe" KeyPath="yes" />
      <File Source="test2.dll" />
    </Component>
  </Package>
</Wix>"#;
        WixDocument::parse_str(xml).unwrap()
    }

    #[test]
    fn test_attribute_exists() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name"));
        assert!(!eval.evaluate("attributes.Missing"));
        assert!(eval.evaluate("!attributes.Missing"));
    }

    #[test]
    fn test_count_children() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        assert!(eval.evaluate("countChildren('File') > 1"));
    }

    #[test]
    fn test_has_child() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        assert!(eval.evaluate("hasChild('File')"));
        assert!(!eval.evaluate("hasChild('Registry')"));
    }

    #[test]
    fn test_compound_and() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name && attributes.Version"));
        assert!(!eval.evaluate("attributes.Name && attributes.Missing"));
    }

    #[test]
    fn test_compound_or() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name || attributes.Missing"));
        assert!(eval.evaluate("attributes.Missing || attributes.Name"));
        assert!(!eval.evaluate("attributes.Missing || attributes.NotHere"));
    }

    #[test]
    fn test_equality_comparison() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name === 'Test'"));
        assert!(!eval.evaluate("attributes.Name === 'Other'"));
    }

    #[test]
    fn test_inequality_comparison() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name !== 'Other'"));
        assert!(!eval.evaluate("attributes.Name !== 'Test'"));
    }

    #[test]
    fn test_parentheses() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("(attributes.Name)"));
        assert!(!eval.evaluate("(!attributes.Name)"));
    }

    #[test]
    fn test_negation() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(!eval.evaluate("!attributes.Name"));
        assert!(eval.evaluate("!attributes.Missing"));
        assert!(eval.evaluate("!!attributes.Name"));
    }

    #[test]
    fn test_count_children_all() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        // countChildren() with no args counts all children
        assert!(eval.evaluate("countChildren()"));
    }

    #[test]
    fn test_starts_with() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        assert!(eval.evaluate("attributes.Id.startsWith('Test')"));
        assert!(!eval.evaluate("attributes.Id.startsWith('Other')"));
    }

    #[test]
    fn test_ends_with() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        assert!(eval.evaluate("attributes.Id.endsWith('Component')"));
        assert!(!eval.evaluate("attributes.Id.endsWith('Other')"));
    }

    #[test]
    fn test_is_valid_guid() {
        let doc = make_test_doc();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Component").unwrap();
        let eval = ConditionEvaluator::new(&doc, component, idx);

        assert!(eval.evaluate("isValidGuid(attributes.Guid)"));
    }

    #[test]
    fn test_is_valid_guid_literal() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package UpgradeCode="12345678-1234-1234-1234-123456789012" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("isValidGuid(attributes.UpgradeCode)"));
    }

    #[test]
    fn test_is_standard_directory() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Directory Id="ProgramFilesFolder" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let dir = doc.elements.iter().find(|e| e.name == "Directory").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Directory").unwrap();
        let eval = ConditionEvaluator::new(&doc, dir, idx);

        assert!(eval.evaluate("isStandardDirectory(attributes.Id)"));
    }

    #[test]
    fn test_is_not_standard_directory() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Directory Id="INSTALLFOLDER" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let dir = doc.elements.iter().find(|e| e.name == "Directory").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Directory").unwrap();
        let eval = ConditionEvaluator::new(&doc, dir, idx);

        assert!(!eval.evaluate("isStandardDirectory(attributes.Id)"));
    }

    #[test]
    fn test_is_sensitive_property_name() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Property Id="DATABASE_PASSWORD" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let prop = doc.elements.iter().find(|e| e.name == "Property").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Property").unwrap();
        let eval = ConditionEvaluator::new(&doc, prop, idx);

        assert!(eval.evaluate("isSensitivePropertyName(attributes.Id)"));
    }

    #[test]
    fn test_get_depth() {
        let doc = make_test_doc();
        let file = doc.elements.iter().find(|e| e.name == "File").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "File").unwrap();
        let eval = ConditionEvaluator::new(&doc, file, idx);

        // File is nested: Wix > Package > Component > File = depth 4
        assert!(eval.evaluate("getDepth()"));
    }

    #[test]
    fn test_regex_test() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Property Id="MY_PROPERTY" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let prop = doc.elements.iter().find(|e| e.name == "Property").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Property").unwrap();
        let eval = ConditionEvaluator::new(&doc, prop, idx);

        assert!(eval.evaluate("/^[A-Z_]+$/.test(attributes.Id)"));
        assert!(!eval.evaluate("/^[a-z]+$/.test(attributes.Id)"));
    }

    #[test]
    fn test_parent_expression() {
        let doc = make_test_doc();
        let file = doc.elements.iter().find(|e| e.name == "File").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "File").unwrap();
        let eval = ConditionEvaluator::new(&doc, file, idx);

        assert!(eval.evaluate("parent.countChildren('File')"));
        assert!(eval.evaluate("parent.name"));
    }

    #[test]
    fn test_numeric_literal() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("1 === '1'"));
    }

    #[test]
    fn test_string_literal_single_quotes() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name === 'Test'"));
    }

    #[test]
    fn test_string_literal_double_quotes() {
        let doc = make_test_doc();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Package").unwrap();
        let eval = ConditionEvaluator::new(&doc, package, idx);

        assert!(eval.evaluate("attributes.Name === \"Test\""));
    }

    #[test]
    fn test_to_uppercase_comparison() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Property Id="ALLUPPERCASE" />
</Wix>"#;
        let doc = WixDocument::parse_str(xml).unwrap();
        let prop = doc.elements.iter().find(|e| e.name == "Property").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Property").unwrap();
        let eval = ConditionEvaluator::new(&doc, prop, idx);

        // ALLUPPERCASE === ALLUPPERCASE.toUpperCase() should be true
        assert!(eval.evaluate("attributes.Id.toUpperCase()"));
    }

    #[test]
    fn test_no_parent() {
        let doc = make_test_doc();
        let wix = doc.elements.iter().find(|e| e.name == "Wix").unwrap();
        let idx = doc.elements.iter().position(|e| e.name == "Wix").unwrap();
        let eval = ConditionEvaluator::new(&doc, wix, idx);

        // Wix has no parent, so parent expressions should return false
        assert!(!eval.evaluate("parent.name"));
    }
}
