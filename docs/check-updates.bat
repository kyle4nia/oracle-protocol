@echo off
title Kaspa - Pre-Session Update Check
echo ============================================================
echo  CHECKING FOR UPSTREAM CHANGES
echo ============================================================
echo.
echo ------------------------------------------------------------
echo  [1/3] SILVERSCRIPT commits (empty = up to date)
echo ------------------------------------------------------------
cd /d C:\kaspa-tn12\silverscript
git fetch origin
git log HEAD..origin/master --oneline
echo.
echo ------------------------------------------------------------
echo  [2/3] RUSTY-KASPA new tags (look for anything past tn10-toc3)
echo ------------------------------------------------------------
cd /d C:\kaspa-tn12\rusty-kaspa
git fetch origin --tags
git tag --list "tn10*" "v*toc*"
echo.
echo ------------------------------------------------------------
echo  [3/3] RUSTY-KASPA master commits (general dev activity)
echo ------------------------------------------------------------
git log HEAD..origin/master --oneline
echo.
echo ============================================================
echo  DONE.
echo  Section 1 empty + no new tag in Section 2 = nothing to do.
echo  If SILVERSCRIPT shows commits: recompile oracle_rep_v3.sil
echo  and diff the script bytes before trusting the covenant.
echo  If a NEW tag appears (e.g. tn10-toc4): a newer network
echo  build exists - may need to upgrade node + re-pin.
echo ============================================================
echo.
pause
