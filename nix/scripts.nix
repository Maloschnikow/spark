{pkgs}: let
  mkScript = name: text: (pkgs.writeShellScriptBin name text);

  shellScripts = [
    (mkScript "db-reset" ''
      db-stop
      rm -rf $PG_DIR
      db-start
    '')

    (mkScript "db-start" ''
      # Initialize a PostgreSQL database, in case none exists
      if [ ! -d "$PG_DATA_DIR" ]; then
          echo "Initializing PostgreSQL data directory..."
          initdb --locale en_US.UTF-8 -D "$PG_DATA_DIR"
      fi

      # Create a Socket directory, in case none exists
      if [ ! -d "$PG_SOCKET_DIR" ]; then
          echo "Initializing PostgreSQL socket directory..."
          mkdir -p "$PG_SOCKET_DIR"
      fi

      # Check if a postgres is server already running
      if pg_ctl -D "$PG_DATA_DIR" status > /dev/null 2>&1; then
          echo "PostgreSQL is already running. Skipping start."
      else
          # Start PostgreSQL, in case no server runs already
          echo "Starting PostgreSQL..."
          pg_ctl -D "$PG_DATA_DIR" -o "-k $PG_SOCKET_DIR" -l $PG_DIR/psql.log start

          # wait for postgres to start
          sleep 2
      fi

      # Check and create user
      if ! psql -h $PG_SOCKET_DIR -d postgres -tAc "SELECT 1 FROM pg_roles WHERE rolname='$CUSTOM_PGUSER'" | grep -q 1; then
          psql -h $PG_SOCKET_DIR -d postgres -c "CREATE USER $CUSTOM_PGUSER WITH PASSWORD '$PGPASSWORD' CREATEDB;"
      fi

      # Check and create database
      if ! psql -h $PG_SOCKET_DIR -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$PGDATABASE'" | grep -q 1; then
          psql -h $PG_SOCKET_DIR -d postgres -c "CREATE DATABASE \"$PGDATABASE\" OWNER $CUSTOM_PGUSER;"
      fi

      echo "PostgreSQL environment setup complete."
      echo "DATABASE_URL=\"$DATABASE_URL\" (in case you need it)"
    '')

    (mkScript "db-stop" ''
      echo "Cleaning up PostgreSQL..."
      pg_ctl -D "$PG_DATA_DIR" -l $REPO_ROOT/psql.log stop
      echo "Cleanup complete."
    '')

    (mkScript "db-downgrade" ''
      echo "Migrate down by 1..."
      sqlx migrate revert --source "$REPO_ROOT/database/migrations"
    '')

    (mkScript "db-upgrade" ''
      echo "Migrate up everything..."
      sqlx migrate run --source "$REPO_ROOT/database/migrations"
    '')
  ];
in
  shellScripts