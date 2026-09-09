Set WshShell = CreateObject("WScript.Shell")
Set FSO = CreateObject("Scripting.FileSystemObject")
ScriptDir = FSO.GetParentFolderName(WScript.ScriptFullName)
TorExe = ScriptDir & "\tor-real.exe"
Torrc = ScriptDir & "\torrc"
WshShell.Run """" & TorExe & """ -f """" & Torrc & """", 1
Set WshShell = Nothing
Set FSO = Nothing