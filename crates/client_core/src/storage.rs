use std::{
    ffi::OsString,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::hud::HudLayout;

pub struct LoadOutcome {
    pub layout: HudLayout,
    pub notice: Option<String>,
}

#[derive(Debug)]
pub struct StorageError(pub String);

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StorageError {}

/// The platform supplies a storage service; the HUD knows no OS-specific paths.
pub trait StorageService: Send + Sync {
    fn load_hud(&self) -> Result<LoadOutcome, StorageError>;
    fn save_hud(&self, layout: &HudLayout) -> Result<(), StorageError>;
}

pub struct FileStorage {
    directory: PathBuf,
}

impl FileStorage {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
    pub fn path(&self) -> PathBuf {
        self.directory.join("hud.json")
    }
}

impl StorageService for FileStorage {
    fn load_hud(&self) -> Result<LoadOutcome, StorageError> {
        let path = self.path();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LoadOutcome {
                    layout: HudLayout::default(),
                    notice: None,
                });
            }
            Err(error) => {
                return Err(StorageError(format!(
                    "Не вдалося прочитати {}: {error}",
                    path.display()
                )));
            }
        };
        let parsed = serde_json::from_slice::<HudLayout>(&bytes).map_err(|error| error.to_string());
        match parsed.and_then(|mut layout| {
            let repaired = layout.normalize()?;
            Ok((layout, repaired))
        }) {
            Ok((layout, repaired)) => Ok(LoadOutcome {
                layout,
                notice: repaired.then(|| "Некоректні значення HUD виправлено. Файл зміниться лише після натискання «Зберегти».".into()),
            }),
            Err(error) => Ok(LoadOutcome {
                layout: HudLayout::default(),
                notice: Some(format!("Не вдалося завантажити HUD: {error}. Використано пресет ПК; початковий файл не змінено.")),
            }),
        }
    }

    fn save_hud(&self, layout: &HudLayout) -> Result<(), StorageError> {
        let mut normalized = layout.clone();
        normalized.normalize().map_err(StorageError)?;
        let mut bytes = serde_json::to_vec_pretty(&normalized)
            .map_err(|error| StorageError(error.to_string()))?;
        bytes.push(b'\n');
        fs::create_dir_all(&self.directory).map_err(|error| {
            StorageError(format!(
                "Не вдалося створити {}: {error}",
                self.directory.display()
            ))
        })?;

        let temporary = self.directory.join(unique_name(".hud"));
        let mut created_temporary = false;
        let result = (|| -> std::io::Result<()> {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            created_temporary = true;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            // Same directory guarantees an atomic rename on the supported
            // Linux filesystem, without truncating the previous settings.
            fs::rename(&temporary, self.path())
        })();
        if let Err(error) = result {
            if created_temporary {
                let _ = fs::remove_file(&temporary);
            }
            return Err(StorageError(format!(
                "Не вдалося зберегти {}: {error}",
                self.path().display()
            )));
        }
        Ok(())
    }
}

pub fn config_directory(
    override_path: Option<PathBuf>,
    xdg: Option<OsString>,
    home: Option<OsString>,
) -> Result<PathBuf, StorageError> {
    if let Some(path) = override_path {
        return Ok(path);
    }
    if let Some(path) = xdg.map(PathBuf::from).filter(|path| path.is_absolute()) {
        return Ok(path.join("midnight-survival"));
    }
    home.map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|path| path.join(".config/midnight-survival"))
        .ok_or_else(|| {
            StorageError("Немає каталогу конфігурації. Передайте --config-dir PATH.".into())
        })
}

fn unique_name(prefix: &str) -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{prefix}.{}.{}.{}.tmp",
        std::process::id(),
        stamp,
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

/// Atomically write a report to an explicit path using the same filesystem.
pub fn write_report(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| StorageError(e.to_string()))?;
    let temporary = parent.join(unique_name(".benchmark"));
    let mut created_temporary = false;
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        created_temporary = true;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if let Err(error) = result {
        if created_temporary {
            let _ = fs::remove_file(&temporary);
        }
        return Err(StorageError(format!(
            "Не вдалося записати {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::{HudPreset, WidgetId};

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(unique_name("midnight-storage-test"));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn round_trip_preserves_all_widget_settings_and_replaces_atomically() {
        let dir = TestDirectory::new();
        let storage = FileStorage::new(dir.0.clone());
        assert!(storage.load_hud().unwrap().notice.is_none());
        let mut layout = HudLayout::preset(HudPreset::FourFingers);
        layout.widgets.get_mut(&WidgetId::Minimap).unwrap().opacity = 0.47;
        storage.save_hud(&layout).unwrap();
        assert_eq!(storage.load_hud().unwrap().layout, layout);
        let second = HudLayout::default();
        storage.save_hud(&second).unwrap();
        assert_eq!(storage.load_hud().unwrap().layout, second);
        assert_eq!(
            fs::read_dir(&dir.0).unwrap().count(),
            1,
            "temporary files must be removed"
        );
    }

    #[test]
    fn corrupted_and_future_files_are_preserved_until_explicit_save() {
        let dir = TestDirectory::new();
        let storage = FileStorage::new(dir.0.clone());
        for bytes in [b"{broken".as_slice(), b"{\"version\":99}".as_slice()] {
            fs::write(storage.path(), bytes).unwrap();
            let loaded = storage.load_hud().unwrap();
            assert!(loaded.notice.is_some());
            assert_eq!(loaded.layout, HudLayout::default());
            assert_eq!(fs::read(storage.path()).unwrap(), bytes);
        }
    }

    #[test]
    fn io_failure_is_reported_and_does_not_destroy_existing_data() {
        let dir = TestDirectory::new();
        let blocked = dir.0.join("not-a-directory");
        fs::write(&blocked, b"keep me").unwrap();
        let storage = FileStorage::new(blocked.clone());
        assert!(storage.save_hud(&HudLayout::default()).is_err());
        assert_eq!(fs::read(&blocked).unwrap(), b"keep me");
    }

    #[test]
    fn xdg_paths_and_explicit_override_follow_platform_rules() {
        assert_eq!(
            config_directory(Some("relative".into()), None, None).unwrap(),
            PathBuf::from("relative")
        );
        assert_eq!(
            config_directory(None, Some("/settings".into()), Some("/users/test".into())).unwrap(),
            PathBuf::from("/settings/midnight-survival")
        );
        assert_eq!(
            config_directory(None, Some("relative".into()), Some("/users/test".into())).unwrap(),
            PathBuf::from("/users/test/.config/midnight-survival")
        );
        assert!(config_directory(None, None, None).is_err());
    }

    #[test]
    fn partial_file_uses_its_declared_mobile_preset() {
        let dir = TestDirectory::new();
        let storage = FileStorage::new(dir.0.clone());
        fs::write(storage.path(), br#"{"version":1,"preset":"four_fingers"}"#).unwrap();
        let loaded = storage.load_hud().unwrap();
        assert_eq!(loaded.layout, HudLayout::preset(HudPreset::FourFingers));
        assert!(loaded.notice.is_some());
    }

    #[test]
    fn failed_rename_cleans_temporary_files_and_preserves_destination() {
        let dir = TestDirectory::new();
        let storage = FileStorage::new(dir.0.clone());
        fs::create_dir(storage.path()).unwrap();
        fs::write(storage.path().join("keep"), b"original data").unwrap();
        assert!(storage.save_hud(&HudLayout::default()).is_err());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
        assert_eq!(
            fs::read(storage.path().join("keep")).unwrap(),
            b"original data"
        );
    }

    #[test]
    fn benchmark_report_is_written_to_the_explicit_destination() {
        let dir = TestDirectory::new();
        let path = dir.0.join("results/frame-time.json");
        write_report(&path, b"{\"frames\":100}").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{\"frames\":100}");
        write_report(&path, b"{\"frames\":200}").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"{\"frames\":200}");
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }
}
