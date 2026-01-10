#!/usr/bin/env python3
"""Add column definitions to MSI tables that are missing them."""

import sqlite3
import json
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# MSI table column definitions per Windows Installer SDK
# Format: table_name -> [{"name": str, "type": str, "nullable": bool, "key": bool, "description": str}, ...]
MSI_TABLE_COLUMNS = {
    "ActionText": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "Localized description of action."},
        {"name": "Template", "type": "Template", "nullable": True, "key": False, "description": "Template for formatting progress messages."},
    ],
    "AdminExecuteSequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action to execute."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Conditional expression for execution."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number in admin install."},
    ],
    "AdminUISequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action to execute."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Conditional expression for execution."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number in admin UI."},
    ],
    "AdvtExecuteSequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action to execute."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Conditional expression for execution."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number in advertisement."},
    ],
    "AdvtUISequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action to execute."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Conditional expression for execution."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number."},
    ],
    "AppId": [
        {"name": "AppId", "type": "GUID", "nullable": False, "key": True, "description": "The AppId GUID."},
        {"name": "RemoteServerName", "type": "Text", "nullable": True, "key": False, "description": "Remote server name for DCOM."},
        {"name": "LocalService", "type": "Text", "nullable": True, "key": False, "description": "Local service name."},
        {"name": "ServiceParameters", "type": "Text", "nullable": True, "key": False, "description": "Service command-line parameters."},
        {"name": "DllSurrogate", "type": "Text", "nullable": True, "key": False, "description": "DLL surrogate path."},
        {"name": "ActivateAtStorage", "type": "Integer", "nullable": True, "key": False, "description": "1 to activate at storage."},
        {"name": "RunAsInteractiveUser", "type": "Integer", "nullable": True, "key": False, "description": "1 to run as interactive user."},
    ],
    "AppSearch": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property to set with search result."},
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Signature table."},
    ],
    "BBControl": [
        {"name": "Billboard_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Billboard table."},
        {"name": "BBControl", "type": "Identifier", "nullable": False, "key": True, "description": "Control identifier."},
        {"name": "Type", "type": "Identifier", "nullable": False, "key": False, "description": "Control type (Text, Bitmap, etc.)."},
        {"name": "X", "type": "Integer", "nullable": False, "key": False, "description": "X coordinate in installer units."},
        {"name": "Y", "type": "Integer", "nullable": False, "key": False, "description": "Y coordinate in installer units."},
        {"name": "Width", "type": "Integer", "nullable": False, "key": False, "description": "Width in installer units."},
        {"name": "Height", "type": "Integer", "nullable": False, "key": False, "description": "Height in installer units."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Control attributes bitmask."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Control text or property reference."},
    ],
    "Billboard": [
        {"name": "Billboard", "type": "Identifier", "nullable": False, "key": True, "description": "Billboard identifier."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Feature that triggers this billboard."},
        {"name": "Action", "type": "Identifier", "nullable": True, "key": False, "description": "Action that triggers this billboard."},
        {"name": "Ordering", "type": "Integer", "nullable": True, "key": False, "description": "Display order within feature."},
    ],
    "Binary": [
        {"name": "Name", "type": "Identifier", "nullable": False, "key": True, "description": "Unique binary data identifier."},
        {"name": "Data", "type": "Binary", "nullable": False, "key": False, "description": "Binary stream data."},
    ],
    "BindImage": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to File table."},
        {"name": "Path", "type": "Paths", "nullable": True, "key": False, "description": "Semicolon-delimited DLL search paths."},
    ],
    "CCPSearch": [
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Signature table."},
    ],
    "CheckBox": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Value when checked."},
    ],
    "Class": [
        {"name": "CLSID", "type": "GUID", "nullable": False, "key": True, "description": "Class identifier (CLSID)."},
        {"name": "Context", "type": "Identifier", "nullable": False, "key": True, "description": "Server context (LocalServer32, InprocServer32)."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "ProgId_Default", "type": "Text", "nullable": True, "key": False, "description": "Default ProgId for this class."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "Class description."},
        {"name": "AppId_", "type": "GUID", "nullable": True, "key": False, "description": "Reference to AppId table."},
        {"name": "FileTypeMask", "type": "Text", "nullable": True, "key": False, "description": "File type mask for detection."},
        {"name": "Icon_", "type": "Identifier", "nullable": True, "key": False, "description": "Reference to Icon table."},
        {"name": "IconIndex", "type": "Integer", "nullable": True, "key": False, "description": "Icon index within file."},
        {"name": "DefInprocHandler", "type": "Text", "nullable": True, "key": False, "description": "Default in-process handler."},
        {"name": "Argument", "type": "Formatted", "nullable": True, "key": False, "description": "Command-line arguments."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Feature table."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Class registration attributes."},
    ],
    "ComboBox": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Order", "type": "Integer", "nullable": False, "key": True, "description": "Item display order."},
        {"name": "Value", "type": "Formatted", "nullable": False, "key": False, "description": "Item value."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Display text."},
    ],
    "CompLocator": [
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Signature table."},
        {"name": "ComponentId", "type": "GUID", "nullable": False, "key": False, "description": "Component code GUID."},
        {"name": "Type", "type": "Integer", "nullable": True, "key": False, "description": "Search type (0=directory, 1=file)."},
    ],
    "Complus": [
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "ExpType", "type": "Integer", "nullable": True, "key": False, "description": "COM+ application export type."},
    ],
    "Component": [
        {"name": "Component", "type": "Identifier", "nullable": False, "key": True, "description": "Component identifier."},
        {"name": "ComponentId", "type": "GUID", "nullable": True, "key": False, "description": "Component code GUID."},
        {"name": "Directory_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Directory table."},
        {"name": "Attributes", "type": "Integer", "nullable": False, "key": False, "description": "Component attributes bitmask."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Installation condition."},
        {"name": "KeyPath", "type": "Identifier", "nullable": True, "key": False, "description": "Key path resource identifier."},
    ],
    "Condition": [
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Feature table."},
        {"name": "Level", "type": "Integer", "nullable": False, "key": True, "description": "New install level if condition is true."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Conditional expression."},
    ],
    "Control": [
        {"name": "Dialog_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Dialog table."},
        {"name": "Control", "type": "Identifier", "nullable": False, "key": True, "description": "Control identifier."},
        {"name": "Type", "type": "Identifier", "nullable": False, "key": False, "description": "Control type."},
        {"name": "X", "type": "Integer", "nullable": False, "key": False, "description": "X coordinate."},
        {"name": "Y", "type": "Integer", "nullable": False, "key": False, "description": "Y coordinate."},
        {"name": "Width", "type": "Integer", "nullable": False, "key": False, "description": "Control width."},
        {"name": "Height", "type": "Integer", "nullable": False, "key": False, "description": "Control height."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Control attributes."},
        {"name": "Property", "type": "Identifier", "nullable": True, "key": False, "description": "Associated property."},
        {"name": "Text", "type": "Formatted", "nullable": True, "key": False, "description": "Control text."},
        {"name": "Control_Next", "type": "Identifier", "nullable": True, "key": False, "description": "Next control in tab order."},
        {"name": "Help", "type": "Text", "nullable": True, "key": False, "description": "Tooltip and context help."},
    ],
    "ControlCondition": [
        {"name": "Dialog_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Dialog table."},
        {"name": "Control_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Control table."},
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Action (Default, Disable, Enable, Hide, Show)."},
        {"name": "Condition", "type": "Condition", "nullable": False, "key": True, "description": "Condition for the action."},
    ],
    "ControlEvent": [
        {"name": "Dialog_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Dialog table."},
        {"name": "Control_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Control table."},
        {"name": "Event", "type": "Formatted", "nullable": False, "key": True, "description": "Event name."},
        {"name": "Argument", "type": "Formatted", "nullable": False, "key": True, "description": "Event argument."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": True, "description": "Condition for event."},
        {"name": "Ordering", "type": "Integer", "nullable": True, "key": False, "description": "Event processing order."},
    ],
    "CreateFolder": [
        {"name": "Directory_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Directory table."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
    ],
    "CustomAction": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Custom action identifier."},
        {"name": "Type", "type": "Integer", "nullable": False, "key": False, "description": "Custom action type number."},
        {"name": "Source", "type": "Identifier", "nullable": True, "key": False, "description": "Source (Binary, Property, Directory)."},
        {"name": "Target", "type": "Formatted", "nullable": True, "key": False, "description": "Target (DLL entry, EXE path, script)."},
        {"name": "ExtendedType", "type": "Integer", "nullable": True, "key": False, "description": "Extended type attributes."},
    ],
    "Dialog": [
        {"name": "Dialog", "type": "Identifier", "nullable": False, "key": True, "description": "Dialog identifier."},
        {"name": "HCentering", "type": "Integer", "nullable": False, "key": False, "description": "Horizontal centering percent."},
        {"name": "VCentering", "type": "Integer", "nullable": False, "key": False, "description": "Vertical centering percent."},
        {"name": "Width", "type": "Integer", "nullable": False, "key": False, "description": "Dialog width."},
        {"name": "Height", "type": "Integer", "nullable": False, "key": False, "description": "Dialog height."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Dialog attributes."},
        {"name": "Title", "type": "Formatted", "nullable": True, "key": False, "description": "Dialog title."},
        {"name": "Control_First", "type": "Identifier", "nullable": False, "key": False, "description": "First control in tab order."},
        {"name": "Control_Default", "type": "Identifier", "nullable": True, "key": False, "description": "Default pushbutton."},
        {"name": "Control_Cancel", "type": "Identifier", "nullable": True, "key": False, "description": "Cancel pushbutton."},
    ],
    "Directory": [
        {"name": "Directory", "type": "Identifier", "nullable": False, "key": True, "description": "Directory identifier."},
        {"name": "Directory_Parent", "type": "Identifier", "nullable": True, "key": False, "description": "Parent directory."},
        {"name": "DefaultDir", "type": "DefaultDir", "nullable": False, "key": False, "description": "Default directory name."},
    ],
    "DrLocator": [
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Signature table."},
        {"name": "Parent", "type": "Identifier", "nullable": True, "key": True, "description": "Parent signature for nested search."},
        {"name": "Path", "type": "AnyPath", "nullable": True, "key": False, "description": "Path to search."},
        {"name": "Depth", "type": "Integer", "nullable": True, "key": False, "description": "Subdirectory search depth."},
    ],
    "DuplicateFile": [
        {"name": "FileKey", "type": "Identifier", "nullable": False, "key": True, "description": "Duplicate file identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Component table."},
        {"name": "File_", "type": "Identifier", "nullable": False, "key": False, "description": "Source file reference."},
        {"name": "DestName", "type": "Filename", "nullable": True, "key": False, "description": "Destination filename."},
        {"name": "DestFolder", "type": "Identifier", "nullable": True, "key": False, "description": "Destination directory."},
    ],
    "Environment": [
        {"name": "Environment", "type": "Identifier", "nullable": False, "key": True, "description": "Environment variable identifier."},
        {"name": "Name", "type": "Text", "nullable": False, "key": False, "description": "Environment variable name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Environment variable value."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Component table."},
    ],
    "Error": [
        {"name": "Error", "type": "Integer", "nullable": False, "key": True, "description": "Error code number."},
        {"name": "Message", "type": "Template", "nullable": True, "key": False, "description": "Error message template."},
    ],
    "EventMapping": [
        {"name": "Dialog_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Dialog table."},
        {"name": "Control_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Control table."},
        {"name": "Event", "type": "Identifier", "nullable": False, "key": True, "description": "Event name."},
        {"name": "Attribute", "type": "Identifier", "nullable": False, "key": False, "description": "Control attribute to update."},
    ],
    "Extension": [
        {"name": "Extension", "type": "Text", "nullable": False, "key": True, "description": "File extension (without dot)."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "ProgId_", "type": "Text", "nullable": True, "key": False, "description": "Default ProgId for extension."},
        {"name": "MIME_", "type": "Text", "nullable": True, "key": False, "description": "MIME content type."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Feature table."},
    ],
    "Feature": [
        {"name": "Feature", "type": "Identifier", "nullable": False, "key": True, "description": "Feature identifier."},
        {"name": "Feature_Parent", "type": "Identifier", "nullable": True, "key": False, "description": "Parent feature."},
        {"name": "Title", "type": "Text", "nullable": True, "key": False, "description": "Feature title."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "Feature description."},
        {"name": "Display", "type": "Integer", "nullable": True, "key": False, "description": "Display order and state."},
        {"name": "Level", "type": "Integer", "nullable": False, "key": False, "description": "Installation level."},
        {"name": "Directory_", "type": "Identifier", "nullable": True, "key": False, "description": "Feature configuration directory."},
        {"name": "Attributes", "type": "Integer", "nullable": False, "key": False, "description": "Feature attributes."},
    ],
    "FeatureComponents": [
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Feature table."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
    ],
    "File": [
        {"name": "File", "type": "Identifier", "nullable": False, "key": True, "description": "File identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Component table."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "Short|Long filename."},
        {"name": "FileSize", "type": "Integer", "nullable": False, "key": False, "description": "File size in bytes."},
        {"name": "Version", "type": "Version", "nullable": True, "key": False, "description": "File version string."},
        {"name": "Language", "type": "Language", "nullable": True, "key": False, "description": "Language ID."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "File attributes."},
        {"name": "Sequence", "type": "Integer", "nullable": False, "key": False, "description": "Sequence in media."},
    ],
    "FileSFPCatalog": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to File table."},
        {"name": "SFPCatalog_", "type": "Filename", "nullable": False, "key": True, "description": "Reference to SFPCatalog table."},
    ],
    "Font": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to File table."},
        {"name": "FontTitle", "type": "Text", "nullable": True, "key": False, "description": "Font title for registration."},
    ],
    "Icon": [
        {"name": "Name", "type": "Identifier", "nullable": False, "key": True, "description": "Icon identifier."},
        {"name": "Data", "type": "Binary", "nullable": False, "key": False, "description": "Icon binary data."},
    ],
    "IniFile": [
        {"name": "IniFile", "type": "Identifier", "nullable": False, "key": True, "description": "INI file entry identifier."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "INI filename."},
        {"name": "DirProperty", "type": "Identifier", "nullable": True, "key": False, "description": "Directory property."},
        {"name": "Section", "type": "Formatted", "nullable": False, "key": False, "description": "Section name."},
        {"name": "Key", "type": "Formatted", "nullable": False, "key": False, "description": "Key name."},
        {"name": "Value", "type": "Formatted", "nullable": False, "key": False, "description": "Value to write."},
        {"name": "Action", "type": "Integer", "nullable": False, "key": False, "description": "Write action type."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Component table."},
    ],
    "IniLocator": [
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Signature table."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "INI filename to search."},
        {"name": "Section", "type": "Text", "nullable": False, "key": False, "description": "Section name."},
        {"name": "Key", "type": "Text", "nullable": False, "key": False, "description": "Key name."},
        {"name": "Field", "type": "Integer", "nullable": True, "key": False, "description": "Field number in value."},
        {"name": "Type", "type": "Integer", "nullable": True, "key": False, "description": "Search result type."},
    ],
    "InstallExecuteSequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Execution condition."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number."},
    ],
    "InstallUISequence": [
        {"name": "Action", "type": "Identifier", "nullable": False, "key": True, "description": "Name of the action."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Execution condition."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Sequence number."},
    ],
    "IsolatedComponent": [
        {"name": "Component_Shared", "type": "Identifier", "nullable": False, "key": True, "description": "Shared component reference."},
        {"name": "Component_Application", "type": "Identifier", "nullable": False, "key": True, "description": "Application component reference."},
    ],
    "LaunchCondition": [
        {"name": "Condition", "type": "Condition", "nullable": False, "key": True, "description": "Launch condition expression."},
        {"name": "Description", "type": "Formatted", "nullable": False, "key": False, "description": "Error message if condition fails."},
    ],
    "ListBox": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Order", "type": "Integer", "nullable": False, "key": True, "description": "Item display order."},
        {"name": "Value", "type": "Formatted", "nullable": False, "key": False, "description": "Item value."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Display text."},
    ],
    "ListView": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Order", "type": "Integer", "nullable": False, "key": True, "description": "Item display order."},
        {"name": "Value", "type": "Formatted", "nullable": False, "key": False, "description": "Item value."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Display text."},
        {"name": "Binary_", "type": "Identifier", "nullable": True, "key": False, "description": "Icon binary reference."},
    ],
    "LockPermissions": [
        {"name": "LockObject", "type": "Identifier", "nullable": False, "key": True, "description": "File, Registry, or Directory key."},
        {"name": "Table", "type": "Identifier", "nullable": False, "key": True, "description": "Source table name."},
        {"name": "Domain", "type": "Formatted", "nullable": True, "key": True, "description": "Domain name."},
        {"name": "User", "type": "Formatted", "nullable": False, "key": True, "description": "User or group name."},
        {"name": "Permission", "type": "Integer", "nullable": True, "key": False, "description": "Permission bitmask."},
    ],
    "MIME": [
        {"name": "ContentType", "type": "Text", "nullable": False, "key": True, "description": "MIME content type."},
        {"name": "Extension_", "type": "Text", "nullable": False, "key": False, "description": "File extension."},
        {"name": "CLSID", "type": "GUID", "nullable": True, "key": False, "description": "Associated CLSID."},
    ],
    "Media": [
        {"name": "DiskId", "type": "Integer", "nullable": False, "key": True, "description": "Disk identifier."},
        {"name": "LastSequence", "type": "Integer", "nullable": False, "key": False, "description": "Last file sequence on disk."},
        {"name": "DiskPrompt", "type": "Text", "nullable": True, "key": False, "description": "Disk prompt string."},
        {"name": "Cabinet", "type": "Cabinet", "nullable": True, "key": False, "description": "Cabinet filename."},
        {"name": "VolumeLabel", "type": "Text", "nullable": True, "key": False, "description": "Volume label."},
        {"name": "Source", "type": "Property", "nullable": True, "key": False, "description": "Source directory property."},
    ],
    "MoveFile": [
        {"name": "FileKey", "type": "Identifier", "nullable": False, "key": True, "description": "Move file identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Component table."},
        {"name": "SourceName", "type": "Filename", "nullable": True, "key": False, "description": "Source filename or wildcard."},
        {"name": "SourceFolder", "type": "Identifier", "nullable": True, "key": False, "description": "Source directory property."},
        {"name": "DestName", "type": "Filename", "nullable": True, "key": False, "description": "Destination filename."},
        {"name": "DestFolder", "type": "Identifier", "nullable": False, "key": False, "description": "Destination directory property."},
        {"name": "Options", "type": "Integer", "nullable": False, "key": False, "description": "Move or copy options."},
    ],
    "MsiAssembly": [
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Reference to Feature table."},
        {"name": "File_Manifest", "type": "Identifier", "nullable": True, "key": False, "description": "Manifest file reference."},
        {"name": "File_Application", "type": "Identifier", "nullable": True, "key": False, "description": "Application file for private assembly."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Assembly attributes."},
    ],
    "MsiAssemblyName": [
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "Name", "type": "Text", "nullable": False, "key": True, "description": "Assembly name part."},
        {"name": "Value", "type": "Text", "nullable": False, "key": False, "description": "Assembly name value."},
    ],
    "MsiDigitalCertificate": [
        {"name": "DigitalCertificate", "type": "Identifier", "nullable": False, "key": True, "description": "Certificate identifier."},
        {"name": "CertData", "type": "Binary", "nullable": False, "key": False, "description": "Certificate binary data."},
    ],
    "MsiDigitalSignature": [
        {"name": "Table", "type": "Identifier", "nullable": False, "key": True, "description": "Signed table name."},
        {"name": "SignObject", "type": "Text", "nullable": False, "key": True, "description": "Signed object key."},
        {"name": "DigitalCertificate_", "type": "Identifier", "nullable": False, "key": False, "description": "Certificate reference."},
        {"name": "Hash", "type": "Binary", "nullable": True, "key": False, "description": "Hash of signed content."},
    ],
    "MsiDriverPackages": [
        {"name": "Component", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to Component table."},
        {"name": "Flags", "type": "Integer", "nullable": False, "key": False, "description": "DIFx driver package flags."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Processing sequence."},
        {"name": "ReferenceComponents", "type": "Text", "nullable": True, "key": False, "description": "Dependent component list."},
    ],
    "MsiEmbeddedChainer": [
        {"name": "MsiEmbeddedChainer", "type": "Identifier", "nullable": False, "key": True, "description": "Chainer identifier."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Execution condition."},
        {"name": "CommandLine", "type": "Formatted", "nullable": True, "key": False, "description": "Command-line arguments."},
        {"name": "Source", "type": "CustomSource", "nullable": False, "key": False, "description": "Chainer source."},
        {"name": "Type", "type": "Integer", "nullable": False, "key": False, "description": "Chainer type."},
    ],
    "MsiEmbeddedUI": [
        {"name": "MsiEmbeddedUI", "type": "Identifier", "nullable": False, "key": True, "description": "Embedded UI identifier."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "UI handler DLL name."},
        {"name": "Attributes", "type": "Integer", "nullable": False, "key": False, "description": "UI attributes."},
        {"name": "MessageFilter", "type": "Integer", "nullable": True, "key": False, "description": "Message filter mask."},
        {"name": "Data", "type": "Binary", "nullable": False, "key": False, "description": "UI handler binary."},
    ],
    "MsiFileHash": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to File table."},
        {"name": "Options", "type": "Integer", "nullable": False, "key": False, "description": "Hash options (must be 0)."},
        {"name": "HashPart1", "type": "Integer", "nullable": False, "key": False, "description": "First part of MD5 hash."},
        {"name": "HashPart2", "type": "Integer", "nullable": False, "key": False, "description": "Second part of MD5 hash."},
        {"name": "HashPart3", "type": "Integer", "nullable": False, "key": False, "description": "Third part of MD5 hash."},
        {"name": "HashPart4", "type": "Integer", "nullable": False, "key": False, "description": "Fourth part of MD5 hash."},
    ],
    "MsiLockPermissionsEx": [
        {"name": "MsiLockPermissionsEx", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "LockObject", "type": "Identifier", "nullable": False, "key": False, "description": "Object to secure."},
        {"name": "Table", "type": "Identifier", "nullable": False, "key": False, "description": "Source table name."},
        {"name": "SDDLText", "type": "FormattedSDDL", "nullable": False, "key": False, "description": "SDDL security descriptor."},
        {"name": "Condition", "type": "Condition", "nullable": True, "key": False, "description": "Application condition."},
    ],
    "MsiPackageCertificate": [
        {"name": "PackageCertificate", "type": "Identifier", "nullable": False, "key": True, "description": "Certificate identifier."},
        {"name": "DigitalCertificate_", "type": "Identifier", "nullable": False, "key": False, "description": "Certificate reference."},
    ],
    "MsiPatchCertificate": [
        {"name": "PatchCertificate", "type": "Identifier", "nullable": False, "key": True, "description": "Patch certificate identifier."},
        {"name": "DigitalCertificate_", "type": "Identifier", "nullable": False, "key": False, "description": "Certificate reference."},
    ],
    "MsiPatchHeaders": [
        {"name": "StreamRef", "type": "Identifier", "nullable": False, "key": True, "description": "Stream reference."},
        {"name": "Header", "type": "Binary", "nullable": False, "key": False, "description": "Patch header data."},
    ],
    "MsiPatchMetadata": [
        {"name": "Company", "type": "Identifier", "nullable": True, "key": True, "description": "Company identifier."},
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Metadata property name."},
        {"name": "Value", "type": "Text", "nullable": True, "key": False, "description": "Metadata value."},
    ],
    "MsiPatchOldAssemblyFile": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "File reference."},
        {"name": "Assembly_", "type": "Identifier", "nullable": False, "key": True, "description": "Assembly reference."},
    ],
    "MsiPatchOldAssemblyName": [
        {"name": "Assembly", "type": "Identifier", "nullable": False, "key": True, "description": "Assembly identifier."},
        {"name": "Name", "type": "Text", "nullable": False, "key": True, "description": "Name attribute."},
        {"name": "Value", "type": "Text", "nullable": True, "key": False, "description": "Attribute value."},
    ],
    "MsiPatchSequence": [
        {"name": "PatchFamily", "type": "Identifier", "nullable": False, "key": True, "description": "Patch family identifier."},
        {"name": "ProductCode", "type": "GUID", "nullable": True, "key": True, "description": "Target product code."},
        {"name": "Sequence", "type": "Version", "nullable": False, "key": False, "description": "Patch sequence."},
        {"name": "Attributes", "type": "Integer", "nullable": True, "key": False, "description": "Supersedence attributes."},
    ],
    "MsiSFCBypass": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to File table."},
    ],
    "MsiServiceConfig": [
        {"name": "MsiServiceConfig", "type": "Identifier", "nullable": False, "key": True, "description": "Service config identifier."},
        {"name": "Name", "type": "Formatted", "nullable": False, "key": False, "description": "Service name."},
        {"name": "Event", "type": "Integer", "nullable": False, "key": False, "description": "Install/Uninstall event."},
        {"name": "ConfigType", "type": "Integer", "nullable": False, "key": False, "description": "Configuration type."},
        {"name": "Argument", "type": "Text", "nullable": True, "key": False, "description": "Configuration argument."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "MsiServiceConfigFailureActions": [
        {"name": "MsiServiceConfigFailureActions", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Name", "type": "Formatted", "nullable": False, "key": False, "description": "Service name."},
        {"name": "Event", "type": "Integer", "nullable": False, "key": False, "description": "Install/Uninstall event."},
        {"name": "ResetPeriod", "type": "Integer", "nullable": True, "key": False, "description": "Failure count reset period."},
        {"name": "RebootMessage", "type": "Formatted", "nullable": True, "key": False, "description": "Reboot message."},
        {"name": "Command", "type": "Formatted", "nullable": True, "key": False, "description": "Command to run on failure."},
        {"name": "Actions", "type": "Text", "nullable": True, "key": False, "description": "Failure action sequence."},
        {"name": "DelayActions", "type": "Text", "nullable": True, "key": False, "description": "Delay between actions."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "MsiShortcutProperty": [
        {"name": "MsiShortcutProperty", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Shortcut_", "type": "Identifier", "nullable": False, "key": False, "description": "Shortcut reference."},
        {"name": "PropertyKey", "type": "Formatted", "nullable": False, "key": False, "description": "Property key GUID."},
        {"name": "PropVariantValue", "type": "Formatted", "nullable": False, "key": False, "description": "Property value."},
    ],
    "ODBCAttribute": [
        {"name": "Driver_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to ODBCDriver table."},
        {"name": "Attribute", "type": "Text", "nullable": False, "key": True, "description": "Attribute name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Attribute value."},
    ],
    "ODBCDataSource": [
        {"name": "DataSource", "type": "Identifier", "nullable": False, "key": True, "description": "Data source identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "Description", "type": "Text", "nullable": False, "key": False, "description": "Data source description."},
        {"name": "DriverDescription", "type": "Text", "nullable": False, "key": False, "description": "Driver description."},
        {"name": "Registration", "type": "Integer", "nullable": False, "key": False, "description": "Registration type."},
    ],
    "ODBCDriver": [
        {"name": "Driver", "type": "Identifier", "nullable": False, "key": True, "description": "Driver identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "Description", "type": "Text", "nullable": False, "key": False, "description": "Driver description."},
        {"name": "File_", "type": "Identifier", "nullable": False, "key": False, "description": "Driver file reference."},
        {"name": "File_Setup", "type": "Identifier", "nullable": True, "key": False, "description": "Setup file reference."},
    ],
    "ODBCSourceAttribute": [
        {"name": "DataSource_", "type": "Identifier", "nullable": False, "key": True, "description": "Reference to ODBCDataSource table."},
        {"name": "Attribute", "type": "Text", "nullable": False, "key": True, "description": "Attribute name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Attribute value."},
    ],
    "ODBCTranslator": [
        {"name": "Translator", "type": "Identifier", "nullable": False, "key": True, "description": "Translator identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "Description", "type": "Text", "nullable": False, "key": False, "description": "Translator description."},
        {"name": "File_", "type": "Identifier", "nullable": False, "key": False, "description": "Translator file reference."},
        {"name": "File_Setup", "type": "Identifier", "nullable": True, "key": False, "description": "Setup file reference."},
    ],
    "Patch": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "File reference."},
        {"name": "Sequence", "type": "Integer", "nullable": False, "key": True, "description": "File sequence."},
        {"name": "PatchSize", "type": "Integer", "nullable": False, "key": False, "description": "Patch data size."},
        {"name": "Attributes", "type": "Integer", "nullable": False, "key": False, "description": "Patch attributes."},
        {"name": "Header", "type": "Binary", "nullable": True, "key": False, "description": "Patch header."},
        {"name": "StreamRef_", "type": "Identifier", "nullable": True, "key": False, "description": "Stream reference."},
    ],
    "PatchPackage": [
        {"name": "PatchId", "type": "GUID", "nullable": False, "key": True, "description": "Patch package GUID."},
        {"name": "Media_", "type": "Integer", "nullable": False, "key": False, "description": "Media table reference."},
    ],
    "ProgId": [
        {"name": "ProgId", "type": "Text", "nullable": False, "key": True, "description": "Programmatic identifier."},
        {"name": "ProgId_Parent", "type": "Text", "nullable": True, "key": False, "description": "Parent ProgId."},
        {"name": "Class_", "type": "GUID", "nullable": True, "key": False, "description": "CLSID reference."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "ProgId description."},
        {"name": "Icon_", "type": "Identifier", "nullable": True, "key": False, "description": "Icon reference."},
        {"name": "IconIndex", "type": "Integer", "nullable": True, "key": False, "description": "Icon index."},
    ],
    "Property": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Value", "type": "Text", "nullable": False, "key": False, "description": "Property value."},
    ],
    "PublishComponent": [
        {"name": "ComponentId", "type": "GUID", "nullable": False, "key": True, "description": "Published component GUID."},
        {"name": "Qualifier", "type": "Text", "nullable": False, "key": True, "description": "Qualifier string."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Component reference."},
        {"name": "AppData", "type": "Text", "nullable": True, "key": False, "description": "Application data."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Feature reference."},
    ],
    "RadioButton": [
        {"name": "Property", "type": "Identifier", "nullable": False, "key": True, "description": "Property name."},
        {"name": "Order", "type": "Integer", "nullable": False, "key": True, "description": "Button order."},
        {"name": "Value", "type": "Formatted", "nullable": False, "key": False, "description": "Button value."},
        {"name": "X", "type": "Integer", "nullable": False, "key": False, "description": "X coordinate."},
        {"name": "Y", "type": "Integer", "nullable": False, "key": False, "description": "Y coordinate."},
        {"name": "Width", "type": "Integer", "nullable": False, "key": False, "description": "Button width."},
        {"name": "Height", "type": "Integer", "nullable": False, "key": False, "description": "Button height."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Button text."},
        {"name": "Help", "type": "Text", "nullable": True, "key": False, "description": "Help text."},
    ],
    "RegLocator": [
        {"name": "Signature_", "type": "Identifier", "nullable": False, "key": True, "description": "Signature reference."},
        {"name": "Root", "type": "Integer", "nullable": False, "key": False, "description": "Registry root (0=HKCR, 1=HKCU, 2=HKLM, 3=HKU)."},
        {"name": "Key", "type": "RegPath", "nullable": False, "key": False, "description": "Registry key path."},
        {"name": "Name", "type": "Formatted", "nullable": True, "key": False, "description": "Value name."},
        {"name": "Type", "type": "Integer", "nullable": True, "key": False, "description": "Search type."},
    ],
    "Registry": [
        {"name": "Registry", "type": "Identifier", "nullable": False, "key": True, "description": "Registry entry identifier."},
        {"name": "Root", "type": "Integer", "nullable": False, "key": False, "description": "Registry root."},
        {"name": "Key", "type": "RegPath", "nullable": False, "key": False, "description": "Registry key path."},
        {"name": "Name", "type": "Formatted", "nullable": True, "key": False, "description": "Value name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Registry value."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "RemoveFile": [
        {"name": "FileKey", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "FileName", "type": "WildCardFilename", "nullable": True, "key": False, "description": "Filename or wildcard."},
        {"name": "DirProperty", "type": "Identifier", "nullable": False, "key": False, "description": "Directory property."},
        {"name": "InstallMode", "type": "Integer", "nullable": False, "key": False, "description": "When to remove (1=install, 2=uninstall, 3=both)."},
    ],
    "RemoveIniFile": [
        {"name": "RemoveIniFile", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "INI filename."},
        {"name": "DirProperty", "type": "Identifier", "nullable": True, "key": False, "description": "Directory property."},
        {"name": "Section", "type": "Formatted", "nullable": False, "key": False, "description": "Section name."},
        {"name": "Key", "type": "Formatted", "nullable": False, "key": False, "description": "Key name."},
        {"name": "Value", "type": "Formatted", "nullable": True, "key": False, "description": "Value to match."},
        {"name": "Action", "type": "Integer", "nullable": False, "key": False, "description": "Remove action type."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "RemoveRegistry": [
        {"name": "RemoveRegistry", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Root", "type": "Integer", "nullable": False, "key": False, "description": "Registry root."},
        {"name": "Key", "type": "RegPath", "nullable": False, "key": False, "description": "Registry key path."},
        {"name": "Name", "type": "Formatted", "nullable": True, "key": False, "description": "Value name (null = key)."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "ReserveCost": [
        {"name": "ReserveKey", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "ReserveFolder", "type": "Identifier", "nullable": True, "key": False, "description": "Directory property."},
        {"name": "ReserveLocal", "type": "Integer", "nullable": False, "key": False, "description": "Local install cost."},
        {"name": "ReserveSource", "type": "Integer", "nullable": False, "key": False, "description": "Source install cost."},
    ],
    "SFPCatalog": [
        {"name": "SFPCatalog", "type": "Filename", "nullable": False, "key": True, "description": "Catalog filename."},
        {"name": "Catalog", "type": "Binary", "nullable": False, "key": False, "description": "Catalog binary data."},
        {"name": "Dependency", "type": "Formatted", "nullable": True, "key": False, "description": "Catalog dependencies."},
    ],
    "SelfReg": [
        {"name": "File_", "type": "Identifier", "nullable": False, "key": True, "description": "File reference."},
        {"name": "Cost", "type": "Integer", "nullable": True, "key": False, "description": "Self-registration cost."},
    ],
    "ServiceControl": [
        {"name": "ServiceControl", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Name", "type": "Formatted", "nullable": False, "key": False, "description": "Service name."},
        {"name": "Event", "type": "Integer", "nullable": False, "key": False, "description": "Control events bitmask."},
        {"name": "Arguments", "type": "Formatted", "nullable": True, "key": False, "description": "Start arguments."},
        {"name": "Wait", "type": "Integer", "nullable": True, "key": False, "description": "Wait for completion."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
    ],
    "ServiceInstall": [
        {"name": "ServiceInstall", "type": "Identifier", "nullable": False, "key": True, "description": "Entry identifier."},
        {"name": "Name", "type": "Formatted", "nullable": False, "key": False, "description": "Service name."},
        {"name": "DisplayName", "type": "Formatted", "nullable": True, "key": False, "description": "Display name."},
        {"name": "ServiceType", "type": "Integer", "nullable": False, "key": False, "description": "Service type."},
        {"name": "StartType", "type": "Integer", "nullable": False, "key": False, "description": "Start type."},
        {"name": "ErrorControl", "type": "Integer", "nullable": False, "key": False, "description": "Error control."},
        {"name": "LoadOrderGroup", "type": "Formatted", "nullable": True, "key": False, "description": "Load order group."},
        {"name": "Dependencies", "type": "Formatted", "nullable": True, "key": False, "description": "Dependencies."},
        {"name": "StartName", "type": "Formatted", "nullable": True, "key": False, "description": "Account name."},
        {"name": "Password", "type": "Formatted", "nullable": True, "key": False, "description": "Account password."},
        {"name": "Arguments", "type": "Formatted", "nullable": True, "key": False, "description": "Start arguments."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "Description", "type": "Formatted", "nullable": True, "key": False, "description": "Service description."},
    ],
    "Shortcut": [
        {"name": "Shortcut", "type": "Identifier", "nullable": False, "key": True, "description": "Shortcut identifier."},
        {"name": "Directory_", "type": "Identifier", "nullable": False, "key": False, "description": "Shortcut directory."},
        {"name": "Name", "type": "Filename", "nullable": False, "key": False, "description": "Shortcut filename."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": False, "description": "Component reference."},
        {"name": "Target", "type": "Shortcut", "nullable": False, "key": False, "description": "Shortcut target."},
        {"name": "Arguments", "type": "Formatted", "nullable": True, "key": False, "description": "Command arguments."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "Shortcut description."},
        {"name": "Hotkey", "type": "Integer", "nullable": True, "key": False, "description": "Hotkey."},
        {"name": "Icon_", "type": "Identifier", "nullable": True, "key": False, "description": "Icon reference."},
        {"name": "IconIndex", "type": "Integer", "nullable": True, "key": False, "description": "Icon index."},
        {"name": "ShowCmd", "type": "Integer", "nullable": True, "key": False, "description": "Show command."},
        {"name": "WkDir", "type": "Identifier", "nullable": True, "key": False, "description": "Working directory."},
        {"name": "DisplayResourceDLL", "type": "Formatted", "nullable": True, "key": False, "description": "Resource DLL for display name."},
        {"name": "DisplayResourceId", "type": "Integer", "nullable": True, "key": False, "description": "Display name resource ID."},
        {"name": "DescriptionResourceDLL", "type": "Formatted", "nullable": True, "key": False, "description": "Resource DLL for description."},
        {"name": "DescriptionResourceId", "type": "Integer", "nullable": True, "key": False, "description": "Description resource ID."},
    ],
    "Signature": [
        {"name": "Signature", "type": "Identifier", "nullable": False, "key": True, "description": "Signature identifier."},
        {"name": "FileName", "type": "Filename", "nullable": False, "key": False, "description": "Filename to search."},
        {"name": "MinVersion", "type": "Text", "nullable": True, "key": False, "description": "Minimum version."},
        {"name": "MaxVersion", "type": "Text", "nullable": True, "key": False, "description": "Maximum version."},
        {"name": "MinSize", "type": "Integer", "nullable": True, "key": False, "description": "Minimum file size."},
        {"name": "MaxSize", "type": "Integer", "nullable": True, "key": False, "description": "Maximum file size."},
        {"name": "MinDate", "type": "Integer", "nullable": True, "key": False, "description": "Minimum modification date."},
        {"name": "MaxDate", "type": "Integer", "nullable": True, "key": False, "description": "Maximum modification date."},
        {"name": "Languages", "type": "Text", "nullable": True, "key": False, "description": "Language IDs."},
    ],
    "TextStyle": [
        {"name": "TextStyle", "type": "Identifier", "nullable": False, "key": True, "description": "Text style identifier."},
        {"name": "FaceName", "type": "Text", "nullable": False, "key": False, "description": "Font face name."},
        {"name": "Size", "type": "Integer", "nullable": False, "key": False, "description": "Font size in points."},
        {"name": "Color", "type": "Integer", "nullable": True, "key": False, "description": "Text color (RGB)."},
        {"name": "StyleBits", "type": "Integer", "nullable": True, "key": False, "description": "Bold, italic, etc."},
    ],
    "TypeLib": [
        {"name": "LibID", "type": "GUID", "nullable": False, "key": True, "description": "TypeLib GUID."},
        {"name": "Language", "type": "Integer", "nullable": False, "key": True, "description": "Language ID."},
        {"name": "Component_", "type": "Identifier", "nullable": False, "key": True, "description": "Component reference."},
        {"name": "Version", "type": "Integer", "nullable": True, "key": False, "description": "TypeLib version."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "TypeLib description."},
        {"name": "Directory_", "type": "Identifier", "nullable": True, "key": False, "description": "Help directory."},
        {"name": "Feature_", "type": "Identifier", "nullable": False, "key": False, "description": "Feature reference."},
        {"name": "Cost", "type": "Integer", "nullable": True, "key": False, "description": "Registration cost."},
    ],
    "UIText": [
        {"name": "Key", "type": "Identifier", "nullable": False, "key": True, "description": "UI text key."},
        {"name": "Text", "type": "Text", "nullable": True, "key": False, "description": "Localized text."},
    ],
    "Upgrade": [
        {"name": "UpgradeCode", "type": "GUID", "nullable": False, "key": True, "description": "UpgradeCode GUID."},
        {"name": "VersionMin", "type": "Text", "nullable": True, "key": True, "description": "Minimum version."},
        {"name": "VersionMax", "type": "Text", "nullable": True, "key": True, "description": "Maximum version."},
        {"name": "Language", "type": "Text", "nullable": True, "key": True, "description": "Language constraint."},
        {"name": "Attributes", "type": "Integer", "nullable": False, "key": True, "description": "Upgrade attributes."},
        {"name": "Remove", "type": "Formatted", "nullable": True, "key": False, "description": "Features to remove."},
        {"name": "ActionProperty", "type": "Identifier", "nullable": False, "key": False, "description": "Property to receive product codes."},
    ],
    "Verb": [
        {"name": "Extension_", "type": "Text", "nullable": False, "key": True, "description": "Extension reference."},
        {"name": "Verb", "type": "Text", "nullable": False, "key": True, "description": "Verb name (open, edit, print)."},
        {"name": "Sequence", "type": "Integer", "nullable": True, "key": False, "description": "Verb order."},
        {"name": "Command", "type": "Formatted", "nullable": True, "key": False, "description": "Menu text."},
        {"name": "Argument", "type": "Formatted", "nullable": True, "key": False, "description": "Command arguments."},
    ],
    "_Columns": [
        {"name": "Table", "type": "Identifier", "nullable": False, "key": True, "description": "Table name."},
        {"name": "Number", "type": "Integer", "nullable": False, "key": True, "description": "Column number."},
        {"name": "Name", "type": "Identifier", "nullable": False, "key": False, "description": "Column name."},
        {"name": "Type", "type": "Text", "nullable": False, "key": False, "description": "Column type."},
    ],
    "_Storages": [
        {"name": "Name", "type": "Identifier", "nullable": False, "key": True, "description": "Storage name."},
        {"name": "Data", "type": "Binary", "nullable": True, "key": False, "description": "Storage data."},
    ],
    "_Streams": [
        {"name": "Name", "type": "Identifier", "nullable": False, "key": True, "description": "Stream name."},
        {"name": "Data", "type": "Binary", "nullable": True, "key": False, "description": "Stream data."},
    ],
    "_Tables": [
        {"name": "Name", "type": "Identifier", "nullable": False, "key": True, "description": "Table name."},
    ],
    "_TransformView": [
        {"name": "Table", "type": "Identifier", "nullable": True, "key": True, "description": "Table name."},
        {"name": "Column", "type": "Text", "nullable": True, "key": True, "description": "Column name."},
        {"name": "Row", "type": "Text", "nullable": True, "key": True, "description": "Row identifier."},
        {"name": "Data", "type": "Text", "nullable": True, "key": False, "description": "Data value."},
        {"name": "Current", "type": "Text", "nullable": True, "key": False, "description": "Current value."},
    ],
    "_Validation": [
        {"name": "Table", "type": "Identifier", "nullable": False, "key": True, "description": "Table name."},
        {"name": "Column", "type": "Identifier", "nullable": False, "key": True, "description": "Column name."},
        {"name": "Nullable", "type": "Text", "nullable": False, "key": False, "description": "Is nullable (Y/N)."},
        {"name": "MinValue", "type": "Integer", "nullable": True, "key": False, "description": "Minimum value."},
        {"name": "MaxValue", "type": "Integer", "nullable": True, "key": False, "description": "Maximum value."},
        {"name": "KeyTable", "type": "Identifier", "nullable": True, "key": False, "description": "Foreign key table."},
        {"name": "KeyColumn", "type": "Integer", "nullable": True, "key": False, "description": "Foreign key column."},
        {"name": "Category", "type": "Text", "nullable": True, "key": False, "description": "Column category."},
        {"name": "Set", "type": "Text", "nullable": True, "key": False, "description": "Valid value set."},
        {"name": "Description", "type": "Text", "nullable": True, "key": False, "description": "Column description."},
    ],
}

# Add missing tables that are duplicated entries (cleanup)
TABLES_TO_SKIP = ["MsiLockPermissionsEx Table", "_TransformView Table"]


def main():
    """Add column definitions to MSI tables."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    updated = 0
    for table_name, columns in MSI_TABLE_COLUMNS.items():
        columns_json = json.dumps(columns)
        cursor.execute("""
            UPDATE msi_tables
            SET columns = ?
            WHERE name = ?
        """, (columns_json, table_name))
        if cursor.rowcount > 0:
            updated += 1

    conn.commit()

    # Clean up duplicate/invalid table entries
    for table in TABLES_TO_SKIP:
        cursor.execute("DELETE FROM msi_tables WHERE name = ?", (table,))
        if cursor.rowcount > 0:
            print(f"Removed invalid entry: {table}")

    conn.commit()

    # Verify
    cursor.execute("""
        SELECT COUNT(*) FROM msi_tables
        WHERE columns IS NULL OR columns = '' OR columns = '[]'
    """)
    still_missing = cursor.fetchone()[0]

    cursor.execute("SELECT COUNT(*) FROM msi_tables")
    total = cursor.fetchone()[0]

    print(f"Updated {updated} MSI table column definitions")
    print(f"Tables with columns: {total - still_missing}/{total}")

    if still_missing > 0:
        cursor.execute("""
            SELECT name FROM msi_tables
            WHERE columns IS NULL OR columns = '' OR columns = '[]'
            ORDER BY name
        """)
        missing = [row[0] for row in cursor.fetchall()]
        print(f"Still missing columns: {missing}")

    conn.close()


if __name__ == "__main__":
    main()
