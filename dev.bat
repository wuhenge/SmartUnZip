@echo off
cd /d "%~dp0"

echo ================================
echo   SmartUnZip Dev Build
echo ================================
echo.

echo [1/2] Compiling release...
cargo build --release
if errorlevel 1 (
    echo [-] Build failed
    goto :end
)
echo [+] Build OK
echo.

echo [2/2] Copying to dist...
if not exist "dist" mkdir "dist"
copy /y "target\release\smartunzip.exe" "dist\" >nul
echo [+] Done

:end
echo.
pause
