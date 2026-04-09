@echo off
cd /d "%~dp0"

echo ================================
echo   SmartUnZip Build Script
echo ================================
echo.

if not exist "dist" mkdir "dist"

echo [1/3] Building CLI release...
cargo build --release
if errorlevel 1 (
    echo [-] CLI build failed
    goto :end
)
copy /y "target\release\smartunzip.exe" "dist\" >nul
echo [+] CLI: dist\smartunzip.exe
echo.

echo [2/3] Building GUI release...
cd src-tauri
cargo tauri build
if errorlevel 1 (
    echo [-] GUI build failed
    cd ..
    goto :end
)
cd ..
echo [+] GUI build complete
echo.

echo [3/3] Copying GUI files...
if exist "src-tauri\target\release\smartunzip-gui.exe" (
    copy /y "src-tauri\target\release\smartunzip-gui.exe" "dist\" >nul
    echo [+] GUI: dist\smartunzip-gui.exe
)
if exist "src-tauri\target\release\bundle\nsis\smartunzip-gui_*.exe" (
    for %%f in (src-tauri\target\release\bundle\nsis\smartunzip-gui_*.exe) do (
        copy /y "%%f" "dist\" >nul
        echo [+] Installer: dist\%%~nxf
    )
)
echo.

echo ================================
echo   Build Complete
echo ================================
dir /b dist

:end
echo.
pause
