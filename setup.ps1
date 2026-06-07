mkdir -Force libs | Out-Null

echo "Downloading swipl-10.0.2-1.x64.exe..."
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest https://www.swi-prolog.org/download/stable/bin/swipl-10.0.2-1.x64.exe `
  -OutFile libs\swipl-installer.exe

echo "Extracting..."
7z x -y libs\swipl-installer.exe '-olibs\swipl' | Out-Null

rm libs\swipl-installer.exe

if (Test-Path libs\swipl\bin\swipl.exe) {
  echo "Done"
} else {
  echo "Extracted installer did not contain the expected files"
  exit 1
}
