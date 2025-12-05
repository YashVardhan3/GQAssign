@echo off
echo Setting up Oracle Backend...

if not exist .env (
    echo Creating .env from .env.example...
    copy .env.example .env
    echo [IMPORTANT] A .env file has been created. Please open it and update DATABASE_URL with your PostgreSQL credentials.
) else (
    echo .env already exists. Skipping creation.
)

echo.
echo --- Database Setup Instructions ---
echo 1. Ensure PostgreSQL is running.
echo 2. Create the database:
echo    createdb -U postgres oracle_db
echo 3. Run the migration:
echo    psql -U postgres -d oracle_db -f migrations/001_initial_schema.sql
echo.
echo --- Redis Setup Instructions ---
echo 1. Ensure Redis is running on port 6379.
echo.
pause