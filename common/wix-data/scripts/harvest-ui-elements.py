#!/usr/bin/env python3
"""
Harvest UI elements from external sources:
1. WiX UI dialog files from GitHub
2. MSI control types from Microsoft Learn

Usage:
    python3 harvest-ui-elements.py

Requires: requests (pip install requests)
"""

import json
import re
import sys
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path

try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    import urllib.request
    import ssl
    HAS_REQUESTS = False

# WiX GitHub base URL
WIX_GITHUB_RAW = "https://raw.githubusercontent.com/wixtoolset/wix/HEAD/src/ext/UI/wixlib"

# Known WiX UI dialog files
DIALOG_FILES = [
    "BrowseDlg.wxs",
    "CancelDlg.wxs",
    "CustomizeDlg.wxs",
    "DiskCostDlg.wxs",
    "ErrorDlg.wxs",
    "ExitDialog.wxs",
    "FatalError.wxs",
    "FeaturesDlg.wxs",
    "FilesInUse.wxs",
    "InstallDirDlg.wxs",
    "InstallScopeDlg.wxs",
    "LicenseAgreementDlg.wxs",
    "MaintenanceTypeDlg.wxs",
    "MaintenanceWelcomeDlg.wxs",
    "MsiRMFilesInUse.wxs",
    "OutOfDiskDlg.wxs",
    "OutOfRbDiskDlg.wxs",
    "PrepareDlg.wxs",
    "ProgressDlg.wxs",
    "ResumeDlg.wxs",
    "SetupTypeDlg.wxs",
    "UserExit.wxs",
    "VerifyReadyDlg.wxs",
    "WaitForCostingDlg.wxs",
    "WelcomeDlg.wxs",
    "WelcomeEulaDlg.wxs",
]

# MSI Control types documentation
MSI_CONTROLS_URL = "https://learn.microsoft.com/en-us/windows/win32/msi/controls"


class MSIControlsParser(HTMLParser):
    """Parse MSI controls from Microsoft Learn page

    Table structure:
    | Control name | Associated property | Brief description |
    """

    def __init__(self):
        super().__init__()
        self.controls = []
        self.in_table = False
        self.in_row = False
        self.in_cell = False
        self.current_row = []
        self.cell_text = ""

    def handle_starttag(self, tag, attrs):
        if tag == "table":
            self.in_table = True
        elif tag == "tr" and self.in_table:
            self.in_row = True
            self.current_row = []
        elif tag in ("td", "th") and self.in_row:
            self.in_cell = True
            self.cell_text = ""

    def handle_endtag(self, tag):
        if tag == "table":
            self.in_table = False
        elif tag == "tr" and self.in_row:
            self.in_row = False
            # Table has 3 columns: Name, Associated Property, Description
            if len(self.current_row) >= 3:
                name = self.current_row[0].strip()
                # Third column is the description
                desc = self.current_row[2].strip() if len(self.current_row) > 2 else ""
                # Skip header row
                if name and name != "Control name" and not name.startswith("Control name"):
                    # Clean up name (remove " control" suffix if present)
                    name = name.replace(" control", "")
                    self.controls.append({"name": name, "description": desc})
        elif tag in ("td", "th") and self.in_cell:
            self.in_cell = False
            self.current_row.append(self.cell_text)

    def handle_data(self, data):
        if self.in_cell:
            self.cell_text += data


def fetch_url(url):
    """Fetch content from URL"""
    print(f"  Fetching: {url}")
    try:
        if HAS_REQUESTS:
            response = requests.get(url, headers={"User-Agent": "WixCraft-Harvester/1.0"}, timeout=30)
            response.raise_for_status()
            return response.text
        else:
            # Fallback to urllib with SSL context
            ctx = ssl.create_default_context()
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            req = urllib.request.Request(url, headers={"User-Agent": "WixCraft-Harvester/1.0"})
            with urllib.request.urlopen(req, timeout=30, context=ctx) as response:
                return response.read().decode("utf-8")
    except Exception as e:
        print(f"    Error: {e}")
        return None


def parse_wxs_dialogs(content):
    """Parse Dialog and Control elements from WXS content"""
    dialogs = []
    controls = []

    # Remove XML namespace for easier parsing
    content = re.sub(r'xmlns="[^"]+"', '', content)

    try:
        root = ET.fromstring(content)
    except ET.ParseError as e:
        print(f"    XML parse error: {e}")
        return dialogs, controls

    # Find all Dialog elements
    for dialog in root.iter("Dialog"):
        dialog_id = dialog.get("Id")
        if dialog_id:
            width = dialog.get("Width", "")
            height = dialog.get("Height", "")
            title = dialog.get("Title", "")
            dialogs.append({
                "name": dialog_id,
                "width": width,
                "height": height,
                "title": title,
            })

            # Find controls within this dialog
            for control in dialog.iter("Control"):
                ctrl_id = control.get("Id")
                ctrl_type = control.get("Type")
                if ctrl_id and ctrl_type:
                    controls.append({
                        "dialog": dialog_id,
                        "id": ctrl_id,
                        "type": ctrl_type,
                        "x": control.get("X", ""),
                        "y": control.get("Y", ""),
                        "width": control.get("Width", ""),
                        "height": control.get("Height", ""),
                    })

    return dialogs, controls


def harvest_wix_dialogs():
    """Harvest dialogs from WiX GitHub repository"""
    print("\n=== Harvesting WiX UI Dialogs from GitHub ===")

    all_dialogs = []
    all_controls = []
    control_types = set()

    for filename in DIALOG_FILES:
        url = f"{WIX_GITHUB_RAW}/{filename}"
        content = fetch_url(url)

        if content:
            dialogs, controls = parse_wxs_dialogs(content)
            all_dialogs.extend(dialogs)
            all_controls.extend(controls)

            for ctrl in controls:
                control_types.add(ctrl["type"])

            print(f"    Found {len(dialogs)} dialogs, {len(controls)} controls")

    print(f"\nTotal: {len(all_dialogs)} dialogs, {len(all_controls)} controls")
    print(f"Control types found: {sorted(control_types)}")

    return all_dialogs, all_controls, control_types


def harvest_msi_controls():
    """Harvest MSI control types from Microsoft Learn"""
    print("\n=== Harvesting MSI Control Types from Microsoft Learn ===")

    content = fetch_url(MSI_CONTROLS_URL)
    if not content:
        return get_fallback_controls()

    parser = MSIControlsParser()
    parser.feed(content)

    if parser.controls:
        print(f"  Found {len(parser.controls)} control types")
        return parser.controls
    else:
        print("  No controls found in HTML, using fallback data")
        return get_fallback_controls()


def get_fallback_controls():
    """Fallback MSI control definitions from official documentation"""
    # These are from https://learn.microsoft.com/en-us/windows/win32/msi/controls
    return [
        {"name": "Billboard", "description": "Displays billboards during installation", "category": "display"},
        {"name": "Bitmap", "description": "Displays a static bitmap", "category": "display"},
        {"name": "CheckBox", "description": "A two-state check box", "category": "input"},
        {"name": "ComboBox", "description": "A drop-down list with edit field", "category": "input"},
        {"name": "DirectoryCombo", "description": "Shows path parts as combo box", "category": "directory"},
        {"name": "DirectoryList", "description": "Lists subdirectories", "category": "directory"},
        {"name": "Edit", "description": "A text edit field", "category": "input"},
        {"name": "GroupBox", "description": "Groups related controls", "category": "container"},
        {"name": "Hyperlink", "description": "Displays a hyperlink (MSI 5.0+)", "category": "display"},
        {"name": "Icon", "description": "Displays an icon", "category": "display"},
        {"name": "Line", "description": "Displays a horizontal/vertical line", "category": "display"},
        {"name": "ListBox", "description": "Displays list of values", "category": "input"},
        {"name": "ListView", "description": "Displays list with icons", "category": "input"},
        {"name": "MaskedEdit", "description": "Edit with input mask", "category": "input"},
        {"name": "PathEdit", "description": "Shows selected path", "category": "directory"},
        {"name": "ProgressBar", "description": "Shows installation progress", "category": "display"},
        {"name": "PushButton", "description": "A clickable button", "category": "action"},
        {"name": "RadioButtonGroup", "description": "Mutually exclusive options", "category": "input"},
        {"name": "ScrollableText", "description": "Scrolling text area", "category": "display"},
        {"name": "SelectionTree", "description": "Feature selection tree", "category": "feature"},
        {"name": "Text", "description": "Static text display", "category": "display"},
        {"name": "VolumeCostList", "description": "Shows disk space by volume", "category": "display"},
        {"name": "VolumeSelectCombo", "description": "Volume selection dropdown", "category": "directory"},
    ]


def get_wixui_dialog_sets():
    """WixUI dialog set definitions"""
    return [
        {
            "name": "WixUI_Minimal",
            "description": "Minimal UI with welcome and completion dialogs only",
            "use_case": "Simple installers with no user choices needed",
            "dialogs": ["WelcomeDlg", "ProgressDlg", "ExitDialog"],
        },
        {
            "name": "WixUI_InstallDir",
            "description": "Adds installation directory selection to minimal UI",
            "use_case": "Standard applications where users need to choose install directory",
            "dialogs": ["WelcomeDlg", "LicenseAgreementDlg", "InstallDirDlg", "VerifyReadyDlg", "ProgressDlg", "ExitDialog"],
        },
        {
            "name": "WixUI_FeatureTree",
            "description": "Feature selection tree for component choices",
            "use_case": "Applications with optional features or components",
            "dialogs": ["WelcomeDlg", "LicenseAgreementDlg", "CustomizeDlg", "VerifyReadyDlg", "ProgressDlg", "ExitDialog"],
        },
        {
            "name": "WixUI_Mondo",
            "description": "Full UI with setup type selection (Typical/Custom/Complete)",
            "use_case": "Complex installers with multiple setup types",
            "dialogs": ["WelcomeDlg", "LicenseAgreementDlg", "SetupTypeDlg", "CustomizeDlg", "VerifyReadyDlg", "ProgressDlg", "ExitDialog"],
        },
        {
            "name": "WixUI_Advanced",
            "description": "Advanced UI with per-user/per-machine installation choice",
            "use_case": "Installers needing per-user vs per-machine choice",
            "dialogs": ["WelcomeDlg", "InstallScopeDlg", "InstallDirDlg", "FeaturesDlg", "VerifyReadyDlg", "ProgressDlg", "ExitDialog"],
        },
    ]


def get_control_events():
    """Standard control events"""
    return [
        {"name": "NewDialog", "description": "Navigate to a different dialog"},
        {"name": "EndDialog", "description": "Close dialog with result (Exit, Return, Retry, Ignore)"},
        {"name": "SetProperty", "description": "Set a property value"},
        {"name": "SpawnDialog", "description": "Open modal dialog on top of current"},
        {"name": "DoAction", "description": "Execute a custom action"},
        {"name": "Reset", "description": "Reset all properties to default values"},
        {"name": "SpawnWaitDialog", "description": "Display wait dialog during action"},
        {"name": "AddLocal", "description": "Set feature to install locally"},
        {"name": "AddSource", "description": "Set feature to run from source"},
        {"name": "Remove", "description": "Set feature to not install"},
        {"name": "Reinstall", "description": "Reinstall feature"},
        {"name": "ReinstallMode", "description": "Set reinstall mode"},
    ]


def get_control_conditions():
    """Standard control conditions"""
    return [
        {"name": "Default", "description": "Control is the default button"},
        {"name": "Disable", "description": "Control is disabled (grayed out)"},
        {"name": "Enable", "description": "Control is enabled"},
        {"name": "Hide", "description": "Control is hidden"},
        {"name": "Show", "description": "Control is visible"},
    ]


def main():
    output_file = Path(__file__).parent.parent / "config" / "data" / "ui-elements.json"

    # Harvest from external sources
    dialogs, controls, control_types = harvest_wix_dialogs()
    msi_controls = harvest_msi_controls()

    # Build output structure
    output = {
        "_source": "Harvested from WiX GitHub and Microsoft Learn",
        "_harvested_from": [
            "https://github.com/wixtoolset/wix/tree/HEAD/src/ext/UI/wixlib",
            "https://learn.microsoft.com/en-us/windows/win32/msi/controls",
        ],
        "controls": msi_controls,
        "dialog_sets": get_wixui_dialog_sets(),
        "dialogs": [
            {
                "name": d["name"],
                "title": d.get("title", ""),
                "width": d.get("width", ""),
                "height": d.get("height", ""),
            }
            for d in dialogs
        ],
        "control_events": get_control_events(),
        "control_conditions": get_control_conditions(),
        "harvested_controls": [
            {
                "dialog": c["dialog"],
                "id": c["id"],
                "type": c["type"],
            }
            for c in controls
        ],
    }

    # Write output
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, "w") as f:
        json.dump(output, f, indent=2)

    print(f"\n=== Output written to {output_file} ===")
    print(f"  Controls: {len(output['controls'])}")
    print(f"  Dialog sets: {len(output['dialog_sets'])}")
    print(f"  Dialogs: {len(output['dialogs'])}")
    print(f"  Control instances: {len(output['harvested_controls'])}")


if __name__ == "__main__":
    main()
