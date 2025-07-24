#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use anyhow::Result;
use include_dir::Dir;
use planning_poker_database::Database;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("Database error: {0}")]
    Database(#[from] planning_poker_database::DatabaseError),
    #[error("Switchy database error: {0}")]
    SwitchyDatabase(#[from] switchy::database::DatabaseError),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

const MIGRATIONS_TABLE_NAME: &str = "__planning_poker_schema_migrations";

pub struct Migrations {
    pub directory: &'static Dir<'static>,
}

impl Migrations {
    /// Run all migrations in the directory
    ///
    /// # Errors
    ///
    /// Returns `MigrateError` if any migration fails to execute
    pub async fn run(&'static self, db: &dyn Database) -> Result<(), MigrateError> {
        self.run_until(db, None).await
    }

    /// Run migrations up to a specific migration name
    ///
    /// # Errors
    ///
    /// Returns `MigrateError` if any migration fails to execute
    ///
    /// # Panics
    ///
    /// Panics if a migration directory name cannot be extracted (should never happen with valid migration directories)
    pub async fn run_until(
        &'static self,
        db: &dyn Database,
        migration_name: Option<&str>,
    ) -> Result<(), MigrateError> {
        // Create migrations table if it doesn't exist
        self.create_migrations_table(db).await?;

        // Get list of already applied migrations
        let applied_migrations = self.get_applied_migrations(db).await?;

        // Get all migration directories sorted by name
        let mut migration_dirs: Vec<_> = self.directory.dirs().collect();
        migration_dirs.sort_by_key(|dir| dir.path().file_name().unwrap());

        for migration_dir in migration_dirs {
            let migration_name_str = migration_dir
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Stop if we've reached the target migration
            if let Some(target) = migration_name {
                if migration_name_str == target {
                    break;
                }
            }

            // Skip if already applied
            if applied_migrations.contains(&migration_name_str) {
                tracing::debug!("Skipping already applied migration: {}", migration_name_str);
                continue;
            }

            // Run the migration
            self.run_migration(db, migration_dir, &migration_name_str)
                .await?;
        }

        Ok(())
    }

    async fn create_migrations_table(&self, db: &dyn Database) -> Result<(), MigrateError> {
        let sql = format!(
            r"
            CREATE TABLE IF NOT EXISTS {MIGRATIONS_TABLE_NAME} (
                name TEXT PRIMARY KEY NOT NULL,
                run_on TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "
        );

        db.exec_raw(&sql).await?;
        Ok(())
    }

    async fn get_applied_migrations(&self, db: &dyn Database) -> Result<Vec<String>, MigrateError> {
        let rows = db
            .select(MIGRATIONS_TABLE_NAME)
            .columns(&["name"])
            .execute(db)
            .await?;

        let mut migrations = Vec::new();
        for row in rows {
            if let Some(planning_poker_database::DatabaseValue::String(name)) = row.get("name") {
                migrations.push(name);
            }
        }

        Ok(migrations)
    }

    async fn run_migration(
        &self,
        db: &dyn Database,
        migration_dir: &Dir<'static>,
        migration_name: &str,
    ) -> Result<(), MigrateError> {
        tracing::info!("Running migration: {}", migration_name);

        // Find and read the up.sql file
        let up_file_path = format!("{migration_name}/up.sql");
        let up_file = migration_dir.get_file(&up_file_path).ok_or_else(|| {
            MigrateError::Migration(format!("Missing up.sql for migration: {migration_name}"))
        })?;

        let sql = up_file.contents_utf8().ok_or_else(|| {
            MigrateError::Migration(format!(
                "Invalid UTF-8 in up.sql for migration: {migration_name}"
            ))
        })?;

        // Execute the migration SQL
        db.exec_raw(sql).await?;

        // Record the migration as applied
        db.insert(MIGRATIONS_TABLE_NAME)
            .value("name", migration_name)
            .execute(db)
            .await?;

        tracing::info!("Successfully applied migration: {}", migration_name);
        Ok(())
    }
}

// Embedded migrations for SQLite
#[cfg(feature = "sqlite")]
pub const SQLITE_MIGRATIONS: Migrations = Migrations {
    directory: &include_dir::include_dir!("$CARGO_MANIFEST_DIR/migrations/sqlite"),
};

// Embedded migrations for PostgreSQL
#[cfg(feature = "postgres")]
pub const POSTGRES_MIGRATIONS: Migrations = Migrations {
    directory: &include_dir::include_dir!("$CARGO_MANIFEST_DIR/migrations/postgres"),
};

/// Main migration function for the planning poker database
///
/// # Errors
///
/// Returns `MigrateError` if any migration fails to execute
#[allow(clippy::cognitive_complexity)]
pub async fn migrate(db: &dyn Database) -> Result<(), MigrateError> {
    #[cfg(feature = "postgres")]
    {
        tracing::debug!("migrate: running postgres migrations");
        POSTGRES_MIGRATIONS.run(db).await?;
        tracing::debug!("migrate: finished running postgres migrations");
    }

    #[cfg(feature = "sqlite")]
    {
        tracing::debug!("migrate: running sqlite migrations");
        SQLITE_MIGRATIONS.run(db).await?;
        tracing::debug!("migrate: finished running sqlite migrations");
    }

    Ok(())
}

/// Migration function that runs up to a specific migration
///
/// # Errors
///
/// Returns `MigrateError` if any migration fails to execute
#[allow(clippy::cognitive_complexity)]
pub async fn migrate_until(
    db: &dyn Database,
    migration_name: Option<&str>,
) -> Result<(), MigrateError> {
    #[cfg(feature = "postgres")]
    {
        tracing::debug!("migrate_until: running postgres migrations");
        POSTGRES_MIGRATIONS.run_until(db, migration_name).await?;
        tracing::debug!("migrate_until: finished running postgres migrations");
    }

    #[cfg(feature = "sqlite")]
    {
        tracing::debug!("migrate_until: running sqlite migrations");
        SQLITE_MIGRATIONS.run_until(db, migration_name).await?;
        tracing::debug!("migrate_until: finished running sqlite migrations");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "sqlite")]
    use super::*;

    #[test]
    fn test_migrations_directory_exists() {
        #[cfg(feature = "sqlite")]
        {
            assert!(SQLITE_MIGRATIONS.directory.dirs().count() > 0);
        }
        #[cfg(feature = "postgres")]
        {
            assert!(POSTGRES_MIGRATIONS.directory.dirs().count() > 0);
        }
    }

    #[test]
    fn test_migration_files_exist() {
        #[cfg(feature = "sqlite")]
        {
            for migration_dir in SQLITE_MIGRATIONS.directory.dirs() {
                let migration_name = migration_dir.path().file_name().unwrap().to_string_lossy();

                // Check that up.sql exists
                let up_file_path = format!("{migration_name}/up.sql");
                assert!(
                    migration_dir.get_file(&up_file_path).is_some(),
                    "Missing up.sql for migration: {migration_name}"
                );

                // down.sql is optional but if it exists, it should be valid UTF-8
                let down_file_path = format!("{migration_name}/down.sql");
                if let Some(down_file) = migration_dir.get_file(&down_file_path) {
                    assert!(
                        down_file.contents_utf8().is_some(),
                        "Invalid UTF-8 in down.sql for migration: {migration_name}"
                    );
                }
            }
        }
        #[cfg(feature = "postgres")]
        {
            for migration_dir in POSTGRES_MIGRATIONS.directory.dirs() {
                let migration_name = migration_dir.path().file_name().unwrap().to_string_lossy();

                // Check that up.sql exists
                let up_file_path = format!("{migration_name}/up.sql");
                assert!(
                    migration_dir.get_file(&up_file_path).is_some(),
                    "Missing up.sql for migration: {migration_name}"
                );

                // down.sql is optional but if it exists, it should be valid UTF-8
                let down_file_path = format!("{migration_name}/down.sql");
                if let Some(down_file) = migration_dir.get_file(&down_file_path) {
                    assert!(
                        down_file.contents_utf8().is_some(),
                        "Invalid UTF-8 in down.sql for migration: {migration_name}"
                    );
                }
            }
        }
    }
}
