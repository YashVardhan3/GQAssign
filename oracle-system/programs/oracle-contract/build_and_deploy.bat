@echo off
cd /d "%~dp0"
echo Building Anchor Program...
call anchor build
if %errorlevel% neq 0 (
    echo Build failed.
    pause
    exit /b %errorlevel%
)

echo.
echo Deploying to Devnet...
echo Ensure you have a Solana wallet configured (solana config get) and enough devnet SOL (solana airdrop 2).
call anchor deploy
pause