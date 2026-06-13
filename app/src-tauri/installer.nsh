!macro NSIS_HOOK_POSTINSTALL
  ; Preserve registers
  Push $0
  Push $1

  DetailPrint "Copying DLLs to main directory..."
  CopyFiles /SILENT "$INSTDIR\resources\libs\*.dll" "$INSTDIR\"

  DetailPrint "Installing driver and executables..."
  ClearErrors
  FindFirst $0 $1 "$INSTDIR\resources\libs\*.exe"
  loop:
    IfErrors done
    ; Run the found EXE silently
    DetailPrint "Executing $1 silently..."
    ExecWait '"$INSTDIR\resources\libs\$1" /quiet /norestart'
    FindNext $0 $1
    Goto loop
  done:
    FindClose $0

  ; Restore registers
  Pop $1
  Pop $0
!macroend
