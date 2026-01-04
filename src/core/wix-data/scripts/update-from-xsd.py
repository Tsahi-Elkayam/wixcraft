#!/usr/bin/env python3
"""
update-from-xsd.py - Extract WiX element definitions from XSD schemas

This script downloads and parses WiX Toolset XSD schema files to extract
element definitions, attributes, and relationships. The extracted data
is written to JSON files in the wix-data/elements directory.

Usage:
    python update-from-xsd.py [--force] [--dry-run] [--verbose]

Options:
    --force     Overwrite existing element files
    --dry-run   Show what would be done without writing files
    --verbose   Print detailed progress information
    --offline   Use cached XSD files only (don't download)
"""

import argparse
import json
import os
import re
import sys
import urllib.request
from pathlib import Path
from xml.etree import ElementTree as ET
from typing import Dict, List, Optional, Any
from datetime import datetime

# XSD namespace
XS = "{http://www.w3.org/2001/XMLSchema}"

# WiX XSD URLs (WiX v4)
WIX_XSD_URLS = {
    "wxs": "https://raw.githubusercontent.com/wixtoolset/wix/main/src/xsd/wix.xsd",
    "bal": "https://raw.githubusercontent.com/wixtoolset/wix/main/src/xsd/bal.xsd",
    "util": "https://raw.githubusercontent.com/wixtoolset/wix/main/src/xsd/util.xsd",
    "netfx": "https://raw.githubusercontent.com/wixtoolset/wix/main/src/xsd/netfx.xsd",
    "ui": "https://raw.githubusercontent.com/wixtoolset/wix/main/src/xsd/ui.xsd",
}

# Alternative URLs for WiX v4 (wixtoolset.org)
WIX_XSD_URLS_ALT = {
    "wxs": "https://wixtoolset.org/schemas/v4/wxs/wix.xsd",
}


class XsdParser:
    """Parses WiX XSD files and extracts element definitions."""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.elements: Dict[str, Dict[str, Any]] = {}
        self.types: Dict[str, Dict[str, Any]] = {}
        self.groups: Dict[str, List[str]] = {}

    def log(self, msg: str):
        if self.verbose:
            print(f"  {msg}")

    def parse_file(self, xsd_path: Path, namespace: str = "wix") -> None:
        """Parse an XSD file and extract element definitions."""
        self.log(f"Parsing {xsd_path}")

        try:
            tree = ET.parse(xsd_path)
            root = tree.getroot()
        except ET.ParseError as e:
            print(f"Error parsing {xsd_path}: {e}", file=sys.stderr)
            return

        # First pass: collect all types and groups
        for type_elem in root.findall(f".//{XS}complexType[@name]"):
            type_name = type_elem.get("name")
            self.types[type_name] = self._parse_complex_type(type_elem)

        for type_elem in root.findall(f".//{XS}simpleType[@name]"):
            type_name = type_elem.get("name")
            self.types[type_name] = self._parse_simple_type(type_elem)

        for group_elem in root.findall(f".//{XS}group[@name]"):
            group_name = group_elem.get("name")
            self.groups[group_name] = self._parse_group(group_elem)

        # Second pass: extract elements
        for elem in root.findall(f"./{XS}element"):
            element_def = self._parse_element(elem, namespace)
            if element_def:
                self.elements[element_def["name"]] = element_def

    def _parse_element(self, elem: ET.Element, namespace: str) -> Optional[Dict[str, Any]]:
        """Parse an xs:element definition."""
        name = elem.get("name")
        if not name:
            return None

        self.log(f"  Found element: {name}")

        # Get documentation
        doc = self._get_documentation(elem)

        # Get type info
        type_name = elem.get("type")
        attributes = {}
        children = []

        if type_name and type_name in self.types:
            type_info = self.types[type_name]
            attributes = type_info.get("attributes", {})
            children = type_info.get("children", [])
        else:
            # Inline complex type
            complex_type = elem.find(f"./{XS}complexType")
            if complex_type is not None:
                type_info = self._parse_complex_type(complex_type)
                attributes = type_info.get("attributes", {})
                children = type_info.get("children", [])

        return {
            "name": name,
            "namespace": namespace,
            "since": "v4",
            "description": doc or f"The {name} element.",
            "documentation": f"https://wixtoolset.org/docs/schema/wxs/{name.lower()}/",
            "parents": [],  # Filled in during relationship resolution
            "children": children,
            "attributes": attributes,
        }

    def _parse_complex_type(self, type_elem: ET.Element) -> Dict[str, Any]:
        """Parse a complexType definition."""
        result = {
            "attributes": {},
            "children": [],
        }

        # Parse attributes
        for attr in type_elem.findall(f".//{XS}attribute"):
            attr_def = self._parse_attribute(attr)
            if attr_def:
                result["attributes"][attr_def["name"]] = attr_def["definition"]

        # Parse child elements from sequence/choice/all
        for container in [f"{XS}sequence", f"{XS}choice", f"{XS}all"]:
            for child_elem in type_elem.findall(f".//{container}/{XS}element"):
                ref = child_elem.get("ref")
                if ref:
                    result["children"].append(ref)

        # Parse group references
        for group_ref in type_elem.findall(f".//{XS}group"):
            ref = group_ref.get("ref")
            if ref and ref in self.groups:
                result["children"].extend(self.groups[ref])

        return result

    def _parse_simple_type(self, type_elem: ET.Element) -> Dict[str, Any]:
        """Parse a simpleType definition."""
        result = {"type": "string"}

        # Check for enumeration
        enum_values = []
        for enum in type_elem.findall(f".//{XS}enumeration"):
            value = enum.get("value")
            if value:
                enum_values.append(value)

        if enum_values:
            result["type"] = "enum"
            result["values"] = enum_values

        return result

    def _parse_group(self, group_elem: ET.Element) -> List[str]:
        """Parse a group definition and return child element names."""
        children = []
        for child_elem in group_elem.findall(f".//{XS}element"):
            ref = child_elem.get("ref")
            if ref:
                children.append(ref)
        return children

    def _parse_attribute(self, attr_elem: ET.Element) -> Optional[Dict[str, Any]]:
        """Parse an attribute definition."""
        name = attr_elem.get("name")
        if not name:
            return None

        doc = self._get_documentation(attr_elem)
        attr_type = attr_elem.get("type", "string")
        use = attr_elem.get("use", "optional")
        default = attr_elem.get("default")

        # Map XSD types to our types
        type_mapping = {
            "xs:string": "string",
            "xs:boolean": "yesno",
            "xs:integer": "integer",
            "xs:int": "integer",
            "xs:positiveInteger": "integer",
            "xs:nonNegativeInteger": "integer",
            "YesNoType": "yesno",
            "Guid": "guid",
            "VersionType": "version",
        }

        mapped_type = type_mapping.get(attr_type, "string")

        # Check if type is an enum
        if attr_type in self.types:
            type_info = self.types[attr_type]
            if type_info.get("type") == "enum":
                mapped_type = "enum"

        definition = {
            "type": mapped_type,
            "required": use == "required",
            "description": doc or f"The {name} attribute.",
        }

        if default:
            definition["default"] = default

        if mapped_type == "enum" and attr_type in self.types:
            definition["values"] = self.types[attr_type].get("values", [])

        return {"name": name, "definition": definition}

    def _get_documentation(self, elem: ET.Element) -> Optional[str]:
        """Extract documentation from xs:annotation/xs:documentation."""
        doc_elem = elem.find(f".//{XS}documentation")
        if doc_elem is not None and doc_elem.text:
            # Clean up whitespace
            text = " ".join(doc_elem.text.split())
            return text
        return None

    def resolve_relationships(self) -> None:
        """Resolve parent-child relationships between elements."""
        self.log("Resolving element relationships...")

        # Build parent relationships from children
        for name, element in self.elements.items():
            for child_name in element.get("children", []):
                if child_name in self.elements:
                    if name not in self.elements[child_name].get("parents", []):
                        self.elements[child_name].setdefault("parents", []).append(name)


def download_xsd(url: str, cache_dir: Path, verbose: bool = False) -> Optional[Path]:
    """Download an XSD file and cache it locally."""
    filename = url.split("/")[-1]
    cache_path = cache_dir / filename

    if cache_path.exists():
        if verbose:
            print(f"  Using cached: {cache_path}")
        return cache_path

    if verbose:
        print(f"  Downloading: {url}")

    try:
        req = urllib.request.Request(url, headers={"User-Agent": "wix-data-updater/1.0"})
        with urllib.request.urlopen(req, timeout=30) as response:
            content = response.read()
            cache_path.write_bytes(content)
            return cache_path
    except Exception as e:
        print(f"  Failed to download {url}: {e}", file=sys.stderr)
        return None


def load_existing_element(elements_dir: Path, name: str) -> Optional[Dict[str, Any]]:
    """Load an existing element JSON file."""
    file_path = elements_dir / f"{name.lower()}.json"
    if file_path.exists():
        try:
            with open(file_path) as f:
                return json.load(f)
        except json.JSONDecodeError:
            pass
    return None


def merge_element(existing: Dict[str, Any], extracted: Dict[str, Any]) -> Dict[str, Any]:
    """Merge extracted data into existing element, preserving manual edits."""
    result = existing.copy()

    # Update fields from XSD, but preserve manual additions
    if "description" in extracted and not existing.get("description"):
        result["description"] = extracted["description"]

    # Merge attributes - add new ones, update existing types
    existing_attrs = existing.get("attributes", {})
    for attr_name, attr_def in extracted.get("attributes", {}).items():
        if attr_name not in existing_attrs:
            existing_attrs[attr_name] = attr_def
        else:
            # Preserve manual description if it exists
            if not existing_attrs[attr_name].get("description"):
                existing_attrs[attr_name]["description"] = attr_def.get("description", "")
    result["attributes"] = existing_attrs

    # Merge children - add new ones
    existing_children = set(existing.get("children", []))
    for child in extracted.get("children", []):
        existing_children.add(child)
    result["children"] = sorted(existing_children)

    # Merge parents - add new ones
    existing_parents = set(existing.get("parents", []))
    for parent in extracted.get("parents", []):
        existing_parents.add(parent)
    result["parents"] = sorted(existing_parents)

    return result


def write_element(elements_dir: Path, element: Dict[str, Any], dry_run: bool = False) -> bool:
    """Write an element to a JSON file."""
    name = element["name"]
    file_path = elements_dir / f"{name.lower()}.json"

    # Add schema reference
    output = {"$schema": "../schema/element.schema.json"}
    output.update(element)

    # Sort keys for consistent output
    json_str = json.dumps(output, indent=2, sort_keys=False)

    if dry_run:
        print(f"  Would write: {file_path}")
        return True

    try:
        with open(file_path, "w") as f:
            f.write(json_str)
            f.write("\n")
        return True
    except IOError as e:
        print(f"  Failed to write {file_path}: {e}", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Extract WiX element definitions from XSD schemas"
    )
    parser.add_argument("--force", action="store_true", help="Overwrite existing files")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--offline", action="store_true", help="Use cached XSD only")
    parser.add_argument("--xsd", type=Path, help="Path to local XSD file")
    args = parser.parse_args()

    # Determine paths
    script_dir = Path(__file__).parent
    wix_data_dir = script_dir.parent
    elements_dir = wix_data_dir / "elements"
    cache_dir = script_dir / ".cache"

    # Create directories
    cache_dir.mkdir(exist_ok=True)
    elements_dir.mkdir(exist_ok=True)

    print("WiX Data Updater")
    print("=" * 40)
    print(f"Elements directory: {elements_dir}")
    print(f"Cache directory: {cache_dir}")
    print()

    # Initialize parser
    xsd_parser = XsdParser(verbose=args.verbose)

    # Download and parse XSD files
    if args.xsd:
        # Use local XSD file
        if args.xsd.exists():
            xsd_parser.parse_file(args.xsd)
        else:
            print(f"Error: XSD file not found: {args.xsd}", file=sys.stderr)
            return 1
    else:
        # Download from URLs
        print("Downloading WiX XSD schemas...")
        for namespace, url in WIX_XSD_URLS.items():
            if args.offline:
                xsd_path = cache_dir / url.split("/")[-1]
                if not xsd_path.exists():
                    print(f"  Skipping {namespace} (not cached)")
                    continue
            else:
                xsd_path = download_xsd(url, cache_dir, args.verbose)
                if not xsd_path:
                    # Try alternative URL
                    if namespace in WIX_XSD_URLS_ALT:
                        xsd_path = download_xsd(
                            WIX_XSD_URLS_ALT[namespace], cache_dir, args.verbose
                        )

            if xsd_path:
                xsd_parser.parse_file(xsd_path, namespace)

    # Resolve relationships
    xsd_parser.resolve_relationships()

    # Process elements
    print()
    print(f"Found {len(xsd_parser.elements)} elements in XSD")
    print()

    if not xsd_parser.elements:
        print("No elements extracted. The XSD format may have changed.")
        print("Consider updating the parser or providing a local XSD file.")
        return 1

    # Write elements
    print("Processing elements...")
    created = 0
    updated = 0
    skipped = 0

    for name, element in sorted(xsd_parser.elements.items()):
        existing = load_existing_element(elements_dir, name)

        if existing:
            if args.force:
                merged = merge_element(existing, element)
                if write_element(elements_dir, merged, args.dry_run):
                    updated += 1
                    if args.verbose:
                        print(f"  Updated: {name}")
            else:
                skipped += 1
                if args.verbose:
                    print(f"  Skipped (exists): {name}")
        else:
            if write_element(elements_dir, element, args.dry_run):
                created += 1
                if args.verbose:
                    print(f"  Created: {name}")

    # Summary
    print()
    print("Summary")
    print("-" * 40)
    print(f"  Created: {created}")
    print(f"  Updated: {updated}")
    print(f"  Skipped: {skipped}")
    print()

    if args.dry_run:
        print("(Dry run - no files were written)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
