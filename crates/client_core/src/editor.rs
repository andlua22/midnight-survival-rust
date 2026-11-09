use crate::{
    hud::{HudLayout, HudPreset, WidgetId},
    storage::StorageService,
};

/// Draft state is separate from committed settings, including preset changes.
pub struct HudEditor {
    committed: HudLayout,
    draft: Option<HudLayout>,
    pub selected: WidgetId,
    pub error: Option<String>,
}

impl HudEditor {
    pub fn new(layout: HudLayout) -> Self {
        Self {
            committed: layout,
            draft: None,
            selected: WidgetId::HealthBars,
            error: None,
        }
    }
    pub fn begin(&mut self) {
        self.draft = Some(self.committed.clone());
        self.error = None;
    }
    pub fn is_editing(&self) -> bool {
        self.draft.is_some()
    }
    pub fn layout(&self) -> &HudLayout {
        self.draft.as_ref().unwrap_or(&self.committed)
    }
    pub fn draft_mut(&mut self) -> Option<&mut HudLayout> {
        self.draft.as_mut()
    }
    pub fn apply_preset(&mut self, preset: HudPreset) {
        if self.is_editing() {
            self.draft = Some(HudLayout::preset(preset));
        }
    }
    pub fn cancel(&mut self) {
        self.draft = None;
        self.error = None;
    }
    pub fn save(&mut self, storage: &dyn StorageService) -> bool {
        let Some(draft) = self.draft.as_mut() else {
            return false;
        };
        if let Err(error) = draft.normalize() {
            self.error = Some(error);
            return false;
        }
        match storage.save_hud(draft) {
            Ok(()) => {
                self.committed = self
                    .draft
                    .take()
                    .expect("draft exists during synchronous save");
                self.error = None;
                true
            }
            Err(error) => {
                self.error = Some(error.to_string());
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{LoadOutcome, StorageError};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStorage {
        saved: Mutex<Option<HudLayout>>,
        fail: bool,
    }
    impl StorageService for MemoryStorage {
        fn load_hud(&self) -> Result<LoadOutcome, StorageError> {
            Ok(LoadOutcome {
                layout: self.saved.lock().unwrap().clone().unwrap_or_default(),
                notice: None,
            })
        }
        fn save_hud(&self, layout: &HudLayout) -> Result<(), StorageError> {
            if self.fail {
                return Err(StorageError("disk unavailable".into()));
            }
            *self.saved.lock().unwrap() = Some(layout.clone());
            Ok(())
        }
    }

    #[test]
    fn cancel_restores_committed_layout_including_preset_and_visibility() {
        let initial = HudLayout::default();
        let mut editor = HudEditor::new(initial.clone());
        editor.begin();
        editor.apply_preset(HudPreset::FourFingers);
        editor
            .draft_mut()
            .unwrap()
            .widgets
            .get_mut(&WidgetId::Minimap)
            .unwrap()
            .visible = false;
        editor.cancel();
        assert_eq!(editor.layout(), &initial);
        assert!(!editor.is_editing());
    }

    #[test]
    fn failed_save_retains_draft_and_old_committed_layout() {
        let initial = HudLayout::default();
        let mut editor = HudEditor::new(initial.clone());
        editor.begin();
        editor.apply_preset(HudPreset::ThreeFingers);
        assert!(!editor.save(&MemoryStorage {
            fail: true,
            ..Default::default()
        }));
        assert!(editor.error.is_some());
        assert!(editor.is_editing());
        assert_eq!(editor.layout().preset, HudPreset::ThreeFingers);
        editor.cancel();
        assert_eq!(editor.layout(), &initial);
    }

    #[test]
    fn successful_save_becomes_next_edit_sessions_baseline() {
        let storage = MemoryStorage::default();
        let mut editor = HudEditor::new(HudLayout::default());
        editor.begin();
        editor.apply_preset(HudPreset::TwoFingers);
        assert!(editor.save(&storage));
        assert!(!editor.is_editing());
        assert_eq!(storage.load_hud().unwrap().layout, *editor.layout());
        editor.begin();
        editor.apply_preset(HudPreset::Desktop);
        editor.cancel();
        assert_eq!(editor.layout().preset, HudPreset::TwoFingers);
    }
}
