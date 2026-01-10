//! Code complexity metrics
//!
//! Calculates McCabe cyclomatic complexity and other metrics for XML documents.

use crate::plugin::Node;
use std::collections::HashMap;

/// Complexity metrics for a document or node
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    /// McCabe cyclomatic complexity
    pub cyclomatic: usize,
    /// Number of decision points (conditions, loops)
    pub decision_points: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Total number of nodes
    pub node_count: usize,
    /// Number of attributes
    pub attribute_count: usize,
}

impl ComplexityMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate complexity rating
    pub fn rating(&self) -> ComplexityRating {
        match self.cyclomatic {
            0..=10 => ComplexityRating::Low,
            11..=20 => ComplexityRating::Moderate,
            21..=50 => ComplexityRating::High,
            _ => ComplexityRating::VeryHigh,
        }
    }
}

/// Complexity rating levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityRating {
    /// Simple, easy to maintain (1-10)
    Low,
    /// Moderate complexity (11-20)
    Moderate,
    /// High complexity, consider refactoring (21-50)
    High,
    /// Very high complexity, refactoring recommended (51+)
    VeryHigh,
}

impl std::fmt::Display for ComplexityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityRating::Low => write!(f, "low"),
            ComplexityRating::Moderate => write!(f, "moderate"),
            ComplexityRating::High => write!(f, "high"),
            ComplexityRating::VeryHigh => write!(f, "very high"),
        }
    }
}

/// Complexity analyzer for XML documents
pub struct ComplexityAnalyzer {
    /// Elements that represent decision points
    decision_elements: Vec<String>,
    /// Elements that represent conditional logic
    condition_elements: Vec<String>,
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityAnalyzer {
    /// Create a new complexity analyzer with default settings
    pub fn new() -> Self {
        Self {
            // WiX elements that represent decision points
            decision_elements: vec![
                "Condition".to_string(),
                "Launch".to_string(),
                "RegistrySearch".to_string(),
                "FileSearch".to_string(),
                "ProductSearch".to_string(),
                "ComponentSearch".to_string(),
            ],
            // Elements with conditional attributes
            condition_elements: vec![
                "Feature".to_string(),        // Level attribute can be conditional
                "Component".to_string(),      // Condition child
                "CustomAction".to_string(),   // Conditional execution
                "SetProperty".to_string(),    // Conditional property
            ],
        }
    }

    /// Add custom decision elements
    pub fn with_decision_elements(mut self, elements: &[&str]) -> Self {
        self.decision_elements.extend(elements.iter().map(|s| s.to_string()));
        self
    }

    /// Calculate complexity metrics for a node tree
    pub fn analyze(&self, root: &dyn Node) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::new();
        self.analyze_node(root, 0, &mut metrics);

        // McCabe complexity = decision points + 1
        metrics.cyclomatic = metrics.decision_points + 1;

        metrics
    }

    /// Recursively analyze a node
    fn analyze_node(&self, node: &dyn Node, depth: usize, metrics: &mut ComplexityMetrics) {
        metrics.node_count += 1;
        metrics.max_depth = metrics.max_depth.max(depth);

        // Count attributes
        for _ in node.attributes().keys() {
            metrics.attribute_count += 1;
        }

        // Check if this is a decision point
        let name = node.name();
        if self.decision_elements.iter().any(|e| e == name) {
            metrics.decision_points += 1;
        }

        // Check for conditional attributes
        if self.condition_elements.iter().any(|e| e == name) {
            // Check for Condition attribute or specific conditional attributes
            if node.attributes().contains_key("Condition") {
                metrics.decision_points += 1;
            }
        }

        // Check for inline conditions in attribute values
        for (_, value) in node.attributes().iter() {
            // Count logical operators as decision points
            let operators = value.matches(" AND ").count()
                + value.matches(" OR ").count()
                + value.matches(" and ").count()
                + value.matches(" or ").count();
            metrics.decision_points += operators;
        }

        // Recurse into children
        for child in node.children() {
            self.analyze_node(child, depth + 1, metrics);
        }
    }

    /// Calculate metrics for multiple documents and aggregate
    pub fn analyze_aggregate(&self, roots: &[&dyn Node]) -> ComplexityMetrics {
        let mut total = ComplexityMetrics::new();

        for root in roots {
            let metrics = self.analyze(*root);
            total.decision_points += metrics.decision_points;
            total.node_count += metrics.node_count;
            total.attribute_count += metrics.attribute_count;
            total.max_depth = total.max_depth.max(metrics.max_depth);
        }

        total.cyclomatic = total.decision_points + 1;
        total
    }
}

/// Per-element complexity breakdown
#[derive(Debug, Clone, Default)]
pub struct ElementComplexity {
    /// Element name
    pub name: String,
    /// Number of occurrences
    pub count: usize,
    /// Total children across all occurrences
    pub total_children: usize,
    /// Total attributes across all occurrences
    pub total_attributes: usize,
    /// Maximum depth of any occurrence
    pub max_depth: usize,
}

/// Analyze element distribution and complexity
pub fn analyze_element_distribution(root: &dyn Node) -> HashMap<String, ElementComplexity> {
    let mut distribution: HashMap<String, ElementComplexity> = HashMap::new();
    analyze_element_recursive(root, 0, &mut distribution);
    distribution
}

fn analyze_element_recursive(
    node: &dyn Node,
    depth: usize,
    distribution: &mut HashMap<String, ElementComplexity>,
) {
    let name = node.name().to_string();
    let entry = distribution.entry(name.clone()).or_insert_with(|| ElementComplexity {
        name,
        count: 0,
        total_children: 0,
        total_attributes: 0,
        max_depth: 0,
    });

    entry.count += 1;
    entry.total_children += node.children().len();
    entry.total_attributes += node.attributes().len();
    entry.max_depth = entry.max_depth.max(depth);

    for child in node.children() {
        analyze_element_recursive(child, depth + 1, distribution);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    // Mock Node implementation for testing
    struct MockNode {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<MockNode>,
    }

    impl MockNode {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                attributes: HashMap::new(),
                children: Vec::new(),
            }
        }

        fn with_attr(mut self, key: &str, value: &str) -> Self {
            self.attributes.insert(key.to_string(), value.to_string());
            self
        }

        fn with_child(mut self, child: MockNode) -> Self {
            self.children.push(child);
            self
        }
    }

    impl Node for MockNode {
        fn kind(&self) -> &str {
            "element"
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn get(&self, key: &str) -> Option<&str> {
            self.attributes.get(key).map(|s| s.as_str())
        }

        fn attributes(&self) -> &HashMap<String, String> {
            &self.attributes
        }

        fn children(&self) -> Vec<&dyn Node> {
            self.children.iter().map(|c| c as &dyn Node).collect()
        }

        fn parent(&self) -> Option<&dyn Node> {
            None
        }

        fn location(&self) -> Location {
            Location::new(PathBuf::from("test.wxs"), 1, 1)
        }

        fn text(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn test_simple_complexity() {
        let analyzer = ComplexityAnalyzer::new();
        let root = MockNode::new("Wix")
            .with_child(MockNode::new("Package"));

        let metrics = analyzer.analyze(&root);
        assert_eq!(metrics.cyclomatic, 1); // No decision points
        assert_eq!(metrics.node_count, 2);
        assert_eq!(metrics.max_depth, 1);
    }

    #[test]
    fn test_decision_points() {
        let analyzer = ComplexityAnalyzer::new();
        let root = MockNode::new("Wix")
            .with_child(MockNode::new("Package")
                .with_child(MockNode::new("Condition"))
                .with_child(MockNode::new("Condition")));

        let metrics = analyzer.analyze(&root);
        assert_eq!(metrics.decision_points, 2);
        assert_eq!(metrics.cyclomatic, 3); // 2 decision points + 1
    }

    #[test]
    fn test_logical_operators() {
        let analyzer = ComplexityAnalyzer::new();
        let root = MockNode::new("Wix")
            .with_child(MockNode::new("Package")
                .with_attr("Condition", "A AND B OR C"));

        let metrics = analyzer.analyze(&root);
        assert!(metrics.decision_points >= 2); // AND and OR
    }

    #[test]
    fn test_complexity_rating() {
        let mut metrics = ComplexityMetrics::new();

        metrics.cyclomatic = 5;
        assert_eq!(metrics.rating(), ComplexityRating::Low);

        metrics.cyclomatic = 15;
        assert_eq!(metrics.rating(), ComplexityRating::Moderate);

        metrics.cyclomatic = 30;
        assert_eq!(metrics.rating(), ComplexityRating::High);

        metrics.cyclomatic = 100;
        assert_eq!(metrics.rating(), ComplexityRating::VeryHigh);
    }

    #[test]
    fn test_depth_calculation() {
        let analyzer = ComplexityAnalyzer::new();
        let root = MockNode::new("A")
            .with_child(MockNode::new("B")
                .with_child(MockNode::new("C")
                    .with_child(MockNode::new("D"))));

        let metrics = analyzer.analyze(&root);
        assert_eq!(metrics.max_depth, 3);
    }

    #[test]
    fn test_element_distribution() {
        let root = MockNode::new("Wix")
            .with_child(MockNode::new("Package")
                .with_child(MockNode::new("Component"))
                .with_child(MockNode::new("Component")));

        let dist = analyze_element_distribution(&root);
        assert_eq!(dist.get("Component").map(|e| e.count), Some(2));
        assert_eq!(dist.get("Package").map(|e| e.count), Some(1));
    }
}
