Copy-Item -Path "target\release\rundell.exe" -Destination "C:\Users\spr\.local\bin" -Force
if (Test-Path "target\release\rundell.pdb") {
	Copy-Item -Path "target\release\rundell.pdb" -Destination "C:\Users\spr\.local\bin" -Force
}
Copy-Item -Path "target\release\rundell-gui.exe" -Destination "C:\Users\spr\.local\bin" -Force
if (Test-Path "target\release\rundell_gui.pdb") {
	Copy-Item -Path "target\release\rundell_gui.pdb" -Destination "C:\Users\spr\.local\bin" -Force
}

$commands = Get-Command rundell -All -ErrorAction SilentlyContinue | Where-Object { $_.CommandType -eq "Application" }
if ($commands) {
	$firstPath = $commands[0].Source
	$expectedPath = "C:\Users\spr\.local\bin\rundell.exe"
	if ($firstPath -ne $expectedPath) {
		Write-Warning "PATH resolves to '$firstPath' before '$expectedPath'."
	}
}

