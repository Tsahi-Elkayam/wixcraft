#!/usr/bin/env python3
"""
WiX XSD Schema Extractor

Parses WiX XSD schema files and generates JSON element definitions
for the wix-data database.

Usage:
    python extract-xsd.py [--url URL] [--output DIR]

Example:
    python extract-xsd.py --url https://raw.githubusercontent.com/wixtoolset/web/master/src/xsd4/wix.xsd --output ../src/core/wix-data/elements
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any
from urllib.request import urlopen
from xml.etree import ElementTree as ET

# XML Schema namespace
XS_NS = "{http://www.w3.org/2001/XMLSchema}"

# WiX namespaces
WIX_NS = {
    "xs": "http://www.w3.org/2001/XMLSchema",
    "wix": "http://wixtoolset.org/schemas/v4/wxs",
    "html": "http://www.w3.org/1999/xhtml"
}


def fetch_xsd(url: str) -> str:
    """Fetch XSD content from URL."""
    print(f"Fetching XSD from {url}...")
    with urlopen(url) as response:
        return response.read().decode("utf-8")


def parse_documentation(element: ET.Element) -> str:
    """Extract documentation text from xs:annotation/xs:documentation."""
    doc = ""
    annotation = element.find(f"{XS_NS}annotation")
    if annotation is not None:
        documentation = annotation.find(f"{XS_NS}documentation")
        if documentation is not None:
            # Get all text content, stripping HTML
            doc = "".join(documentation.itertext()).strip()
            # Clean up whitespace
            doc = re.sub(r"\s+", " ", doc)
    return doc


def parse_type_name(type_str: str | None) -> str:
    """Convert XSD type to our type system."""
    if type_str is None:
        return "string"

    # Remove namespace prefix
    type_str = type_str.split(":")[-1]

    type_map = {
        "string": "string",
        "NMTOKEN": "string",
        "NMTOKENS": "string",
        "token": "string",
        "normalizedString": "string",
        "integer": "integer",
        "int": "integer",
        "positiveInteger": "integer",
        "nonNegativeInteger": "integer",
        "long": "integer",
        "short": "integer",
        "boolean": "yesno",
        "Guid": "guid",
        "GuidType": "guid",
        "ComponentGuid": "guid",
        "AutogenGuid": "guid",
        "YesNoType": "yesno",
        "YesNoDefaultType": "yesno",
        "YesNoButtonType": "yesno",
        "VersionType": "version",
        "LocalizableInteger": "integer",
    }

    return type_map.get(type_str, "string")


def parse_enum_values(simple_type: ET.Element) -> list[str]:
    """Extract enum values from xs:simpleType with xs:restriction/xs:enumeration."""
    values = []
    restriction = simple_type.find(f"{XS_NS}restriction")
    if restriction is not None:
        for enum in restriction.findall(f"{XS_NS}enumeration"):
            value = enum.get("value")
            if value:
                values.append(value)
    return values


def parse_attribute(attr_elem: ET.Element, simple_types: dict) -> dict[str, Any]:
    """Parse xs:attribute element into our attribute schema."""
    name = attr_elem.get("name", "")
    attr_type = attr_elem.get("type", "string")
    use = attr_elem.get("use", "optional")
    default = attr_elem.get("default")

    attr_def = {
        "type": parse_type_name(attr_type),
        "required": use == "required",
        "description": parse_documentation(attr_elem)
    }

    if default:
        attr_def["default"] = default

    # Check if it's an enum type
    type_name = attr_type.split(":")[-1] if attr_type else None
    if type_name and type_name in simple_types:
        values = simple_types[type_name]
        if values:
            attr_def["type"] = "enum"
            attr_def["values"] = values

    # Check for inline simpleType
    inline_type = attr_elem.find(f"{XS_NS}simpleType")
    if inline_type is not None:
        values = parse_enum_values(inline_type)
        if values:
            attr_def["type"] = "enum"
            attr_def["values"] = values

    return attr_def


def parse_element(elem: ET.Element, simple_types: dict, all_elements: dict) -> dict[str, Any] | None:
    """Parse xs:element into our element schema."""
    name = elem.get("name")
    if not name:
        return None

    element_def = {
        "name": name,
        "namespace": "wix",
        "since": "v4",
        "description": parse_documentation(elem),
        "documentation": f"https://wixtoolset.org/docs/schema/wxs/{name.lower()}/",
        "parents": [],
        "children": [],
        "attributes": {}
    }

    # Find complexType (inline or referenced)
    complex_type = elem.find(f"{XS_NS}complexType")
    if complex_type is None:
        type_ref = elem.get("type")
        if type_ref:
            type_name = type_ref.split(":")[-1]
            complex_type = all_elements.get(f"type:{type_name}")

    if complex_type is not None:
        # Parse attributes
        for attr in complex_type.findall(f".//{XS_NS}attribute"):
            attr_name = attr.get("name")
            if attr_name:
                element_def["attributes"][attr_name] = parse_attribute(attr, simple_types)

        # Parse child elements from sequence/choice/all
        for child_ref in complex_type.findall(f".//{XS_NS}element"):
            child_name = child_ref.get("ref") or child_ref.get("name")
            if child_name:
                child_name = child_name.split(":")[-1]
                if child_name not in element_def["children"]:
                    element_def["children"].append(child_name)

    return element_def


def extract_simple_types(root: ET.Element) -> dict[str, list[str]]:
    """Extract all simpleType definitions with enumerations."""
    simple_types = {}

    for simple_type in root.findall(f".//{XS_NS}simpleType"):
        name = simple_type.get("name")
        if name:
            values = parse_enum_values(simple_type)
            if values:
                simple_types[name] = values

    return simple_types


def extract_elements(root: ET.Element, simple_types: dict) -> list[dict[str, Any]]:
    """Extract all top-level element definitions."""
    elements = []
    all_elements = {}

    # First pass: collect all types
    for complex_type in root.findall(f"{XS_NS}complexType"):
        name = complex_type.get("name")
        if name:
            all_elements[f"type:{name}"] = complex_type

    # Second pass: parse elements
    for elem in root.findall(f"{XS_NS}element"):
        element_def = parse_element(elem, simple_types, all_elements)
        if element_def:
            elements.append(element_def)

    return elements


def save_element(element_def: dict[str, Any], output_dir: Path) -> None:
    """Save element definition to JSON file."""
    name = element_def["name"].lower()
    output_path = output_dir / f"{name}.json"

    # Add schema reference
    element_def["$schema"] = "../schema/element.schema.json"

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(element_def, f, indent=2)

    print(f"  Created {output_path.name}")


def main():
    parser = argparse.ArgumentParser(description="Extract WiX XSD schema to JSON")
    parser.add_argument(
        "--url",
        default="https://raw.githubusercontent.com/wixtoolset/web/master/src/xsd4/wix.xsd",
        help="URL of the WiX XSD schema"
    )
    parser.add_argument(
        "--output",
        default="src/core/wix-data/elements",
        help="Output directory for JSON files"
    )
    parser.add_argument(
        "--local",
        help="Path to local XSD file instead of URL"
    )

    args = parser.parse_args()

    # Get XSD content
    if args.local:
        print(f"Reading local file {args.local}...")
        with open(args.local, "r", encoding="utf-8") as f:
            xsd_content = f.read()
    else:
        xsd_content = fetch_xsd(args.url)

    # Parse XSD
    print("Parsing XSD schema...")
    root = ET.fromstring(xsd_content)

    # Extract simple types (enums)
    simple_types = extract_simple_types(root)
    print(f"Found {len(simple_types)} enum types")

    # Extract elements
    elements = extract_elements(root, simple_types)
    print(f"Found {len(elements)} elements")

    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save elements
    print(f"\nSaving elements to {output_dir}...")
    for element_def in elements:
        save_element(element_def, output_dir)

    print(f"\nDone! Extracted {len(elements)} elements.")

    # Print summary
    print("\nElements extracted:")
    for elem in sorted(elements, key=lambda x: x["name"]):
        attr_count = len(elem["attributes"])
        child_count = len(elem["children"])
        print(f"  {elem['name']}: {attr_count} attributes, {child_count} children")


if __name__ == "__main__":
    main()
