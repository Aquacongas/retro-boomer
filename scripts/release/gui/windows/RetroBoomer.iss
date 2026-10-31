#ifndef MyAppVersion
  #error MyAppVersion is required
#endif
#ifndef SourceDir
  #error SourceDir is required
#endif
#ifndef NumericVersion
  #error NumericVersion is required
#endif
#ifndef OutputDir
  #error OutputDir is required
#endif
#ifndef OutputBaseFilename
  #error OutputBaseFilename is required
#endif
#ifndef IconFile
  #error IconFile is required
#endif
#ifndef LicenseFile
  #error LicenseFile is required
#endif
#ifndef NoticeFile
  #error NoticeFile is required
#endif
#ifndef UserGuideFile
  #error UserGuideFile is required
#endif
#ifndef ContractGuideFile
  #error ContractGuideFile is required
#endif

#ifndef FontLicenseFile
  #error FontLicenseFile is required
#endif

[Setup]
AppId={{4A287E6D-6F73-4B41-940C-D9C14A5956F8}
AppName=Retro Boomer
AppVersion={#MyAppVersion}
AppVerName=Retro Boomer {#MyAppVersion}
AppPublisher=Retro Boomer
AppPublisherURL=https://retroboomer.org
AppSupportURL=https://github.com/Aquacongas/retro-boomer/issues
AppUpdatesURL=https://github.com/Aquacongas/retro-boomer/releases
DefaultDirName={localappdata}\Programs\Retro Boomer
DefaultGroupName=Retro Boomer
DisableProgramGroupPage=yes
AllowNoIcons=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
SetupIconFile={#IconFile}
LicenseFile={#LicenseFile}
UninstallDisplayIcon={app}\retro-boomer-gui.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
SetupLogging=yes
VersionInfoVersion={#NumericVersion}
VersionInfoCompany=Retro Boomer
VersionInfoDescription=Retro Boomer Wallet Installer
VersionInfoProductName=Retro Boomer
VersionInfoProductVersion={#MyAppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "chinesesimplified"; MessagesFile: "{#SourcePath}\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\retro-boomer-gui.exe"; DestDir: "{app}"; DestName: "retro-boomer-gui.exe"; Flags: ignoreversion
Source: "{#SourceDir}\retro-boomerd.exe"; DestDir: "{app}"; DestName: "retro-boomerd.exe"; Flags: ignoreversion
Source: "{#SourceDir}\retro-boomer-miner.exe"; DestDir: "{app}"; DestName: "retro-boomer-miner.exe"; Flags: ignoreversion
Source: "{#LicenseFile}"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "{#NoticeFile}"; DestDir: "{app}"; DestName: "NOTICE.txt"; Flags: ignoreversion
Source: "{#UserGuideFile}"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion
Source: "{#ContractGuideFile}"; DestDir: "{app}"; DestName: "CONTRACTS.md"; Flags: ignoreversion

Source: "{#FontLicenseFile}"; DestDir: "{app}"; DestName: "OFL-1.1.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\Retro Boomer"; Filename: "{app}\retro-boomer-gui.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\Retro Boomer"; Filename: "{app}\retro-boomer-gui.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\retro-boomer-gui.exe"; Description: "{cm:LaunchProgram,Retro Boomer}"; Flags: nowait postinstall skipifsilent
