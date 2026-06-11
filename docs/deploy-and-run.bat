@echo off
REM Usage: deploy-and-run.bat <example-name>
REM   e.g. deploy-and-run.bat oracle_spend_verify
REM Copies the .rs from oracle-protocol (source of truth) into the
REM rusty-kaspa examples dir, then runs it. One step instead of three.

setlocal
set SRC=C:\oracle-protocol\rust-examples
set DST=C:\kaspa-tn12\rusty-kaspa\crypto\txscript\examples
set NAME=%~1

if "%NAME%"=="" (
    echo ERROR: no example name given.
    echo Usage: deploy-and-run.bat ^<example-name^>
    echo Available examples:
    dir /b "%SRC%\*.rs"
    echo.
    pause
    exit /b 1
)

if not exist "%SRC%\%NAME%.rs" (
    echo ERROR: "%SRC%\%NAME%.rs" not found.
    echo Available examples:
    dir /b "%SRC%\*.rs"
    echo.
    pause
    exit /b 1
)

echo Copying %NAME%.rs  (source of truth -^> build dir)...
copy /Y "%SRC%\%NAME%.rs" "%DST%\%NAME%.rs"
echo.
echo Running example: %NAME%
echo ------------------------------------------------------------
cd /d C:\kaspa-tn12\rusty-kaspa
cargo run --release --example %NAME%
echo ------------------------------------------------------------
echo.
pause
endlocal
