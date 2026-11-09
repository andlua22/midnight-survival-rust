//! Publish an already verified package, retaining its predecessor and rolling back failures.

use std::{io, path::Path};

use crate::{fields::invalid, unity::Error};

/// Callers must finish source/payload verification before publishing.
pub fn publish_verified(candidate: &Path, canonical: &Path, backup: &Path) -> Result<(), Error> {
    publish_with_rename(candidate, canonical, backup, |source, target| {
        std::fs::rename(source, target)
    })
}

fn publish_with_rename(
    candidate: &Path,
    canonical: &Path,
    backup: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> Result<(), Error> {
    let candidate_real = std::fs::canonicalize(candidate)?;
    if !candidate_real.is_dir() {
        return Err(invalid("package candidate must be a directory"));
    }
    let has_previous = canonical.try_exists()?;
    if has_previous && candidate_real.starts_with(std::fs::canonicalize(canonical)?) {
        return Err(invalid("candidate must be outside the canonical package"));
    }
    if backup.try_exists()? {
        return Err(invalid(format!(
            "package backup already exists: {}",
            backup.display()
        )));
    }
    let parent = canonical
        .parent()
        .ok_or_else(|| invalid("canonical package has no parent"))?;
    std::fs::create_dir_all(parent)?;
    if has_previous {
        let parent = backup
            .parent()
            .ok_or_else(|| invalid("package backup has no parent"))?;
        std::fs::create_dir_all(parent)?;
        rename(canonical, backup)?;
    }
    if let Err(error) = rename(candidate, canonical) {
        if has_previous {
            rename(backup, canonical).map_err(|restore| invalid(format!(
                "package publication failed: {error}; restoring its predecessor failed: {restore}; predecessor remains at {}", backup.display())))?;
        }
        return Err(error.into());
    }
    if has_previous {
        eprintln!("Previous package archived at {}", backup.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Fixture {
        root: PathBuf,
        candidate: PathBuf,
        canonical: PathBuf,
        backup: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "midnight-package-publish-{}-{stamp}",
                std::process::id()
            ));
            let candidate = root.join("candidate");
            let canonical = root.join("canonical");
            let backup = root.join("backups/previous");
            std::fs::create_dir_all(&candidate).unwrap();
            std::fs::create_dir_all(&canonical).unwrap();
            std::fs::write(candidate.join("payload"), b"new").unwrap();
            std::fs::write(canonical.join("payload"), b"old").unwrap();
            Self {
                root,
                candidate,
                canonical,
                backup,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn successful_publication_archives_predecessor_and_rejects_self_promotion() {
        let fixture = Fixture::new();
        publish_verified(&fixture.candidate, &fixture.canonical, &fixture.backup).unwrap();
        assert_eq!(
            std::fs::read(fixture.canonical.join("payload")).unwrap(),
            b"new"
        );
        assert_eq!(
            std::fs::read(fixture.backup.join("payload")).unwrap(),
            b"old"
        );
        assert!(!fixture.candidate.exists());
        assert!(
            publish_verified(
                &fixture.canonical,
                &fixture.canonical,
                &fixture.root.join("other-backup")
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(fixture.canonical.join("payload")).unwrap(),
            b"new"
        );
    }

    #[test]
    fn failed_publication_restores_predecessor_without_destroying_candidate() {
        let fixture = Fixture::new();
        let mut calls = 0;
        let result = publish_with_rename(
            &fixture.candidate,
            &fixture.canonical,
            &fixture.backup,
            |source, target| {
                calls += 1;
                if calls == 2 {
                    return Err(io::Error::other("injected publication failure"));
                }
                std::fs::rename(source, target)
            },
        );
        assert!(result.is_err());
        assert_eq!(
            std::fs::read(fixture.canonical.join("payload")).unwrap(),
            b"old"
        );
        assert_eq!(
            std::fs::read(fixture.candidate.join("payload")).unwrap(),
            b"new"
        );
        assert!(!fixture.backup.exists());
    }

    #[test]
    fn existing_backup_or_missing_candidate_preserves_every_package() {
        let fixture = Fixture::new();
        std::fs::create_dir_all(&fixture.backup).unwrap();
        std::fs::write(fixture.backup.join("payload"), b"older").unwrap();
        assert!(publish_verified(&fixture.candidate, &fixture.canonical, &fixture.backup).is_err());
        assert!(
            publish_verified(
                &fixture.root.join("missing"),
                &fixture.canonical,
                &fixture.root.join("other-backup")
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(fixture.canonical.join("payload")).unwrap(),
            b"old"
        );
        assert_eq!(
            std::fs::read(fixture.candidate.join("payload")).unwrap(),
            b"new"
        );
        assert_eq!(
            std::fs::read(fixture.backup.join("payload")).unwrap(),
            b"older"
        );
    }
}
