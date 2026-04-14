// SPDX-License-Identifier: PMPL-1.0-or-later
#![forbid(unsafe_code)]

use crate::model::{now_rfc3339, Annotation, OverlayState, Status, Visibility};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("annotation id not found: {0}")]
    NotFound(String),
}

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct AnnotationStore {
    root: PathBuf,
}

impl AnnotationStore {
    pub fn new(root: impl AsRef<Path>) -> StoreResult<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> StoreResult<Vec<Annotation>> {
        let file = self.annotations_file();
        if !file.exists() {
            return Ok(Vec::new());
        }

        let handle = fs::File::open(file)?;
        let reader = BufReader::new(handle);
        let mut out = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let parsed: Annotation = serde_json::from_str(&line)?;
            out.push(parsed);
        }

        Ok(out)
    }

    pub fn save(&self, annotation: &Annotation) -> StoreResult<()> {
        let mut annotations = self.list()?;
        if let Some(existing) = annotations.iter_mut().find(|item| item.id == annotation.id) {
            *existing = annotation.clone();
        } else {
            annotations.push(annotation.clone());
        }
        self.write_all(&annotations)
    }

    pub fn save_many(&self, annotations: &[Annotation]) -> StoreResult<()> {
        let mut current = self.list()?;
        for ann in annotations {
            if let Some(existing) = current.iter_mut().find(|item| item.id == ann.id) {
                *existing = ann.clone();
            } else {
                current.push(ann.clone());
            }
        }
        self.write_all(&current)
    }

    pub fn get(&self, id: &str) -> StoreResult<Annotation> {
        self.list()?
            .into_iter()
            .find(|item| item.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_string()))
    }

    pub fn update_status(
        &self,
        id: &str,
        status: Status,
        notes: Option<String>,
    ) -> StoreResult<Annotation> {
        let mut annotations = self.list()?;
        let annotation = annotations
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;

        annotation.status = status;
        annotation.updated_at = now_rfc3339();
        if let Some(extra) = notes {
            if annotation.notes.is_empty() {
                annotation.notes = extra;
            } else {
                annotation.notes = format!("{}\n{}", annotation.notes, extra);
            }
        }

        let result = annotation.clone();
        self.write_all(&annotations)?;
        Ok(result)
    }

    pub fn dismiss_for_self(&self, id: &str, notes: Option<String>) -> StoreResult<Annotation> {
        let mut annotations = self.list()?;
        let annotation = annotations
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;

        annotation.status = Status::Dismissed;
        annotation.visibility = Visibility::Private;
        annotation.updated_at = now_rfc3339();
        if let Some(extra) = notes {
            if annotation.notes.is_empty() {
                annotation.notes = extra;
            } else {
                annotation.notes = format!("{}\n{}", annotation.notes, extra);
            }
        }

        let result = annotation.clone();
        self.write_all(&annotations)?;
        Ok(result)
    }

    pub fn update_annotation(&self, updated: &Annotation) -> StoreResult<()> {
        self.save(updated)
    }

    pub fn overlay_state(&self) -> StoreResult<OverlayState> {
        let path = self.overlay_file();
        if !path.exists() {
            return Ok(OverlayState {
                enabled: false,
                updated_at: now_rfc3339(),
            });
        }
        let data = fs::read_to_string(path)?;
        let state: OverlayState = serde_json::from_str(&data)?;
        Ok(state)
    }

    pub fn set_overlay_state(&self, enabled: bool) -> StoreResult<OverlayState> {
        let state = OverlayState {
            enabled,
            updated_at: now_rfc3339(),
        };
        let path = self.overlay_file();
        fs::write(path, serde_json::to_string_pretty(&state)?)?;
        Ok(state)
    }

    fn annotations_file(&self) -> PathBuf {
        self.root.join("annotations.jsonl")
    }

    fn overlay_file(&self) -> PathBuf {
        self.root.join("overlay_state.json")
    }

    fn write_all(&self, annotations: &[Annotation]) -> StoreResult<()> {
        let file = self.annotations_file();
        let mut handle = fs::File::create(file)?;

        for ann in annotations {
            let json = serde_json::to_string(ann)?;
            handle.write_all(json.as_bytes())?;
            handle.write_all(b"\n")?;
        }

        Ok(())
    }
}
