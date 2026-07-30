@echo off
title Kaspa - Pre-Session Update Check
echo ============================================================
echo  CHECKING FOR UPSTREAM CHANGES
echo ============================================================
echo.
echo ------------------------------------------------------------
echo  [1/3] SILVERSCRIPT commits (empty = up to date)
echo ------------------------------------------------------------
cd /d C:\kaspa-dev\silverscript
git fetch origin
git log HEAD..origin/master --oneline
echo.
echo ------------------------------------------------------------
echo  [2/3] RUSTY-KASPA new tags (look for anything past v2.0.1)
echo ------------------------------------------------------------
cd /d C:\kaspa-dev\rusty-kaspa
git fetch origin --tags
git tag --list "v*"
echo.
echo ------------------------------------------------------------
echo  [3/3] RUSTY-KASPA master commits (general dev activity)
echo ------------------------------------------------------------
git log HEAD..origin/master --oneline
echo.
echo ============================================================
echo  DONE.
echo  Section 1 empty + no new tag in Section 2 = nothing to do.
echo  If SILVERSCRIPT shows commits: recompile oracle_rep_v4.sil
echo  and re-run oracle_v4_verify before trusting the covenant.
echo  If a NEW tag appears past v2.0.1: a newer network
echo  build exists - may need to upgrade node + re-pin.
echo ============================================================
echo.
pause
