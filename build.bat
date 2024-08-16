@echo off
SETLOCAL EnableDelayedExpansion

REM Check for vswhere.exe
if not exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" (
  echo "WARNING: You need VS 2022 (for vswhere.exe)"
  exit /b 1
)

REM Update git repository and submodules
git pull
git submodule update --init --recursive

REM Navigate to cpp directory
pushd cpp

set vswherestr=^"!ProgramFiles(x86)!\Microsoft Visual Studio\Installer\vswhere.exe^" -latest -products * -requires Microsoft.Component.MSBuild -property installationPath
for /f "usebackq tokens=*" %%i in (`!vswherestr!`) do (  
  set BUILDVCTOOLS=%%i\Common7\IDE
  echo BUILDVCTOOLS: !BUILDVCTOOLS!
  if not exist !BUILDVCTOOLS!\devenv.com (
    echo Error: Cannot find VS2022
    exit /b 2
  )

  echo.!BUILDVCTOOLS! | findstr /C:"2022" >nul && (
    "!BUILDVCTOOLS!\devenv.com" "luax.sln" /Rebuild "Release"
  ) || (
    echo Error: Cannot find VS2022
    exit /b 2
  )
)

popd

REM Copy built DLLs to target directory
xcopy .\cpp\build\bin\Release\*.dll .\bin\Windows\ /Y /F
REM Remove lua.dll if it exists
if exist .\bin\Windows\lua.dll (
    del .\bin\Windows\lua.dll
    echo Deleted file: .\bin\Windows\lua.dll
)

REM Navigate to rust directory and build
pushd rust

REM Check if cargo command exists
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 1 (
  cargo build --release
  xcopy .\target\release\*.dll ..\bin\Windows\ /Y /F
) else (
  echo "WARNING: cargo command not found. Skipping Rust build."
)
popd


pause