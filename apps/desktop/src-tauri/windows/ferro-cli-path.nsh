!include "StrFunc.nsh"
!include "WinMessages.nsh"
${StrStr}
${StrRep}

!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  ${StrStr} $1 "$0" "$INSTDIR\resources\binaries"
  StrCmp $1 "" 0 ferro_cli_path_install_done
  StrCmp $0 "" ferro_cli_path_install_empty
  StrCpy $0 "$0;$INSTDIR\resources\binaries"
  Goto ferro_cli_path_install_done
ferro_cli_path_install_empty:
  StrCpy $0 "$INSTDIR\resources\binaries"
ferro_cli_path_install_done:
  WriteRegExpandStr HKCU "Environment" "Path" $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  ${StrRep} $0 "$0" ";$INSTDIR\resources\binaries" ""
  ${StrRep} $0 "$0" "$INSTDIR\resources\binaries;" ""
  ${StrRep} $0 "$0" "$INSTDIR\resources\binaries" ""
  WriteRegExpandStr HKCU "Environment" "Path" $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend
