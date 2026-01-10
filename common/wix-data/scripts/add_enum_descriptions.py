#!/usr/bin/env python3
"""Add descriptions to attribute enum values that are missing them."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Descriptions for enum values organized by element/attribute
ENUM_DESCRIPTIONS = {
    # BootstrapperApplicationDll
    ("BootstrapperApplicationDll", "DpiAwareness", "unaware"): "Application is not DPI aware; Windows scales content.",

    # BundleCustomData
    ("BundleCustomData", "Type", "BootstrapperApplication"): "Data passed to the bootstrapper application.",
    ("BundleCustomData", "Type", "BootstrapperExtension"): "Data passed to a bootstrapper extension.",
    ("BundleCustomData", "Type", "BundleExtension"): "Data passed to a bundle extension.",

    # Certificate
    ("Certificate", "StoreLocation", "currentUser"): "Certificate store for the current user (HKCU).",
    ("Certificate", "StoreLocation", "localMachine"): "Certificate store for the local machine (HKLM).",
    ("Certificate", "StoreLocation", "localMachineEnterprise"): "Enterprise trust store for local machine.",
    ("Certificate", "StoreLocation", "localMachinePolicy"): "Group Policy store for local machine.",
    ("Certificate", "StoreLocation", "services"): "Certificate store for services.",
    ("Certificate", "StoreLocation", "userPolicy"): "Group Policy store for current user.",
    ("Certificate", "StoreLocation", "users"): "Certificate store for all users.",
    ("Certificate", "StoreName", "request"): "Certificate request store for pending requests.",

    # Class
    ("Class", "ThreadingModel", "apartment"): "Single-threaded apartment (STA) model.",
    ("Class", "ThreadingModel", "both"): "Supports both STA and MTA.",
    ("Class", "ThreadingModel", "free"): "Multi-threaded apartment (MTA) model.",
    ("Class", "ThreadingModel", "neutral"): "Neutral threading model; no thread affinity.",
    ("Class", "ThreadingModel", "rental"): "Rental threading model for pooled objects.",
    ("Class", "ThreadingModel", "single"): "Single-threaded; all calls on main thread.",

    # Column
    ("Column", "Category", "anyPath"): "Any valid file system path.",
    ("Column", "Category", "binary"): "Binary stream data.",
    ("Column", "Category", "cabinet"): "Cabinet file name.",
    ("Column", "Category", "condition"): "Conditional expression string.",
    ("Column", "Category", "customSource"): "Custom action source reference.",
    ("Column", "Category", "defaultDir"): "Default directory path in installer format.",
    ("Column", "Category", "doubleInteger"): "32-bit signed integer value.",
    ("Column", "Category", "filename"): "File name with optional short name.",
    ("Column", "Category", "formatted"): "Formatted string with property references.",
    ("Column", "Category", "formattedSddl"): "Formatted SDDL security descriptor.",
    ("Column", "Category", "guid"): "Globally unique identifier (GUID).",
    ("Column", "Category", "identifier"): "MSI identifier (up to 72 characters).",
    ("Column", "Category", "integer"): "16-bit signed integer value.",
    ("Column", "Category", "language"): "Language ID (LCID).",
    ("Column", "Category", "lowerCase"): "Lowercase text string.",
    ("Column", "Category", "path"): "File system path.",
    ("Column", "Category", "paths"): "Semicolon-separated list of paths.",
    ("Column", "Category", "property"): "Property name reference.",
    ("Column", "Category", "regPath"): "Registry key path.",
    ("Column", "Category", "shortcut"): "Shortcut target reference.",
    ("Column", "Category", "template"): "Summary Information template string.",
    ("Column", "Category", "text"): "Arbitrary text string.",
    ("Column", "Category", "timeDate"): "Date/time value.",
    ("Column", "Category", "upperCase"): "Uppercase text string.",
    ("Column", "Category", "version"): "Version string (major.minor.build.revision).",
    ("Column", "Category", "wildCardFilename"): "Filename pattern with wildcards.",

    # ComPlusApplication
    ("ComPlusApplication", "AccessChecksLevel", "applicationComponentLevel"): "Access checks at both application and component level.",
    ("ComPlusApplication", "AccessChecksLevel", "applicationLevel"): "Access checks only at application level.",
    ("ComPlusApplication", "Activation", "inproc"): "In-process activation (library application).",
    ("ComPlusApplication", "Activation", "local"): "Local server activation (server application).",
    ("ComPlusApplication", "Authentication", "call"): "Authenticate each call.",
    ("ComPlusApplication", "Authentication", "connect"): "Authenticate at connection time.",
    ("ComPlusApplication", "Authentication", "default"): "Use default authentication level.",
    ("ComPlusApplication", "Authentication", "integrity"): "Authenticate and verify data integrity.",
    ("ComPlusApplication", "Authentication", "none"): "No authentication required.",
    ("ComPlusApplication", "Authentication", "packet"): "Authenticate each packet.",
    ("ComPlusApplication", "Authentication", "privacy"): "Authenticate and encrypt data.",
    ("ComPlusApplication", "AuthenticationCapability", "dynamicCloaking"): "Dynamic identity cloaking.",
    ("ComPlusApplication", "AuthenticationCapability", "none"): "No authentication capabilities.",
    ("ComPlusApplication", "AuthenticationCapability", "secureReference"): "Secure reference tracking.",
    ("ComPlusApplication", "AuthenticationCapability", "staticCloaking"): "Static identity cloaking.",
    ("ComPlusApplication", "ImpersonationLevel", "anonymous"): "Client identity is anonymous.",
    ("ComPlusApplication", "ImpersonationLevel", "delegate"): "Server can delegate client identity.",
    ("ComPlusApplication", "ImpersonationLevel", "identify"): "Server can identify client but not impersonate.",
    ("ComPlusApplication", "ImpersonationLevel", "impersonate"): "Server can impersonate client locally.",
    ("ComPlusApplication", "QCAuthenticateMsgs", "off"): "No message authentication.",
    ("ComPlusApplication", "QCAuthenticateMsgs", "on"): "Always authenticate messages.",
    ("ComPlusApplication", "QCAuthenticateMsgs", "secureApps"): "Authenticate only for secure applications.",
    ("ComPlusApplication", "SRPTrustLevel", "disallowed"): "Software is not trusted to run.",
    ("ComPlusApplication", "SRPTrustLevel", "fullyTrusted"): "Software is fully trusted.",

    # ComPlusAssembly
    ("ComPlusAssembly", "Type", ".net"): ".NET managed assembly.",
    ("ComPlusAssembly", "Type", "native"): "Native COM component.",

    # ComPlusComponent
    ("ComPlusComponent", "Synchronization", "ignored"): "Synchronization attribute is ignored.",
    ("ComPlusComponent", "Synchronization", "none"): "No synchronization support.",
    ("ComPlusComponent", "Synchronization", "required"): "Component requires synchronization.",
    ("ComPlusComponent", "Synchronization", "requiresNew"): "Component requires new synchronization domain.",
    ("ComPlusComponent", "Synchronization", "supported"): "Synchronization is supported but not required.",
    ("ComPlusComponent", "Transaction", "ignored"): "Transaction attribute is ignored.",
    ("ComPlusComponent", "Transaction", "none"): "Component does not use transactions.",
    ("ComPlusComponent", "Transaction", "required"): "Component requires a transaction.",
    ("ComPlusComponent", "Transaction", "requiresNew"): "Component requires a new transaction.",
    ("ComPlusComponent", "Transaction", "supported"): "Transaction is supported but not required.",
    ("ComPlusComponent", "TxIsolationLevel", "any"): "Any isolation level is acceptable.",
    ("ComPlusComponent", "TxIsolationLevel", "readCommitted"): "Read only committed data.",
    ("ComPlusComponent", "TxIsolationLevel", "readUnCommitted"): "Read uncommitted data allowed.",
    ("ComPlusComponent", "TxIsolationLevel", "repeatableRead"): "Repeatable reads guaranteed.",
    ("ComPlusComponent", "TxIsolationLevel", "serializable"): "Fully serializable transactions.",

    # Configuration
    ("Configuration", "Format", "Bitfield"): "Bitfield value with named flags.",
    ("Configuration", "Format", "Integer"): "Integer numeric value.",
    ("Configuration", "Format", "Key"): "Key reference to another table.",
    ("Configuration", "Format", "Text"): "Text string value.",

    # Control
    ("Control", "IconSize", "16"): "16x16 pixel icon (small).",
    ("Control", "IconSize", "32"): "32x32 pixel icon (standard).",
    ("Control", "IconSize", "48"): "48x48 pixel icon (large).",

    # CustomAction
    ("CustomAction", "Script", "jscript"): "JScript (JavaScript) script.",
    ("CustomAction", "Script", "vbscript"): "VBScript script.",

    # DotNetCompatibilityCheck
    ("DotNetCompatibilityCheck", "RollForward", "disable"): "Disable roll-forward; exact version match required.",
    ("DotNetCompatibilityCheck", "RollForward", "latestMajor"): "Roll forward to latest major version.",
    ("DotNetCompatibilityCheck", "RollForward", "latestMinor"): "Roll forward to latest minor version.",
    ("DotNetCompatibilityCheck", "RollForward", "latestPatch"): "Roll forward to latest patch version.",
    ("DotNetCompatibilityCheck", "RollForward", "major"): "Roll forward to next major version if needed.",
    ("DotNetCompatibilityCheck", "RollForward", "minor"): "Roll forward to next minor version if needed.",
    ("DotNetCompatibilityCheck", "RuntimeType", "aspnet"): "ASP.NET Core runtime.",
    ("DotNetCompatibilityCheck", "RuntimeType", "core"): ".NET Core runtime.",
    ("DotNetCompatibilityCheck", "RuntimeType", "desktop"): ".NET Desktop runtime (WPF/WinForms).",

    # MessageQueue
    ("MessageQueue", "PrivLevel", "body"): "Encrypt message body only.",
    ("MessageQueue", "PrivLevel", "none"): "No message encryption.",
    ("MessageQueue", "PrivLevel", "optional"): "Encryption optional; accepts both.",

    # RelatedBundle
    ("RelatedBundle", "Action", "addon"): "Related bundle is an addon to this bundle.",
    ("RelatedBundle", "Action", "detect"): "Only detect the related bundle.",
    ("RelatedBundle", "Action", "patch"): "Related bundle is a patch.",
    ("RelatedBundle", "Action", "upgrade"): "Related bundle upgrades this bundle.",

    # RemoteRelatedBundle
    ("RemoteRelatedBundle", "Action", "addon"): "Remote bundle is an addon.",
    ("RemoteRelatedBundle", "Action", "detect"): "Only detect the remote bundle.",
    ("RemoteRelatedBundle", "Action", "patch"): "Remote bundle is a patch.",
    ("RemoteRelatedBundle", "Action", "upgrade"): "Remote bundle provides upgrade.",

    # ServiceConfig
    ("ServiceConfig", "FirstFailureActionType", "none"): "No action on first failure.",
    ("ServiceConfig", "FirstFailureActionType", "reboot"): "Reboot system on first failure.",
    ("ServiceConfig", "FirstFailureActionType", "restart"): "Restart service on first failure.",
    ("ServiceConfig", "FirstFailureActionType", "runCommand"): "Run command on first failure.",
    ("ServiceConfig", "SecondFailureActionType", "none"): "No action on second failure.",
    ("ServiceConfig", "SecondFailureActionType", "reboot"): "Reboot system on second failure.",
    ("ServiceConfig", "SecondFailureActionType", "restart"): "Restart service on second failure.",
    ("ServiceConfig", "SecondFailureActionType", "runCommand"): "Run command on second failure.",
    ("ServiceConfig", "ThirdFailureActionType", "none"): "No action on third failure.",
    ("ServiceConfig", "ThirdFailureActionType", "reboot"): "Reboot system on third failure.",
    ("ServiceConfig", "ThirdFailureActionType", "restart"): "Restart service on third failure.",
    ("ServiceConfig", "ThirdFailureActionType", "runCommand"): "Run command on third failure.",

    # SniSslCertificate
    ("SniSslCertificate", "HandleExisting", "fail"): "Fail if binding already exists.",
    ("SniSslCertificate", "HandleExisting", "ignore"): "Ignore if binding already exists.",
    ("SniSslCertificate", "HandleExisting", "replace"): "Replace existing binding.",

    # UrlAce
    ("UrlAce", "Rights", "all"): "All URL rights (register and delegate).",
    ("UrlAce", "Rights", "delegate"): "Can delegate URL to others.",
    ("UrlAce", "Rights", "register"): "Can register to listen on URL.",

    # WebAppPool
    ("WebAppPool", "CpuAction", "none"): "No action when CPU limit exceeded.",
    ("WebAppPool", "CpuAction", "shutdown"): "Shut down worker process when CPU limit exceeded.",
    ("WebAppPool", "Identity", "applicationPoolIdentity"): "Built-in ApplicationPoolIdentity account.",
    ("WebAppPool", "Identity", "localService"): "LocalService account (limited privileges).",
    ("WebAppPool", "Identity", "localSystem"): "LocalSystem account (full privileges).",
    ("WebAppPool", "Identity", "networkService"): "NetworkService account (network access).",
    ("WebAppPool", "Identity", "other"): "Custom user account.",

    # WebApplication
    ("WebApplication", "DefaultScript", "JScript"): "JScript as default scripting language.",
    ("WebApplication", "DefaultScript", "VBScript"): "VBScript as default scripting language.",

    # WebProperty
    ("WebProperty", "Id", "ETagChangeNumber"): "ETag change number for cache validation.",
    ("WebProperty", "Id", "IIs5IsolationMode"): "IIS 5 isolation mode compatibility.",
    ("WebProperty", "Id", "LogInUTF8"): "Log files in UTF-8 encoding.",
    ("WebProperty", "Id", "MaxGlobalBandwidth"): "Maximum global bandwidth limit.",

    # WixDotNetCoreBootstrapperApplicationHost
    ("WixDotNetCoreBootstrapperApplicationHost", "Theme", "none"): "No built-in theme; custom UI only.",
    ("WixDotNetCoreBootstrapperApplicationHost", "Theme", "standard"): "Standard WiX bootstrapper theme.",

    # WixInternalUIBootstrapperApplication
    ("WixInternalUIBootstrapperApplication", "Theme", "none"): "No theme; MSI internal UI only.",
    ("WixInternalUIBootstrapperApplication", "Theme", "standard"): "Standard progress theme.",

    # WixManagedBootstrapperApplicationHost
    ("WixManagedBootstrapperApplicationHost", "Theme", "none"): "No built-in theme; custom managed UI.",
    ("WixManagedBootstrapperApplicationHost", "Theme", "standard"): "Standard WiX managed BA theme.",

    # WixPrerequisiteBootstrapperApplication
    ("WixPrerequisiteBootstrapperApplication", "Theme", "none"): "No theme; silent prerequisite check.",
    ("WixPrerequisiteBootstrapperApplication", "Theme", "standard"): "Standard prerequisite theme.",

    # WixStandardBootstrapperApplication
    ("WixStandardBootstrapperApplication", "Theme", "hyperlinkLargeLicense"): "Large license with clickable hyperlink.",
    ("WixStandardBootstrapperApplication", "Theme", "hyperlinkLicense"): "License with clickable hyperlink.",
    ("WixStandardBootstrapperApplication", "Theme", "hyperlinkSidebarLicense"): "Sidebar layout with license hyperlink.",
    ("WixStandardBootstrapperApplication", "Theme", "none"): "No built-in UI; silent install.",
    ("WixStandardBootstrapperApplication", "Theme", "rtfLargeLicense"): "Large RTF license display.",
    ("WixStandardBootstrapperApplication", "Theme", "rtfLicense"): "Standard RTF license display.",

    # XmlConfig
    ("XmlConfig", "Action", "create"): "Create the XML element or attribute.",
    ("XmlConfig", "Action", "delete"): "Delete the XML element or attribute.",
    ("XmlConfig", "Node", "document"): "Apply to the document node.",
    ("XmlConfig", "Node", "element"): "Apply to an element node.",
    ("XmlConfig", "Node", "value"): "Apply to the element value.",
    ("XmlConfig", "On", "install"): "Apply during installation.",
    ("XmlConfig", "On", "uninstall"): "Apply during uninstallation.",

    # XmlFile
    ("XmlFile", "SelectionLanguage", "XPath"): "Use XPath for node selection.",
    ("XmlFile", "SelectionLanguage", "XSLPattern"): "Use XSL Pattern for node selection.",
}


def main():
    """Update enum values with descriptions."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    updated = 0
    for (element, attribute, value), description in ENUM_DESCRIPTIONS.items():
        cursor.execute("""
            UPDATE attribute_enum_values
            SET description = ?
            WHERE id IN (
                SELECT ev.id FROM attribute_enum_values ev
                JOIN attributes a ON ev.attribute_id = a.id
                JOIN elements e ON a.element_id = e.id
                WHERE e.name = ? AND a.name = ? AND ev.value = ?
            )
        """, (description, element, attribute, value))
        if cursor.rowcount > 0:
            updated += 1

    conn.commit()

    # Verify
    cursor.execute("""
        SELECT COUNT(*) FROM attribute_enum_values
        WHERE description IS NULL OR description = ''
    """)
    still_missing = cursor.fetchone()[0]

    print(f"Updated {updated} enum value descriptions")
    print(f"Still missing descriptions: {still_missing}")

    conn.close()


if __name__ == "__main__":
    main()
