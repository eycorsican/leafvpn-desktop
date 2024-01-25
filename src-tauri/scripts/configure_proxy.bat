@echo off

set STATE=%1
set IP=%2
set PORT=%3
set PAC_URL="http://%IP%:%PORT%/proxy.pac"

if %STATE% == on (
  reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /f /v AutoConfigURL /t REG_SZ /d %PAC_URL%
)

if %STATE% == off (
  reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /f /v AutoConfigURL
)
