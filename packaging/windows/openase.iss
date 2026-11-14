; Inno Setup script for the OpenASE-Plus Windows installer.
;
; Build (from the repository root, after `cargo build --release` in rust/):
;   iscc /DAppVersion=0.9.1 packaging\windows\openase.iss
; Optional defines: /DSourceExe=<path to the exe>  /DOutputDir=<folder>
;                   /DOutputName=<setup file name without .exe>

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceExe
  #define SourceExe "..\..\rust\target\release\OpenASEplus.exe"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif
#ifndef OutputName
  #define OutputName "OpenASE-Plus-v" + AppVersion + "-windows-x64-setup"
#endif

[Setup]
; Fixed forever: this GUID is what makes a new setup upgrade an older install.
; OpenASE-Plus has its own (not upstream OpenASE's), so both can be installed.
AppId={{BE92ED7C-7E22-4C2E-A9CC-63ADB627803D}
AppName=OpenASE-Plus
AppVersion={#AppVersion}
AppVerName=OpenASE-Plus {#AppVersion}
AppPublisher=OpenGamePlus
AppPublisherURL=https://github.com/OpenGamePlus/OpenASE-Plus
AppSupportURL=https://github.com/OpenGamePlus/OpenASE-Plus/issues
AppUpdatesURL=https://github.com/OpenGamePlus/OpenASE-Plus/releases
DefaultDirName={autopf}\OpenASE-Plus
DefaultGroupName=OpenASE-Plus
DisableProgramGroupPage=yes
; Per-user by default (no admin prompt); the user can still choose "all users".
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\..\LICENSE
SetupIconFile=..\..\rust\assets\icon.ico
UninstallDisplayIcon={app}\OpenASEplus.exe
UninstallDisplayName=OpenASE-Plus
OutputDir={#OutputDir}
OutputBaseFilename={#OutputName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "OpenASEplus.exe"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\rust\assets\flags-LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; openase://connect/<game>/<host>:<port> links (see rust/src/links.rs).
Root: HKA; Subkey: "Software\Classes\openase"; ValueType: string; ValueName: ""; ValueData: "URL:OpenASE-Plus link"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\openase"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\openase\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\OpenASEplus.exe,0"
Root: HKA; Subkey: "Software\Classes\openase\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\OpenASEplus.exe"" ""%1"""

[Icons]
Name: "{autoprograms}\OpenASE-Plus"; Filename: "{app}\OpenASEplus.exe"
Name: "{autodesktop}\OpenASE-Plus"; Filename: "{app}\OpenASEplus.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\OpenASEplus.exe"; Description: "{cm:LaunchProgram,OpenASE-Plus}"; Flags: nowait postinstall skipifsilent

; Settings live in %APPDATA%\OpenASE-Plus (upstream OpenASE's are imported once). There is deliberately no [UninstallDelete]
; entry for it: uninstalling or upgrading keeps the user's settings.
