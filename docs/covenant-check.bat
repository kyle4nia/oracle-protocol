@echo off
title Kaspa - Covenant Compile Check
REM Recompiles oracle_rep_v3.sil and diffs the full silverc output against
REM the known-good baseline. Detects ANY change in the compiled covenant
REM (script bytes, ABI, state layout) - the early-warning for silverscript
REM compiler changes.
REM
REM To re-baseline after an INTENTIONAL, verified change, run with: rebaseline
REM   covenant-check.bat rebaseline

setlocal
set SIL=C:\oracle-protocol\covenants\oracle_rep_v3.sil
set CTOR=C:\kaspa-tn12\silverscript\oracle_ctor_v3.json
set BASELINE=C:\oracle-protocol\docs\covenant_baseline.txt
set CURRENT=C:\oracle-protocol\docs\covenant_current.txt

echo Recompiling oracle_rep_v3.sil ...
cd /d C:\kaspa-tn12\silverscript
cargo run --bin silverc -- "%SIL%" --ctor "%CTOR%" -c > "%CURRENT%" 2>nul

if "%~1"=="rebaseline" (
    copy /Y "%CURRENT%" "%BASELINE%"
    echo.
    echo BASELINE UPDATED. covenant_baseline.txt now matches current output.
    echo.
    pause
    exit /b 0
)

echo.
echo Diffing current output against baseline ...
echo ============================================================
fc "%BASELINE%" "%CURRENT%" >nul
if errorlevel 1 (
    echo  RESULT: *** DIFFERENT *** - the compiled covenant CHANGED.
    echo.
    echo  The silverscript compiler produced different output than the
    echo  known-good baseline. This means script bytes, ABI, or state
    echo  layout changed. Before trusting the covenant:
    echo    1. Review what changed:  fc "%BASELINE%" "%CURRENT%"
    echo    2. If the change is expected/correct, regenerate addresses
    echo       and update the harness, then re-run oracle_spend_verify.
    echo    3. Once verified good, re-baseline:  covenant-check.bat rebaseline
) else (
    echo  RESULT: IDENTICAL - covenant output unchanged. Safe to proceed.
)
echo ============================================================
echo.
pause
endlocal
