mkdir -Force libs

Invoke-WebRequest https://github.com/SWI-Prolog/swipl-devel/releases/download/V10.1.21/swipl-10.1.21-1.x64.exe `
  -OutFile libs\swipl-installer.exe

7z x -y libs\swipl-installer.exe '-olibs\swipl'
rm libs\swipl-installer.exe

if (Test-Path 'libs\swipl\bin\swipl.exe') {
  echo "Successfully extracted swipl binaries"
} else {
  echo "Extracted installer did not contain the expected files"
  exit 1
}
