; The autostart task lives in Task Scheduler, outside the install dir, so the uninstaller won't
; remove it. Left behind, it would point at a deleted exe and fail at every sign-in.
; Skipped during updates so autostart survives an upgrade.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
    nsExec::Exec 'schtasks.exe /end /tn "DeadlockPlus"'
    Pop $0
    nsExec::Exec 'schtasks.exe /delete /tn "DeadlockPlus" /f'
    Pop $0
  ${EndIf}
!macroend
