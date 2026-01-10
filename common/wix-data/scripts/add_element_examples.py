#!/usr/bin/env python3
"""Add examples to all elements in the database."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Examples organized by namespace
ELEMENT_EXAMPLES = {
    # ===================
    # BAL (Bootstrapper Application Library) namespace
    # ===================
    ("bal", "BootstrapperApplicationPrerequisiteInformation"): '''<bal:BootstrapperApplicationPrerequisiteInformation
    PackageId="NetFx48Redist"
    LicenseUrl="https://go.microsoft.com/fwlink/?LinkId=2009380" />''',

    ("bal", "Condition"): '''<bal:Condition Message="This bundle requires .NET 4.8 or later.">
    NETFRAMEWORK48
</bal:Condition>''',

    ("bal", "ManagedBootstrapperApplicationPrereqInformation"): '''<bal:ManagedBootstrapperApplicationPrereqInformation
    PackageId="NetFx48Redist" />''',

    ("bal", "WixDotNetCoreBootstrapperApplicationHost"): '''<bal:WixDotNetCoreBootstrapperApplicationHost
    Theme="standard"
    LicenseFile="License.rtf"
    LogoFile="logo.png" />''',

    ("bal", "WixInternalUIBootstrapperApplication"): '''<bal:WixInternalUIBootstrapperApplication
    Theme="standard"
    LogoFile="logo.png" />''',

    ("bal", "WixManagedBootstrapperApplicationHost"): '''<bal:WixManagedBootstrapperApplicationHost
    Theme="standard"
    LicenseFile="License.rtf" />''',

    ("bal", "WixPrerequisiteBootstrapperApplication"): '''<bal:WixPrerequisiteBootstrapperApplication
    Theme="standard"
    LicenseFile="License.rtf"
    LogoFile="logo.png" />''',

    ("bal", "WixStandardBootstrapperApplication"): '''<bal:WixStandardBootstrapperApplication
    Theme="hyperlinkLicense"
    LicenseUrl="https://example.com/license"
    LogoFile="logo.png"
    ShowVersion="yes" />''',

    # ===================
    # COM+ namespace
    # ===================
    ("complus", "ComPlusApplication"): '''<complus:ComPlusApplication Id="MyApp"
    Name="My COM+ Application"
    Activation="local"
    Description="Sample COM+ application" />''',

    ("complus", "ComPlusApplicationRole"): '''<complus:ComPlusApplicationRole Id="AdminRole"
    Application="MyApp"
    Name="Administrators"
    Description="Administrative access" />''',

    ("complus", "ComPlusAssembly"): '''<complus:ComPlusAssembly Id="MyAssembly"
    Application="MyApp"
    Type=".net"
    DllPath="[#MyDll]" />''',

    ("complus", "ComPlusAssemblyDependency"): '''<complus:ComPlusAssemblyDependency
    RequiredAssembly="BaseAssembly" />''',

    ("complus", "ComPlusComponent"): '''<complus:ComPlusComponent Id="MyComponent"
    Assembly="MyAssembly"
    CLSID="{GUID}" />''',

    ("complus", "ComPlusGroupInApplicationRole"): '''<complus:ComPlusGroupInApplicationRole
    ApplicationRole="AdminRole"
    Group="BUILTIN\\Administrators" />''',

    ("complus", "ComPlusGroupInPartitionRole"): '''<complus:ComPlusGroupInPartitionRole
    PartitionRole="PartRole1"
    Group="DOMAIN\\Developers" />''',

    ("complus", "ComPlusInterface"): '''<complus:ComPlusInterface Id="IMyInterface"
    Component="MyComponent"
    IID="{GUID}" />''',

    ("complus", "ComPlusMethod"): '''<complus:ComPlusMethod Id="MyMethod"
    Interface="IMyInterface"
    Index="0"
    Name="DoWork" />''',

    ("complus", "ComPlusPartition"): '''<complus:ComPlusPartition Id="MyPartition"
    Name="Application Partition"
    Description="Partition for application isolation" />''',

    ("complus", "ComPlusPartitionRole"): '''<complus:ComPlusPartitionRole Id="PartRole1"
    Partition="MyPartition"
    Name="PartitionAdmins" />''',

    ("complus", "ComPlusPartitionUser"): '''<complus:ComPlusPartitionUser
    Partition="MyPartition"
    User="DOMAIN\\AppUser" />''',

    ("complus", "ComPlusRoleForComponent"): '''<complus:ComPlusRoleForComponent
    Component="MyComponent"
    ApplicationRole="AdminRole" />''',

    ("complus", "ComPlusRoleForInterface"): '''<complus:ComPlusRoleForInterface
    Interface="IMyInterface"
    ApplicationRole="AdminRole" />''',

    ("complus", "ComPlusRoleForMethod"): '''<complus:ComPlusRoleForMethod
    Method="MyMethod"
    ApplicationRole="AdminRole" />''',

    ("complus", "ComPlusSubscription"): '''<complus:ComPlusSubscription Id="EventSub"
    Component="MyComponent"
    EventCLSID="{GUID}"
    SubscriptionId="{GUID}" />''',

    ("complus", "ComPlusUserInApplicationRole"): '''<complus:ComPlusUserInApplicationRole
    ApplicationRole="AdminRole"
    User="DOMAIN\\AppAdmin" />''',

    ("complus", "ComPlusUserInPartitionRole"): '''<complus:ComPlusUserInPartitionRole
    PartitionRole="PartRole1"
    User="DOMAIN\\PartAdmin" />''',

    # ===================
    # DirectX namespace
    # ===================
    ("directx", "GetCapabilities"): '''<directx:GetCapabilities />''',

    # ===================
    # Firewall namespace
    # ===================
    ("firewall", "FirewallException"): '''<firewall:FirewallException Id="MyAppFirewall"
    Name="My Application"
    Program="[#MyApp.exe]"
    Protocol="tcp"
    Port="8080"
    Scope="any" />''',

    ("firewall", "Interface"): '''<firewall:Interface Name="Ethernet" />''',

    ("firewall", "InterfaceType"): '''<firewall:InterfaceType Value="lan" />''',

    ("firewall", "LocalAddress"): '''<firewall:LocalAddress Value="192.168.1.0/24" />''',

    ("firewall", "RemoteAddress"): '''<firewall:RemoteAddress Value="10.0.0.0/8" />''',

    # ===================
    # HTTP namespace
    # ===================
    ("http", "SniSslCertificate"): '''<http:SniSslCertificate
    Host="myapp.example.com"
    Port="443"
    HandleExisting="replace"
    Store="MY"
    Thumbprint="[CERT_THUMBPRINT]" />''',

    ("http", "UrlAce"): '''<http:UrlAce
    Id="MyUrlAce"
    SecurityPrincipal="NT AUTHORITY\\NETWORK SERVICE"
    Rights="all" />''',

    ("http", "UrlReservation"): '''<http:UrlReservation
    Id="MyUrlReservation"
    Url="http://+:8080/myapp/"
    HandleExisting="replace">
    <http:UrlAce SecurityPrincipal="BUILTIN\\Users" Rights="all" />
</http:UrlReservation>''',

    # ===================
    # IIS namespace
    # ===================
    ("iis", "Certificate"): '''<iis:Certificate Id="MyCert"
    Name="My SSL Certificate"
    StoreLocation="localMachine"
    StoreName="MY"
    CertificatePath="cert.pfx"
    PFXPassword="[CERT_PASSWORD]" />''',

    ("iis", "CertificateRef"): '''<iis:CertificateRef Id="MyCert" />''',

    ("iis", "HttpHeader"): '''<iis:HttpHeader Name="X-Frame-Options" Value="SAMEORIGIN" />''',

    ("iis", "MimeMap"): '''<iis:MimeMap Id="JsonMime"
    Type="application/json"
    Extension=".json" />''',

    ("iis", "RecycleTime"): '''<iis:RecycleTime Value="03:00:00" />''',

    ("iis", "WebAddress"): '''<iis:WebAddress Id="AllHttp"
    Port="80"
    IP="*" />''',

    ("iis", "WebAppPool"): '''<iis:WebAppPool Id="MyAppPool"
    Name="MyApplication Pool"
    Identity="applicationPoolIdentity"
    ManagedRuntimeVersion="v4.0"
    ManagedPipelineMode="Integrated" />''',

    ("iis", "WebApplication"): '''<iis:WebApplication Id="MyWebApp"
    Name="myapp"
    WebAppPool="MyAppPool" />''',

    ("iis", "WebApplicationExtension"): '''<iis:WebApplicationExtension
    Executable="[ASPNET_ISAPI]"
    Extension="aspx"
    Verbs="GET,HEAD,POST" />''',

    ("iis", "WebDir"): '''<iis:WebDir Id="ImagesDir"
    Path="images"
    DirProperties="DefaultDirProperties" />''',

    ("iis", "WebDirProperties"): '''<iis:WebDirProperties Id="DefaultDirProperties"
    Read="yes"
    Script="yes"
    Execute="no" />''',

    ("iis", "WebError"): '''<iis:WebError ErrorCode="404"
    SubCode="0"
    File="[INSTALLDIR]errors\\404.html" />''',

    ("iis", "WebFilter"): '''<iis:WebFilter Id="MyFilter"
    Name="My ISAPI Filter"
    Path="[#filter.dll]"
    LoadOrder="1" />''',

    ("iis", "WebLog"): '''<iis:WebLog Id="MyWebLog"
    Type="IIS" />''',

    ("iis", "WebProperty"): '''<iis:WebProperty Id="LogInUTF8" Value="1" />''',

    ("iis", "WebServiceExtension"): '''<iis:WebServiceExtension Id="AspNetExt"
    File="[ASPNET_ISAPI]"
    Description="ASP.NET"
    Allow="yes" />''',

    ("iis", "WebSite"): '''<iis:WebSite Id="MyWebSite"
    Description="My Website"
    Directory="INSTALLDIR"
    AutoStart="yes">
    <iis:WebAddress Id="AllHttp" Port="80" />
</iis:WebSite>''',

    ("iis", "WebVirtualDir"): '''<iis:WebVirtualDir Id="MyVirtualDir"
    Alias="api"
    Directory="APIDIR"
    WebSite="MyWebSite" />''',

    # ===================
    # MSMQ namespace
    # ===================
    ("msmq", "MessageQueue"): '''<msmq:MessageQueue Id="MyQueue"
    PathName=".\\Private$\\MyQueue"
    Label="My Message Queue"
    Transactional="yes" />''',

    ("msmq", "MessageQueuePermission"): '''<msmq:MessageQueuePermission
    User="DOMAIN\\AppUser"
    QueueGenericRead="yes"
    WriteMessage="yes" />''',

    # ===================
    # NetFx namespace
    # ===================
    ("netfx", "DotNetCompatibilityCheck"): '''<netfx:DotNetCompatibilityCheck
    Id="DotNet6Check"
    RuntimeType="desktop"
    Version="6.0.0"
    RollForward="latestMinor"
    Property="DOTNET6_INSTALLED" />''',

    ("netfx", "DotNetCompatibilityCheckRef"): '''<netfx:DotNetCompatibilityCheckRef Id="DotNet6Check" />''',

    ("netfx", "DotNetCoreSdkFeatureBandSearch"): '''<netfx:DotNetCoreSdkFeatureBandSearch
    Id="Sdk6Search"
    MajorVersion="6"
    Variable="DOTNET_SDK_6" />''',

    ("netfx", "DotNetCoreSdkFeatureBandSearchRef"): '''<netfx:DotNetCoreSdkFeatureBandSearchRef Id="Sdk6Search" />''',

    ("netfx", "DotNetCoreSdkSearch"): '''<netfx:DotNetCoreSdkSearch
    Id="SdkSearch"
    Version="6.0.0"
    Variable="DOTNET_SDK" />''',

    ("netfx", "DotNetCoreSdkSearchRef"): '''<netfx:DotNetCoreSdkSearchRef Id="SdkSearch" />''',

    ("netfx", "DotNetCoreSearch"): '''<netfx:DotNetCoreSearch
    Id="DotNetSearch"
    RuntimeType="desktop"
    Version="6.0.0"
    Variable="DOTNET6_VERSION" />''',

    ("netfx", "DotNetCoreSearchRef"): '''<netfx:DotNetCoreSearchRef Id="DotNetSearch" />''',

    ("netfx", "NativeImage"): '''<netfx:NativeImage Id="MyNativeImage"
    Platform="all"
    Priority="1"
    AppBaseDirectory="INSTALLDIR" />''',

    # ===================
    # SQL namespace
    # ===================
    ("sql", "SqlDatabase"): '''<sql:SqlDatabase Id="MyDatabase"
    Server="[SQL_SERVER]"
    Database="MyAppDb"
    CreateOnInstall="yes"
    DropOnUninstall="yes" />''',

    ("sql", "SqlFileSpec"): '''<sql:SqlFileSpec Id="PrimaryFile"
    Name="MyAppDb"
    Filename="[SQL_DATA]MyAppDb.mdf"
    Size="10MB"
    GrowthSize="10%" />''',

    ("sql", "SqlLogFileSpec"): '''<sql:SqlLogFileSpec Id="LogFile"
    Name="MyAppDb_log"
    Filename="[SQL_LOG]MyAppDb_log.ldf"
    Size="5MB" />''',

    ("sql", "SqlScript"): '''<sql:SqlScript Id="CreateTables"
    BinaryRef="CreateTablesScript"
    ExecuteOnInstall="yes"
    RollbackOnInstall="yes"
    SqlDb="MyDatabase"
    Sequence="1" />''',

    ("sql", "SqlString"): '''<sql:SqlString Id="InsertVersion"
    SqlDb="MyDatabase"
    SQL="INSERT INTO Version (Ver) VALUES ('1.0.0')"
    ExecuteOnInstall="yes"
    Sequence="2" />''',

    # ===================
    # UI namespace
    # ===================
    ("ui", "WixUI"): '''<ui:WixUI Id="WixUI_InstallDir"
    InstallDirectory="INSTALLDIR" />''',

    # ===================
    # Util namespace
    # ===================
    ("util", "BroadcastEnvironmentChange"): '''<util:BroadcastEnvironmentChange />''',

    ("util", "BroadcastSettingChange"): '''<util:BroadcastSettingChange />''',

    ("util", "CheckRebootRequired"): '''<util:CheckRebootRequired />''',

    ("util", "CloseApplication"): '''<util:CloseApplication Id="CloseMyApp"
    Target="MyApp.exe"
    RebootPrompt="no"
    PromptToContinue="yes"
    Description="Please close [ProductName] before continuing." />''',

    ("util", "ComponentSearch"): '''<util:ComponentSearch Id="PrevInstallSearch"
    ComponentId="{GUID}"
    Variable="PREV_INSTALLED" />''',

    ("util", "ComponentSearchRef"): '''<util:ComponentSearchRef Id="PrevInstallSearch" />''',

    ("util", "DirectorySearch"): '''<util:DirectorySearch Id="FindAppDir"
    Path="[ProgramFilesFolder]MyApp"
    Variable="APP_DIR" />''',

    ("util", "DirectorySearchRef"): '''<util:DirectorySearchRef Id="FindAppDir" />''',

    ("util", "EventManifest"): '''<util:EventManifest
    MessageFile="[#MyApp.exe]"
    ResourceFile="[#MyApp.exe]" />''',

    ("util", "EventSource"): '''<util:EventSource
    Name="MyApplication"
    Log="Application"
    EventMessageFile="[#MyApp.exe]" />''',

    ("util", "ExitEarlyWithSuccess"): '''<util:ExitEarlyWithSuccess />''',

    ("util", "FailWhenDeferred"): '''<util:FailWhenDeferred />''',

    ("util", "FileSearch"): '''<util:FileSearch Id="FindConfig"
    Path="[INSTALLDIR]config.xml"
    Variable="CONFIG_EXISTS"
    Result="exists" />''',

    ("util", "FileSearchRef"): '''<util:FileSearchRef Id="FindConfig" />''',

    ("util", "FileShare"): '''<util:FileShare Id="DataShare"
    Name="AppData$"
    Description="Application data share" />''',

    ("util", "FileSharePermission"): '''<util:FileSharePermission
    User="Everyone"
    Read="yes" />''',

    ("util", "FormatFile"): '''<util:FormatFile Id="FormatConfig"
    Source="config.template"
    Destination="[INSTALLDIR]config.xml" />''',

    ("util", "Group"): '''<util:Group Id="AppUsers"
    Name="MyApp Users"
    Description="Users of My Application" />''',

    ("util", "GroupRef"): '''<util:GroupRef Id="AppUsers" />''',

    ("util", "InternetShortcut"): '''<util:InternetShortcut Id="WebsiteLink"
    Name="Visit Website"
    Target="https://www.example.com"
    Directory="ProgramMenuFolder" />''',

    ("util", "PerfCounter"): '''<util:PerfCounter Name="Requests"
    Help="Number of requests processed" />''',

    ("util", "PerfCounterManifest"): '''<util:PerfCounterManifest ResourceFileDirectory="INSTALLDIR" />''',

    ("util", "PerformanceCategory"): '''<util:PerformanceCategory Id="MyAppPerf"
    Name="My Application"
    Help="Performance counters for My Application"
    MultiInstance="yes" />''',

    ("util", "PerformanceCounter"): '''<util:PerformanceCounter
    Name="ActiveConnections"
    Type="numberOfItems32"
    Help="Current active connections" />''',

    ("util", "PermissionEx"): '''<util:PermissionEx
    User="BUILTIN\\Users"
    GenericRead="yes"
    Read="yes" />''',

    ("util", "ProductSearch"): '''<util:ProductSearch Id="FindOtherApp"
    UpgradeCode="{GUID}"
    Variable="OTHER_APP_VERSION" />''',

    ("util", "ProductSearchRef"): '''<util:ProductSearchRef Id="FindOtherApp" />''',

    ("util", "QueryNativeMachine"): '''<util:QueryNativeMachine />''',

    ("util", "QueryWindowsDirectories"): '''<util:QueryWindowsDirectories />''',

    ("util", "QueryWindowsDriverInfo"): '''<util:QueryWindowsDriverInfo />''',

    ("util", "QueryWindowsSuiteInfo"): '''<util:QueryWindowsSuiteInfo />''',

    ("util", "QueryWindowsWellKnownSIDs"): '''<util:QueryWindowsWellKnownSIDs />''',

    ("util", "RegistrySearch"): '''<util:RegistrySearch Id="FindJavaHome"
    Root="HKLM"
    Key="SOFTWARE\\JavaSoft\\JDK"
    Value="CurrentVersion"
    Variable="JAVA_VERSION" />''',

    ("util", "RegistrySearchRef"): '''<util:RegistrySearchRef Id="FindJavaHome" />''',

    ("util", "RemoveFolderEx"): '''<util:RemoveFolderEx On="uninstall"
    Property="INSTALLDIR" />''',

    ("util", "RestartResource"): '''<util:RestartResource Id="RestartMyService"
    Path="[System64Folder]svchost.exe"
    ServiceName="MyService" />''',

    ("util", "ServiceConfig"): '''<util:ServiceConfig
    ServiceName="MyService"
    FirstFailureActionType="restart"
    SecondFailureActionType="restart"
    ThirdFailureActionType="none"
    ResetPeriodInDays="1" />''',

    ("util", "TouchFile"): '''<util:TouchFile Id="TouchConfig"
    Path="[INSTALLDIR]config.xml" />''',

    ("util", "User"): '''<util:User Id="ServiceUser"
    Name="MyAppService"
    Password="[SERVICE_PASSWORD]"
    CreateUser="yes"
    CanNotChangePassword="yes"
    PasswordNeverExpires="yes" />''',

    ("util", "WaitForEvent"): '''<util:WaitForEvent Id="WaitForShutdown"
    Name="Global\\MyAppShutdown" />''',

    ("util", "WaitForEventDeferred"): '''<util:WaitForEventDeferred Id="WaitDeferred"
    Name="Global\\MyAppReady" />''',

    ("util", "WindowsFeatureSearch"): '''<util:WindowsFeatureSearch Id="IisSearch"
    Feature="IIS-WebServer"
    Variable="IIS_INSTALLED" />''',

    ("util", "WindowsFeatureSearchRef"): '''<util:WindowsFeatureSearchRef Id="IisSearch" />''',

    ("util", "XmlConfig"): '''<util:XmlConfig Id="SetConnectionString"
    File="[INSTALLDIR]web.config"
    ElementPath="/configuration/connectionStrings/add[@name='Default']"
    Name="connectionString"
    Value="[CONNECTION_STRING]"
    Action="create"
    On="install"
    Node="value" />''',

    ("util", "XmlFile"): '''<util:XmlFile Id="UpdateAppSetting"
    File="[INSTALLDIR]app.config"
    Action="setValue"
    ElementPath="/configuration/appSettings/add[@key='ServerUrl']/@value"
    Value="[SERVER_URL]" />''',

    # ===================
    # VS namespace
    # ===================
    ("vs", "FindVisualStudio"): '''<vs:FindVisualStudio />''',

    ("vs", "VsixPackage"): '''<vs:VsixPackage
    File="MyExtension.vsix"
    PackageId="MyExtension.{GUID}"
    Permanent="no"
    Target="Community,Pro,Enterprise"
    TargetVersion="17.0"
    Vital="yes" />''',

    # ===================
    # WIX (core) namespace - MSI Elements
    # ===================
    ("wix", "AdminExecuteSequence"): '''<AdminExecuteSequence>
    <Custom Action="MyAdminAction" After="CostFinalize" />
</AdminExecuteSequence>''',

    ("wix", "AdminUISequence"): '''<AdminUISequence>
    <Custom Action="MyAdminUIAction" After="CostFinalize" />
</AdminUISequence>''',

    ("wix", "AdvertiseExecuteSequence"): '''<AdvertiseExecuteSequence>
    <Custom Action="MyAdvertiseAction" After="CostFinalize" />
</AdvertiseExecuteSequence>''',

    ("wix", "All"): '''<All />''',

    ("wix", "AllocateRegistrySpace"): '''<AllocateRegistrySpace />''',

    ("wix", "AppId"): '''<AppId Id="{GUID}"
    ActivateAtStorage="yes"
    Description="My Application Server" />''',

    ("wix", "AppSearch"): '''<AppSearch />''',

    ("wix", "ApprovedExeForElevation"): '''<ApprovedExeForElevation Id="MyUpdater"
    Key="SOFTWARE\\MyCompany\\MyApp"
    ValueName="UpdaterPath" />''',

    ("wix", "ArpEntry"): '''<ArpEntry Manufacturer="My Company"
    Contact="support@example.com"
    HelpLink="https://example.com/help"
    AboutUrl="https://example.com" />''',

    ("wix", "AssemblyName"): '''<AssemblyName Name="MyAssembly"
    Version="1.0.0.0"
    Culture="neutral"
    PublicKeyToken="b77a5c561934e089" />''',

    ("wix", "Billboard"): '''<Billboard Id="Billboard1" Feature="MainFeature">
    <Control Id="BillboardText" Type="Text" X="10" Y="10" Width="300" Height="50" />
</Billboard>''',

    ("wix", "BillboardAction"): '''<BillboardAction Id="InstallFilesBillboard"
    Action="InstallFiles" />''',

    ("wix", "Binary"): '''<Binary Id="CustomActionDll"
    SourceFile="CustomActions.dll" />''',

    ("wix", "BinaryRef"): '''<BinaryRef Id="CustomActionDll" />''',

    ("wix", "BindImage"): '''<BindImage />''',

    ("wix", "BootstrapperApplication"): '''<BootstrapperApplication>
    <bal:WixStandardBootstrapperApplication
        Theme="hyperlinkLicense"
        LicenseUrl="https://example.com/license" />
</BootstrapperApplication>''',

    ("wix", "BootstrapperApplicationDll"): '''<BootstrapperApplicationDll
    SourceFile="MyBootstrapperApp.dll"
    DpiAwareness="perMonitorV2" />''',

    ("wix", "BootstrapperApplicationRef"): '''<BootstrapperApplicationRef Id="WixStandardBootstrapperApplication.HyperlinkLicense" />''',

    ("wix", "BootstrapperExtension"): '''<BootstrapperExtension Id="MyExtension"
    SourceFile="MyExtension.dll" />''',

    ("wix", "BootstrapperExtensionRef"): '''<BootstrapperExtensionRef Id="Bal.WixExtension" />''',

    ("wix", "Bundle"): '''<Bundle Name="My Application Setup"
    Version="1.0.0.0"
    Manufacturer="My Company"
    UpgradeCode="{GUID}">
    <BootstrapperApplication>
        <bal:WixStandardBootstrapperApplication Theme="hyperlinkLicense" />
    </BootstrapperApplication>
    <Chain>
        <MsiPackage SourceFile="MyApp.msi" />
    </Chain>
</Bundle>''',

    ("wix", "BundleAttribute"): '''<BundleAttribute Name="CustomAttribute" Value="CustomValue" />''',

    ("wix", "BundleAttributeDefinition"): '''<BundleAttributeDefinition Name="CustomAttribute" Type="string" />''',

    ("wix", "BundleCustomData"): '''<BundleCustomData Id="MyCustomData"
    Type="BootstrapperApplication">
    <BundleElement Id="Element1">
        <BundleAttribute Name="Attr1" Value="Value1" />
    </BundleElement>
</BundleCustomData>''',

    ("wix", "BundleCustomDataRef"): '''<BundleCustomDataRef Id="MyCustomData" />''',

    ("wix", "BundleElement"): '''<BundleElement Id="ConfigElement">
    <BundleAttribute Name="Setting" Value="Enabled" />
</BundleElement>''',

    ("wix", "BundleExtension"): '''<BundleExtension Id="MyBundleExtension"
    SourceFile="MyBundleExtension.dll" />''',

    ("wix", "BundleExtensionRef"): '''<BundleExtensionRef Id="WixDependencyExtension" />''',

    ("wix", "BundlePackage"): '''<BundlePackage SourceFile="SubBundle.exe"
    Compressed="yes" />''',

    ("wix", "BundlePackagePayload"): '''<BundlePackagePayload SourceFile="SubBundle.exe"
    Name="SubBundle.exe" />''',

    ("wix", "CCPSearch"): '''<CCPSearch />''',

    ("wix", "Category"): '''<Category Id="MyCategory"
    AppData="My Application"
    Qualifier="1.0" />''',

    ("wix", "Chain"): '''<Chain>
    <MsiPackage SourceFile="prereq.msi" />
    <MsiPackage SourceFile="main.msi" />
</Chain>''',

    ("wix", "Class"): '''<Class Id="{GUID}"
    Context="InprocServer32"
    Description="My COM Class"
    ThreadingModel="apartment" />''',

    ("wix", "Column"): '''<Column Id="Value"
    PrimaryKey="no"
    Type="string"
    Width="255"
    Nullable="yes"
    Category="text" />''',

    ("wix", "ComboBox"): '''<ComboBox Property="MYOPTION">
    <ListItem Value="Option1" Text="First Option" />
    <ListItem Value="Option2" Text="Second Option" />
</ComboBox>''',

    ("wix", "CommandLine"): '''<CommandLine InstallArgument="/passive"
    UninstallArgument="/quiet"
    RepairArgument="/repair" />''',

    ("wix", "ComplianceCheck"): '''<ComplianceCheck />''',

    ("wix", "ComplianceDrive"): '''<ComplianceDrive />''',

    ("wix", "Component"): '''<Component Id="MainExecutable"
    Guid="*"
    Directory="INSTALLDIR">
    <File Id="MyApp.exe" Source="bin\\MyApp.exe" KeyPath="yes" />
</Component>''',

    ("wix", "ComponentGroup"): '''<ComponentGroup Id="ProductComponents">
    <ComponentRef Id="MainExecutable" />
    <ComponentRef Id="ConfigFile" />
</ComponentGroup>''',

    ("wix", "ComponentGroupRef"): '''<ComponentGroupRef Id="ProductComponents" />''',

    ("wix", "ComponentRef"): '''<ComponentRef Id="MainExecutable" />''',

    ("wix", "Configuration"): '''<Configuration Name="LogLevel"
    Format="Text"
    DefaultValue="Info" />''',

    ("wix", "ConfigurationData"): '''<ConfigurationData Name="LogLevel" Value="Debug" />''',

    ("wix", "Container"): '''<Container Id="MyContainer"
    Name="container.cab"
    Type="detached" />''',

    ("wix", "ContainerRef"): '''<ContainerRef Id="MyContainer" />''',

    ("wix", "Control"): '''<Control Id="NextButton"
    Type="PushButton"
    X="236" Y="243" Width="56" Height="17"
    Default="yes"
    Text="&amp;Next" />''',

    ("wix", "CopyFile"): '''<CopyFile Id="CopyConfig"
    FileRef="DefaultConfig"
    DestinationDirectory="INSTALLDIR"
    DestinationName="config.xml" />''',

    ("wix", "CostFinalize"): '''<CostFinalize />''',

    ("wix", "CostInitialize"): '''<CostInitialize />''',

    ("wix", "CreateFolder"): '''<CreateFolder Directory="DataFolder">
    <Permission User="Everyone" GenericRead="yes" />
</CreateFolder>''',

    ("wix", "CreateFolders"): '''<CreateFolders />''',

    ("wix", "CreateShortcuts"): '''<CreateShortcuts />''',

    ("wix", "Custom"): '''<Custom Action="MyCustomAction" After="InstallFiles">
    NOT Installed
</Custom>''',

    ("wix", "CustomAction"): '''<CustomAction Id="SetInstallDir"
    Property="INSTALLDIR"
    Value="[ProgramFilesFolder]MyApp" />''',

    ("wix", "CustomActionRef"): '''<CustomActionRef Id="WixCloseApplications" />''',

    ("wix", "CustomTable"): '''<CustomTable Id="MyTable">
    <Column Id="Key" PrimaryKey="yes" Type="string" Width="72" />
    <Column Id="Value" Type="string" Width="255" Nullable="yes" />
    <Row>
        <Data Column="Key">Setting1</Data>
        <Data Column="Value">Value1</Data>
    </Row>
</CustomTable>''',

    ("wix", "CustomTableRef"): '''<CustomTableRef Id="MyTable" />''',

    ("wix", "Data"): '''<Data Column="Value">ConfigValue</Data>''',

    ("wix", "DeleteServices"): '''<DeleteServices />''',

    ("wix", "Dependency"): '''<Dependency RequiredId="NetFx48"
    RequiredLanguage="0" />''',

    ("wix", "Dialog"): '''<Dialog Id="MyDialog"
    Width="370" Height="270"
    Title="[ProductName] Setup">
    <Control Id="Title" Type="Text" X="10" Y="10" Width="350" Height="20" Text="Welcome" />
</Dialog>''',

    ("wix", "DialogRef"): '''<DialogRef Id="ErrorDlg" />''',

    ("wix", "DigitalCertificate"): '''<DigitalCertificate Id="MyCert"
    SourceFile="cert.cer" />''',

    ("wix", "DigitalCertificateRef"): '''<DigitalCertificateRef Id="MyCert" />''',

    ("wix", "DigitalSignature"): '''<DigitalSignature SourceFile="signed.dll" />''',

    ("wix", "Directory"): '''<Directory Id="INSTALLDIR" Name="MyApp">
    <Directory Id="ConfigDir" Name="config" />
    <Directory Id="DataDir" Name="data" />
</Directory>''',

    ("wix", "DirectoryRef"): '''<DirectoryRef Id="INSTALLDIR">
    <Component Id="ConfigFile" Guid="*">
        <File Source="config.xml" />
    </Component>
</DirectoryRef>''',

    ("wix", "DisableRollback"): '''<DisableRollback />''',

    ("wix", "DuplicateFiles"): '''<DuplicateFiles />''',

    ("wix", "EmbeddedChainer"): '''<EmbeddedChainer Id="MyChainer"
    BinaryRef="ChainerDll"
    EntryPoint="DoChain" />''',

    ("wix", "EmbeddedChainerRef"): '''<EmbeddedChainerRef Id="MyChainer" />''',

    ("wix", "EmbeddedUI"): '''<EmbeddedUI Id="MyEmbeddedUI"
    SourceFile="MyUI.dll" />''',

    ("wix", "EmbeddedUIResource"): '''<EmbeddedUIResource Id="Logo"
    Name="logo.png"
    SourceFile="logo.png" />''',

    ("wix", "EnsureTable"): '''<EnsureTable Id="RemoveFile" />''',

    ("wix", "Environment"): '''<Environment Id="PathEnv"
    Name="PATH"
    Value="[INSTALLDIR]bin"
    Permanent="no"
    Part="last"
    Action="set"
    System="yes" />''',

    ("wix", "Error"): '''<Error Id="25000">Installation failed: [1]</Error>''',

    ("wix", "Exclude"): '''<Exclude File="debug.pdb" />''',

    ("wix", "Exclusion"): '''<Exclusion ExcludedId="OldVersion" />''',

    ("wix", "ExePackage"): '''<ExePackage Id="VCRedist"
    SourceFile="vc_redist.x64.exe"
    InstallArguments="/quiet /norestart"
    UninstallArguments="/uninstall /quiet"
    DetectCondition="VCRedistInstalled"
    Permanent="no"
    Vital="yes" />''',

    ("wix", "ExePackagePayload"): '''<ExePackagePayload SourceFile="setup.exe"
    Name="setup.exe"
    DownloadUrl="https://example.com/setup.exe" />''',

    ("wix", "ExecuteAction"): '''<ExecuteAction />''',

    ("wix", "ExitCode"): '''<ExitCode Value="3010" Behavior="forceReboot" />''',

    ("wix", "Extension"): '''<Extension Id="myfile"
    ContentType="application/x-myfile">
    <Verb Id="open" Command="Open" TargetFile="MyApp.exe" Argument='"%1"' />
</Extension>''',

    ("wix", "Failure"): '''<Failure Condition="DOTNET_INSTALLED = 0"
    Message="This application requires .NET Framework." />''',

    ("wix", "Feature"): '''<Feature Id="MainFeature"
    Title="Main Application"
    Description="Core application files"
    Level="1">
    <ComponentGroupRef Id="ProductComponents" />
</Feature>''',

    ("wix", "FeatureGroup"): '''<FeatureGroup Id="AllFeatures">
    <FeatureRef Id="MainFeature" />
    <FeatureRef Id="Documentation" />
</FeatureGroup>''',

    ("wix", "FeatureGroupRef"): '''<FeatureGroupRef Id="AllFeatures" />''',

    ("wix", "FeatureRef"): '''<FeatureRef Id="MainFeature" />''',

    ("wix", "File"): '''<File Id="MyApp.exe"
    Name="MyApp.exe"
    Source="bin\\Release\\MyApp.exe"
    KeyPath="yes">
    <Shortcut Id="StartMenuShortcut"
        Directory="ProgramMenuFolder"
        Name="My Application"
        WorkingDirectory="INSTALLDIR" />
</File>''',

    ("wix", "FileCost"): '''<FileCost />''',

    ("wix", "FileTypeMask"): '''<FileTypeMask Offset="0" Mask="FF" Value="4D5A" />''',

    ("wix", "Files"): '''<Files Include="bin\\*.dll" />''',

    ("wix", "FindRelatedProducts"): '''<FindRelatedProducts />''',

    ("wix", "ForceReboot"): '''<ForceReboot />''',

    ("wix", "Fragment"): '''<Fragment>
    <ComponentGroup Id="SharedComponents">
        <Component Id="SharedDll" Directory="INSTALLDIR" Guid="*">
            <File Source="shared.dll" />
        </Component>
    </ComponentGroup>
</Fragment>''',

    ("wix", "Icon"): '''<Icon Id="AppIcon" SourceFile="app.ico" />''',

    ("wix", "IconRef"): '''<IconRef Id="AppIcon" />''',

    ("wix", "IgnoreTable"): '''<IgnoreTable Id="Font" />''',

    ("wix", "Include"): '''<Include xmlns="http://wixtoolset.org/schemas/v4/wxs" />''',

    ("wix", "IniFile"): '''<IniFile Id="AppSettings"
    Name="app.ini"
    Directory="INSTALLDIR"
    Section="Settings"
    Key="InstallPath"
    Value="[INSTALLDIR]"
    Action="createLine" />''',

    ("wix", "IniFileSearch"): '''<IniFileSearch Id="FindConfig"
    Name="app.ini"
    Section="Settings"
    Key="ConfigPath"
    Property="CONFIGPATH" />''',

    ("wix", "InstallAdminPackage"): '''<InstallAdminPackage />''',

    ("wix", "InstallExecute"): '''<InstallExecute />''',

    ("wix", "InstallExecuteAgain"): '''<InstallExecuteAgain />''',

    ("wix", "InstallExecuteSequence"): '''<InstallExecuteSequence>
    <Custom Action="MyCustomAction" Before="InstallFinalize">
        NOT Installed
    </Custom>
</InstallExecuteSequence>''',

    ("wix", "InstallFiles"): '''<InstallFiles />''',

    ("wix", "InstallFinalize"): '''<InstallFinalize />''',

    ("wix", "InstallInitialize"): '''<InstallInitialize />''',

    ("wix", "InstallODBC"): '''<InstallODBC />''',

    ("wix", "InstallServices"): '''<InstallServices />''',

    ("wix", "InstallUISequence"): '''<InstallUISequence>
    <Custom Action="MyUIAction" After="CostFinalize" />
</InstallUISequence>''',

    ("wix", "InstallValidate"): '''<InstallValidate />''',

    ("wix", "Instance"): '''<Instance Id="Instance1"
    ProductCode="{GUID}"
    ProductName="My App Instance 1" />''',

    ("wix", "InstanceTransforms"): '''<InstanceTransforms Property="INSTANCEID">
    <Instance Id="I1" ProductCode="{GUID}" ProductName="Instance 1" />
</InstanceTransforms>''',

    ("wix", "IsolateComponent"): '''<IsolateComponent Shared="SharedDll" />''',

    ("wix", "IsolateComponents"): '''<IsolateComponents />''',

    ("wix", "Launch"): '''<Launch Condition="VersionNT >= 603"
    Message="Windows 8.1 or later is required." />''',

    ("wix", "LaunchConditions"): '''<LaunchConditions />''',

    ("wix", "Level"): '''<Level Value="1" Condition="PREMIUM" />''',

    ("wix", "ListBox"): '''<ListBox Property="LANGUAGESELECT">
    <ListItem Value="en-US" Text="English (US)" />
    <ListItem Value="de-DE" Text="German" />
</ListBox>''',

    ("wix", "ListItem"): '''<ListItem Value="Option1" Text="First Option" Icon="Option1Icon" />''',

    ("wix", "ListView"): '''<ListView Property="FEATURESELECT">
    <ListItem Value="Feature1" Text="Main Feature" Icon="Feature1Icon" />
</ListView>''',

    ("wix", "Log"): '''<Log Name="MyApp_Install.log" />''',

    ("wix", "MIME"): '''<MIME ContentType="application/x-myapp" Default="yes" />''',

    ("wix", "MajorUpgrade"): '''<MajorUpgrade
    DowngradeErrorMessage="A newer version is already installed."
    Schedule="afterInstallInitialize"
    AllowDowngrades="no"
    AllowSameVersionUpgrades="no" />''',

    ("wix", "Media"): '''<Media Id="1" Cabinet="product.cab" EmbedCab="yes" />''',

    ("wix", "MediaTemplate"): '''<MediaTemplate EmbedCab="yes" CompressionLevel="high" />''',

    ("wix", "Merge"): '''<Merge Id="VCRedist"
    SourceFile="Microsoft_VC142_CRT_x64.msm"
    DiskId="1"
    Language="0" />''',

    ("wix", "MergeRef"): '''<MergeRef Id="VCRedist" />''',

    ("wix", "MigrateFeatureStates"): '''<MigrateFeatureStates />''',

    ("wix", "Module"): '''<Module Id="MyModule"
    Language="1033"
    Version="1.0.0.0">
    <Package InstallerVersion="500" />
</Module>''',

    ("wix", "MoveFiles"): '''<MoveFiles />''',

    ("wix", "MsiPackage"): '''<MsiPackage Id="MainProduct"
    SourceFile="MyApp.msi"
    DisplayInternalUI="no"
    Compressed="yes"
    Vital="yes">
    <MsiProperty Name="INSTALLDIR" Value="[InstallFolder]" />
</MsiPackage>''',

    ("wix", "MsiPackagePayload"): '''<MsiPackagePayload SourceFile="product.msi"
    Name="product.msi" />''',

    ("wix", "MsiProperty"): '''<MsiProperty Name="INSTALLLEVEL" Value="3" />''',

    ("wix", "MsiPublishAssemblies"): '''<MsiPublishAssemblies />''',

    ("wix", "MsiUnpublishAssemblies"): '''<MsiUnpublishAssemblies />''',

    ("wix", "MspPackage"): '''<MspPackage Id="HotFix1"
    SourceFile="hotfix.msp"
    Slipstream="no" />''',

    ("wix", "MspPackagePayload"): '''<MspPackagePayload SourceFile="patch.msp"
    Name="patch.msp" />''',

    ("wix", "MsuPackage"): '''<MsuPackage Id="WindowsUpdate"
    SourceFile="Windows10-KB123456-x64.msu"
    Permanent="yes"
    Vital="yes" />''',

    ("wix", "MsuPackagePayload"): '''<MsuPackagePayload SourceFile="update.msu"
    Name="update.msu" />''',

    ("wix", "MultiString"): '''<MultiString Id="PathList">
    <MultiStringValue>C:\\Path1</MultiStringValue>
    <MultiStringValue>C:\\Path2</MultiStringValue>
</MultiString>''',

    ("wix", "MultiStringValue"): '''<MultiStringValue>C:\\MyPath</MultiStringValue>''',

    ("wix", "ODBCDataSource"): '''<ODBCDataSource Id="MyDataSource"
    Name="MyAppData"
    DriverName="SQL Server"
    Registration="perMachine" />''',

    ("wix", "ODBCDriver"): '''<ODBCDriver Id="MyDriver"
    Name="My ODBC Driver"
    File="driver.dll" />''',

    ("wix", "ODBCTranslator"): '''<ODBCTranslator Id="MyTranslator"
    Name="My ODBC Translator"
    File="translator.dll" />''',

    ("wix", "OptimizeCustomActions"): '''<OptimizeCustomActions />''',

    ("wix", "OptionalUpdateRegistration"): '''<OptionalUpdateRegistration
    Manufacturer="My Company"
    Department="Engineering"
    ProductFamily="My Product Line" />''',

    ("wix", "Package"): '''<Package Name="My Application"
    Version="1.0.0"
    Manufacturer="My Company"
    UpgradeCode="{GUID}"
    Scope="perMachine"
    Compressed="yes">
    <MajorUpgrade DowngradeErrorMessage="A newer version is installed." />
</Package>''',

    ("wix", "PackageCertificates"): '''<PackageCertificates>
    <DigitalCertificate Id="Cert1" SourceFile="cert.cer" />
</PackageCertificates>''',

    ("wix", "PackageGroup"): '''<PackageGroup Id="Prerequisites">
    <ExePackage Id="VCRedist" SourceFile="vc_redist.exe" />
</PackageGroup>''',

    ("wix", "PackageGroupRef"): '''<PackageGroupRef Id="Prerequisites" />''',

    ("wix", "Patch"): '''<Patch AllowRemoval="yes"
    Manufacturer="My Company"
    DisplayName="My Application Hotfix 1"
    MoreInfoURL="https://example.com/hotfix1">
    <PatchFamily Id="MyPatch" Version="1.0.0.1" />
</Patch>''',

    ("wix", "PatchBaseline"): '''<PatchBaseline Id="Baseline1" />''',

    ("wix", "PatchCertificates"): '''<PatchCertificates>
    <DigitalCertificate Id="PatchCert" SourceFile="patchcert.cer" />
</PatchCertificates>''',

    ("wix", "PatchFamily"): '''<PatchFamily Id="MainPatch"
    Version="1.0.1"
    Supersede="yes" />''',

    ("wix", "PatchFamilyGroup"): '''<PatchFamilyGroup Id="AllPatches">
    <PatchFamilyRef Id="MainPatch" />
</PatchFamilyGroup>''',

    ("wix", "PatchFamilyGroupRef"): '''<PatchFamilyGroupRef Id="AllPatches" />''',

    ("wix", "PatchFamilyRef"): '''<PatchFamilyRef Id="MainPatch" />''',

    ("wix", "PatchFiles"): '''<PatchFiles />''',

    ("wix", "PatchInformation"): '''<PatchInformation Manufacturer="My Company"
    Description="Security update for My Application" />''',

    ("wix", "PatchProperty"): '''<PatchProperty Name="AllowRemoval" Value="1" />''',

    ("wix", "Payload"): '''<Payload SourceFile="data.zip"
    Name="data.zip"
    Compressed="yes" />''',

    ("wix", "PayloadGroup"): '''<PayloadGroup Id="DataFiles">
    <Payload SourceFile="file1.dat" />
    <Payload SourceFile="file2.dat" />
</PayloadGroup>''',

    ("wix", "PayloadGroupRef"): '''<PayloadGroupRef Id="DataFiles" />''',

    ("wix", "Payloads"): '''<Payloads>
    <Payload SourceFile="setup.exe" />
</Payloads>''',

    ("wix", "Permission"): '''<Permission User="Everyone"
    GenericRead="yes"
    GenericExecute="yes" />''',

    ("wix", "ProcessComponents"): '''<ProcessComponents />''',

    ("wix", "ProgId"): '''<ProgId Id="MyApp.Document"
    Description="My Application Document"
    Icon="AppIcon">
    <Extension Id="mydoc" ContentType="application/x-mydoc">
        <Verb Id="open" Command="Open" TargetFile="MyApp.exe" Argument='"%1"' />
    </Extension>
</ProgId>''',

    ("wix", "ProgressText"): '''<ProgressText Action="InstallFiles">Copying files...</ProgressText>''',

    ("wix", "Property"): '''<Property Id="INSTALLDIR" Value="C:\\Program Files\\MyApp" />''',

    ("wix", "PropertyRef"): '''<PropertyRef Id="NETFRAMEWORK48" />''',

    ("wix", "Provides"): '''<Provides Key="MyApp" Version="1.0.0" DisplayName="My Application" />''',

    ("wix", "Publish"): '''<Publish Dialog="WelcomeDlg"
    Control="Next"
    Event="NewDialog"
    Value="InstallDirDlg">1</Publish>''',

    ("wix", "PublishComponents"): '''<PublishComponents />''',

    ("wix", "PublishFeatures"): '''<PublishFeatures />''',

    ("wix", "PublishProduct"): '''<PublishProduct />''',

    ("wix", "RMCCPSearch"): '''<RMCCPSearch />''',

    ("wix", "RadioButton"): '''<RadioButton Value="Option1"
    X="10" Y="10" Width="200" Height="17"
    Text="First Option" />''',

    ("wix", "RadioButtonGroup"): '''<RadioButtonGroup Property="SELECTOPTION">
    <RadioButton Value="1" X="10" Y="10" Width="200" Height="17" Text="Option 1" />
    <RadioButton Value="2" X="10" Y="30" Width="200" Height="17" Text="Option 2" />
</RadioButtonGroup>''',

    ("wix", "RegisterClassInfo"): '''<RegisterClassInfo />''',

    ("wix", "RegisterComPlus"): '''<RegisterComPlus />''',

    ("wix", "RegisterExtensionInfo"): '''<RegisterExtensionInfo />''',

    ("wix", "RegisterFonts"): '''<RegisterFonts />''',

    ("wix", "RegisterMIMEInfo"): '''<RegisterMIMEInfo />''',

    ("wix", "RegisterProduct"): '''<RegisterProduct />''',

    ("wix", "RegisterProgIdInfo"): '''<RegisterProgIdInfo />''',

    ("wix", "RegisterTypeLibraries"): '''<RegisterTypeLibraries />''',

    ("wix", "RegisterUser"): '''<RegisterUser />''',

    ("wix", "RegistryKey"): '''<RegistryKey Root="HKLM"
    Key="SOFTWARE\\MyCompany\\MyApp">
    <RegistryValue Name="Version" Type="string" Value="1.0.0" />
    <RegistryValue Name="InstallDir" Type="string" Value="[INSTALLDIR]" />
</RegistryKey>''',

    ("wix", "RegistryValue"): '''<RegistryValue Root="HKLM"
    Key="SOFTWARE\\MyCompany\\MyApp"
    Name="Installed"
    Type="integer"
    Value="1"
    KeyPath="yes" />''',

    ("wix", "RelatedBundle"): '''<RelatedBundle Id="{UPGRADE-CODE-GUID}"
    Action="upgrade" />''',

    ("wix", "RemoteBundle"): '''<RemoteBundle BundleId="{BUNDLE-ID-GUID}"
    Version="1.0.0"
    UpgradeCode="{UPGRADE-CODE-GUID}" />''',

    ("wix", "RemoteRelatedBundle"): '''<RemoteRelatedBundle Id="{BUNDLE-ID}"
    Action="detect" />''',

    ("wix", "RemoveDuplicateFiles"): '''<RemoveDuplicateFiles />''',

    ("wix", "RemoveEnvironmentStrings"): '''<RemoveEnvironmentStrings />''',

    ("wix", "RemoveExistingProducts"): '''<RemoveExistingProducts />''',

    ("wix", "RemoveFile"): '''<RemoveFile Id="RemoveLogFiles"
    Name="*.log"
    On="uninstall"
    Directory="INSTALLDIR" />''',

    ("wix", "RemoveFiles"): '''<RemoveFiles />''',

    ("wix", "RemoveFolder"): '''<RemoveFolder Id="RemoveInstallDir"
    On="uninstall"
    Directory="INSTALLDIR" />''',

    ("wix", "RemoveFolders"): '''<RemoveFolders />''',

    ("wix", "RemoveIniValues"): '''<RemoveIniValues />''',

    ("wix", "RemoveODBC"): '''<RemoveODBC />''',

    ("wix", "RemoveRegistryKey"): '''<RemoveRegistryKey Id="RemoveAppKey"
    Root="HKLM"
    Key="SOFTWARE\\MyCompany\\MyApp"
    Action="removeOnUninstall" />''',

    ("wix", "RemoveRegistryValue"): '''<RemoveRegistryValue Root="HKCU"
    Key="SOFTWARE\\MyCompany\\MyApp"
    Name="TempValue" />''',

    ("wix", "RemoveRegistryValues"): '''<RemoveRegistryValues />''',

    ("wix", "RemoveShortcuts"): '''<RemoveShortcuts />''',

    ("wix", "RequiredPrivilege"): '''<RequiredPrivilege Name="SeServiceLogonRight" />''',

    ("wix", "Requires"): '''<Requires Id="NetFx48" />''',

    ("wix", "RequiresRef"): '''<RequiresRef Id="NetFx48" />''',

    ("wix", "ReserveCost"): '''<ReserveCost Id="ReserveSpace"
    Directory="INSTALLDIR"
    RunLocal="1048576"
    RunFromSource="0" />''',

    ("wix", "ResolveSource"): '''<ResolveSource />''',

    ("wix", "RollbackBoundary"): '''<RollbackBoundary Id="AfterPrereqs"
    Vital="yes" />''',

    ("wix", "Row"): '''<Row>
    <Data Column="Property">CustomSetting</Data>
    <Data Column="Value">CustomValue</Data>
</Row>''',

    ("wix", "SFPCatalog"): '''<SFPCatalog Name="MyCatalog"
    SourceFile="catalog.cat" />''',

    ("wix", "SFPFile"): '''<SFPFile Id="ProtectedFile"
    File="SystemFile" />''',

    ("wix", "ScheduleReboot"): '''<ScheduleReboot />''',

    ("wix", "SelfRegModules"): '''<SelfRegModules />''',

    ("wix", "SelfUnregModules"): '''<SelfUnregModules />''',

    ("wix", "ServiceArgument"): '''<ServiceArgument>-config "[INSTALLDIR]service.config"</ServiceArgument>''',

    ("wix", "ServiceConfigFailureActions"): '''<ServiceConfigFailureActions
    OnInstall="yes"
    OnReinstall="yes"
    OnUninstall="no"
    ResetPeriod="86400">
    <Failure Action="restart" Delay="60" />
    <Failure Action="restart" Delay="120" />
    <Failure Action="none" Delay="0" />
</ServiceConfigFailureActions>''',

    ("wix", "ServiceControl"): '''<ServiceControl Id="ControlMyService"
    Name="MyService"
    Start="install"
    Stop="both"
    Remove="uninstall"
    Wait="yes" />''',

    ("wix", "ServiceDependency"): '''<ServiceDependency Id="MSSQLSERVER" />''',

    ("wix", "ServiceInstall"): '''<ServiceInstall Id="MyService"
    Name="MyService"
    DisplayName="My Application Service"
    Description="Background service for My Application"
    Type="ownProcess"
    Start="auto"
    ErrorControl="normal"
    Account="NT AUTHORITY\\LocalService" />''',

    ("wix", "SetDirectory"): '''<SetDirectory Id="INSTALLDIR"
    Value="[WindowsVolume]MyApp"
    Sequence="execute" />''',

    ("wix", "SetODBCFolders"): '''<SetODBCFolders />''',

    ("wix", "SetProperty"): '''<SetProperty Id="INSTALLDIR"
    Value="[ProgramFilesFolder]MyCompany\\MyApp"
    Before="CostInitialize"
    Sequence="execute" />''',

    ("wix", "SetVariable"): '''<SetVariable Id="InstallFolder"
    Value="[ProgramFilesFolder]MyApp"
    Type="string" />''',

    ("wix", "SetVariableRef"): '''<SetVariableRef Id="InstallFolder" />''',

    ("wix", "Shortcut"): '''<Shortcut Id="DesktopShortcut"
    Name="My Application"
    Description="Launch My Application"
    Directory="DesktopFolder"
    Target="[#MyApp.exe]"
    WorkingDirectory="INSTALLDIR"
    Icon="AppIcon" />''',

    ("wix", "ShortcutProperty"): '''<ShortcutProperty Key="System.AppUserModel.ID"
    Value="MyCompany.MyApp" />''',

    ("wix", "Show"): '''<Show Dialog="WelcomeDlg" />''',

    ("wix", "SlipstreamMsp"): '''<SlipstreamMsp Id="PatchMsp" />''',

    ("wix", "SoftwareTag"): '''<SoftwareTag Regid="example.com"
    InstallDirectory="INSTALLDIR" />''',

    ("wix", "SoftwareTagRef"): '''<SoftwareTagRef Id="MainTag" />''',

    ("wix", "StandardDirectory"): '''<StandardDirectory Id="ProgramFilesFolder">
    <Directory Id="INSTALLDIR" Name="MyApp" />
</StandardDirectory>''',

    ("wix", "StartServices"): '''<StartServices />''',

    ("wix", "StopServices"): '''<StopServices />''',

    ("wix", "Subscribe"): '''<Subscribe Event="SelectionChange"
    Attribute="Enabled" />''',

    ("wix", "Substitution"): '''<Substitution Table="Property"
    Row="INSTALLDIR"
    Column="Value" />''',

    ("wix", "SummaryInformation"): '''<SummaryInformation
    Keywords="Installer"
    Description="My Application Installer Package" />''',

    ("wix", "TargetProductCode"): '''<TargetProductCode Id="{PRODUCT-CODE-GUID}" />''',

    ("wix", "TargetProductCodes"): '''<TargetProductCodes Replace="yes">
    <TargetProductCode Id="{GUID1}" />
    <TargetProductCode Id="{GUID2}" />
</TargetProductCodes>''',

    ("wix", "Text"): '''<Text>Welcome to the installation wizard.</Text>''',

    ("wix", "TextStyle"): '''<TextStyle Id="TitleFont"
    FaceName="Tahoma"
    Size="12"
    Bold="yes" />''',

    ("wix", "TypeLib"): '''<TypeLib Id="{GUID}"
    Language="0"
    MajorVersion="1"
    MinorVersion="0"
    Description="My Type Library" />''',

    ("wix", "UI"): '''<UI>
    <UIRef Id="WixUI_InstallDir" />
    <Property Id="WIXUI_INSTALLDIR" Value="INSTALLDIR" />
</UI>''',

    ("wix", "UIRef"): '''<UIRef Id="WixUI_Minimal" />''',

    ("wix", "UIText"): '''<UIText Id="WelcomeTitle">Welcome to [ProductName]</UIText>''',

    ("wix", "UnpublishComponents"): '''<UnpublishComponents />''',

    ("wix", "UnpublishFeatures"): '''<UnpublishFeatures />''',

    ("wix", "UnregisterClassInfo"): '''<UnregisterClassInfo />''',

    ("wix", "UnregisterComPlus"): '''<UnregisterComPlus />''',

    ("wix", "UnregisterExtensionInfo"): '''<UnregisterExtensionInfo />''',

    ("wix", "UnregisterFonts"): '''<UnregisterFonts />''',

    ("wix", "UnregisterMIMEInfo"): '''<UnregisterMIMEInfo />''',

    ("wix", "UnregisterProgIdInfo"): '''<UnregisterProgIdInfo />''',

    ("wix", "UnregisterTypeLibraries"): '''<UnregisterTypeLibraries />''',

    ("wix", "Update"): '''<Update Property="ProductVersion" Value="1.0.1" />''',

    ("wix", "Upgrade"): '''<Upgrade Id="{UPGRADE-CODE-GUID}">
    <UpgradeVersion Minimum="1.0.0"
        IncludeMinimum="yes"
        Maximum="2.0.0"
        IncludeMaximum="no"
        Property="PREVIOUSVERSIONSINSTALLED" />
</Upgrade>''',

    ("wix", "UpgradeVersion"): '''<UpgradeVersion Minimum="1.0.0"
    Maximum="1.9.9"
    IncludeMinimum="yes"
    IncludeMaximum="yes"
    OnlyDetect="no"
    Property="UPGRADEFOUND" />''',

    ("wix", "Validate"): '''<Validate ProductId="*"
    ProductLanguage="*"
    ProductVersion="*"
    UpgradeCode="{UPGRADE-CODE-GUID}" />''',

    ("wix", "ValidateProductID"): '''<ValidateProductID />''',

    ("wix", "Variable"): '''<Variable Name="InstallFolder"
    bal:Overridable="yes"
    Type="string"
    Value="[ProgramFilesFolder]MyApp" />''',

    ("wix", "Verb"): '''<Verb Id="open"
    Command="Open"
    Sequence="1"
    Argument='"%1"' />''',

    ("wix", "Wix"): '''<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="My Application"
        Version="1.0.0"
        Manufacturer="My Company"
        UpgradeCode="{GUID}">
        <!-- Package contents here -->
    </Package>
</Wix>''',

    ("wix", "WixVariable"): '''<WixVariable Id="WixUILicenseRtf"
    Value="License.rtf" />''',

    ("wix", "WriteEnvironmentStrings"): '''<WriteEnvironmentStrings />''',

    ("wix", "WriteIniValues"): '''<WriteIniValues />''',

    ("wix", "WriteRegistryValues"): '''<WriteRegistryValues />''',
}


def main():
    """Add examples to elements in the database."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    updated = 0
    for (namespace, name), example in ELEMENT_EXAMPLES.items():
        cursor.execute("""
            UPDATE elements
            SET example = ?
            WHERE name = ? AND namespace = ?
            AND (example IS NULL OR example = '')
        """, (example.strip(), name, namespace))
        if cursor.rowcount > 0:
            updated += 1

    conn.commit()

    # Verify
    cursor.execute("""
        SELECT COUNT(*) FROM elements
        WHERE example IS NULL OR example = ''
    """)
    still_missing = cursor.fetchone()[0]

    cursor.execute("SELECT COUNT(*) FROM elements")
    total = cursor.fetchone()[0]

    print(f"Updated {updated} element examples")
    print(f"Total elements: {total}")
    print(f"Still missing examples: {still_missing}")

    conn.close()


if __name__ == "__main__":
    main()
