//! Element and attribute ordering

use crate::loader::WixData;
use std::cmp::Ordering;

/// Sort child element names according to canonical wix-data order
/// Unknown elements are placed at the end in their original relative order
pub fn sort_children(
    children: &mut [(String, usize)], // (element_name, original_index)
    parent: &str,
    data: &WixData,
) {
    if let Some(order) = data.get_child_order(parent) {
        children.sort_by(|(a_name, a_idx), (b_name, b_idx)| {
            let a_pos = order.iter().position(|n| n == a_name);
            let b_pos = order.iter().position(|n| n == b_name);

            match (a_pos, b_pos) {
                // Both in canonical order
                (Some(a), Some(b)) => a.cmp(&b),
                // Only a in canonical order - it comes first
                (Some(_), None) => Ordering::Less,
                // Only b in canonical order - it comes first
                (None, Some(_)) => Ordering::Greater,
                // Neither in canonical order - preserve original order
                (None, None) => a_idx.cmp(b_idx),
            }
        });
    }
    // If parent not in wix-data, preserve original order (no sorting)
}

/// Sort attribute names according to priority
/// Id first, then required attributes, then optional alphabetically
pub fn sort_attributes(
    attrs: &mut [(String, String)], // (attr_name, attr_value)
    element: &str,
    data: Option<&WixData>,
) {
    if let Some(data) = data {
        if let Some(priority) = data.get_attr_priority(element) {
            attrs.sort_by(|(a_name, _), (b_name, _)| {
                let a_pos = priority.iter().position(|n| n == a_name);
                let b_pos = priority.iter().position(|n| n == b_name);

                match (a_pos, b_pos) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => sort_attr_fallback(a_name, b_name),
                }
            });
            return;
        }
    }

    // Fallback: Id first, then alphabetical
    attrs.sort_by(|(a_name, _), (b_name, _)| sort_attr_fallback(a_name, b_name));
}

/// Fallback attribute sort: Id/Name first, then alphabetical
fn sort_attr_fallback(a: &str, b: &str) -> Ordering {
    // Id always comes first
    if a == "Id" {
        return Ordering::Less;
    }
    if b == "Id" {
        return Ordering::Greater;
    }
    // Name comes second
    if a == "Name" {
        return Ordering::Less;
    }
    if b == "Name" {
        return Ordering::Greater;
    }
    // Then alphabetical
    a.cmp(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_data() -> (TempDir, WixData) {
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let package = r#"{
            "name": "Package",
            "children": ["Directory", "Component", "Feature"],
            "attributes": {
                "Name": {"type": "string", "required": true},
                "Guid": {"type": "guid", "required": true},
                "Id": {"type": "identifier", "required": false}
            }
        }"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_sort_children_canonical_order() {
        let (_temp, data) = create_test_data();
        let mut children = vec![
            ("Feature".to_string(), 0),
            ("Component".to_string(), 1),
            ("Directory".to_string(), 2),
        ];

        sort_children(&mut children, "Package", &data);

        assert_eq!(children[0].0, "Directory");
        assert_eq!(children[1].0, "Component");
        assert_eq!(children[2].0, "Feature");
    }

    #[test]
    fn test_sort_children_unknown_at_end() {
        let (_temp, data) = create_test_data();
        let mut children = vec![
            ("Unknown1".to_string(), 0),
            ("Directory".to_string(), 1),
            ("Unknown2".to_string(), 2),
            ("Component".to_string(), 3),
        ];

        sort_children(&mut children, "Package", &data);

        // Known elements first in canonical order
        assert_eq!(children[0].0, "Directory");
        assert_eq!(children[1].0, "Component");
        // Unknown elements at end, preserving relative order
        assert_eq!(children[2].0, "Unknown1");
        assert_eq!(children[3].0, "Unknown2");
    }

    #[test]
    fn test_sort_children_unknown_parent() {
        let (_temp, data) = create_test_data();
        let mut children = vec![
            ("C".to_string(), 0),
            ("A".to_string(), 1),
            ("B".to_string(), 2),
        ];

        sort_children(&mut children, "UnknownParent", &data);

        // Original order preserved
        assert_eq!(children[0].0, "C");
        assert_eq!(children[1].0, "A");
        assert_eq!(children[2].0, "B");
    }

    #[test]
    fn test_sort_attributes_with_priority() {
        let (_temp, data) = create_test_data();
        let mut attrs = vec![
            ("Id".to_string(), "test".to_string()),
            ("Name".to_string(), "pkg".to_string()),
            ("Guid".to_string(), "*".to_string()),
        ];

        sort_attributes(&mut attrs, "Package", Some(&data));

        // Required first (Guid, Name), then optional (Id)
        // But Id is special - within required: Guid, Name are sorted alpha
        // From loader: required attrs sorted with Id first within group
        // So: Guid, Name are required; Id is optional
        // Priority order from loader: Guid, Name (required sorted alpha), then Id
        assert_eq!(attrs[0].0, "Guid");
        assert_eq!(attrs[1].0, "Name");
        assert_eq!(attrs[2].0, "Id");
    }

    #[test]
    fn test_sort_attributes_fallback() {
        let mut attrs = vec![
            ("Zebra".to_string(), "z".to_string()),
            ("Id".to_string(), "id".to_string()),
            ("Apple".to_string(), "a".to_string()),
            ("Name".to_string(), "n".to_string()),
        ];

        sort_attributes(&mut attrs, "Unknown", None);

        // Id first, Name second, then alphabetical
        assert_eq!(attrs[0].0, "Id");
        assert_eq!(attrs[1].0, "Name");
        assert_eq!(attrs[2].0, "Apple");
        assert_eq!(attrs[3].0, "Zebra");
    }

    #[test]
    fn test_sort_attr_fallback_id_first() {
        assert_eq!(sort_attr_fallback("Id", "Name"), Ordering::Less);
        assert_eq!(sort_attr_fallback("Anything", "Id"), Ordering::Greater);
    }

    #[test]
    fn test_sort_attr_fallback_name_second() {
        assert_eq!(sort_attr_fallback("Name", "Zebra"), Ordering::Less);
        assert_eq!(sort_attr_fallback("Apple", "Name"), Ordering::Greater);
    }

    #[test]
    fn test_sort_attr_fallback_alphabetical() {
        assert_eq!(sort_attr_fallback("Apple", "Zebra"), Ordering::Less);
        assert_eq!(sort_attr_fallback("Zebra", "Apple"), Ordering::Greater);
        assert_eq!(sort_attr_fallback("Same", "Same"), Ordering::Equal);
    }

    #[test]
    fn test_sort_children_empty() {
        let (_temp, data) = create_test_data();
        let mut children: Vec<(String, usize)> = vec![];
        sort_children(&mut children, "Package", &data);
        assert!(children.is_empty());
    }

    #[test]
    fn test_sort_attributes_empty() {
        let mut attrs: Vec<(String, String)> = vec![];
        sort_attributes(&mut attrs, "Element", None);
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_sort_children_single() {
        let (_temp, data) = create_test_data();
        let mut children = vec![("Directory".to_string(), 0)];
        sort_children(&mut children, "Package", &data);
        assert_eq!(children[0].0, "Directory");
    }

    #[test]
    fn test_sort_attributes_unknown_element_with_data() {
        let (_temp, data) = create_test_data();
        let mut attrs = vec![
            ("Zebra".to_string(), "z".to_string()),
            ("Id".to_string(), "id".to_string()),
        ];

        // Element not in wix-data, should use fallback
        sort_attributes(&mut attrs, "UnknownElement", Some(&data));

        assert_eq!(attrs[0].0, "Id");
        assert_eq!(attrs[1].0, "Zebra");
    }

    #[test]
    fn test_sort_children_one_unknown_first() {
        // Test case where unknown element comes before known
        let (_temp, data) = create_test_data();
        let mut children = vec![
            ("Unknown".to_string(), 0),
            ("Directory".to_string(), 1),
        ];

        sort_children(&mut children, "Package", &data);

        // Directory (known) should come first, Unknown at end
        assert_eq!(children[0].0, "Directory");
        assert_eq!(children[1].0, "Unknown");
    }

    #[test]
    fn test_sort_children_known_before_unknown() {
        // Test the (None, Some(_)) => Ordering::Greater branch
        let (_temp, data) = create_test_data();
        let mut children = vec![
            ("Component".to_string(), 0), // Known
            ("Unknown".to_string(), 1),    // Unknown should stay after
        ];

        sort_children(&mut children, "Package", &data);

        assert_eq!(children[0].0, "Component");
        assert_eq!(children[1].0, "Unknown");
    }

    #[test]
    fn test_sort_attributes_mixed_known_unknown() {
        // Test attribute sorting with mix of known and unknown attrs
        let (_temp, data) = create_test_data();
        let mut attrs = vec![
            ("Unknown1".to_string(), "u1".to_string()),  // Not in priority
            ("Guid".to_string(), "*".to_string()),       // In priority
            ("Unknown2".to_string(), "u2".to_string()),  // Not in priority
        ];

        sort_attributes(&mut attrs, "Package", Some(&data));

        // Guid is known and should be first
        assert_eq!(attrs[0].0, "Guid");
        // Unknown attrs use fallback alphabetical
        assert_eq!(attrs[1].0, "Unknown1");
        assert_eq!(attrs[2].0, "Unknown2");
    }

    #[test]
    fn test_sort_attributes_only_one_known() {
        // Only one attribute is in the priority list
        let (_temp, data) = create_test_data();
        let mut attrs = vec![
            ("Zebra".to_string(), "z".to_string()),  // Not in priority
            ("Name".to_string(), "n".to_string()),   // In priority
        ];

        sort_attributes(&mut attrs, "Package", Some(&data));

        // Name is in priority, should be first
        assert_eq!(attrs[0].0, "Name");
        assert_eq!(attrs[1].0, "Zebra");
    }

    #[test]
    fn test_sort_attributes_known_after_unknown() {
        // Test the (None, Some(_)) => Ordering::Greater branch for attributes
        let (_temp, data) = create_test_data();
        let mut attrs = vec![
            ("Alpha".to_string(), "a".to_string()),  // Not in priority (alphabetically first)
            ("Guid".to_string(), "*".to_string()),   // In priority
        ];

        sort_attributes(&mut attrs, "Package", Some(&data));

        // Guid (known priority) should come first despite Alpha being alphabetically first
        assert_eq!(attrs[0].0, "Guid");
        assert_eq!(attrs[1].0, "Alpha");
    }
}
