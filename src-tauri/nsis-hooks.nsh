; Domain Scanner - NSIS Installer Hooks
; Custom actions before/after install and uninstall

!macro NSIS_HOOK_PREINSTALL
  ; Check for previous installation and clean up old files
  ReadRegStr $R0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Domain Scanner" "UninstallString"
  ${If} $R0 != ""
    DetailPrint "Found previous installation, will be upgraded..."
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Post-installation tasks
  DetailPrint "Domain Scanner installation complete!"
  
  ; Register the app path so users can find it easily
  WriteRegStr HKLM "Software\Domain Scanner" "InstallPath" "$INSTDIR"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Pre-uninstall: ask user about data retention
  MessageBox MB_YESNO "Do you want to keep application data?$\n$\nChoose Yes to keep your scan results and settings.$\nChoose No to remove everything." IDYES keepData IDNO removeData
  
  keepData:
    DetailPrint "Keeping application data..."
    SetShellVarContext current
    RMDir /r "$APPDATA\com.domainscanner.app"
    goto endUninstall
  
  removeData:
    DetailPrint "Removing application data..."
    SetShellVarContext current
    RMDir /r "$APPDATA\com.domainscanner.app"
  
  endUninstall:
!macroend
