use std::cell::RefCell;

mod ccpp;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FileKind {
    Source,
    Deliverable,
    Temporary,
    Other,
}

pub struct Project {
    pub path: std::path::PathBuf,
    pub files: Vec<ccpp::CustomDirEnt>,
    pub artefacts_sizes: std::cell::RefCell<Option<u64>>,
}

impl Project {
    pub fn from_c_project_path(base_path: &std::path::Path) -> Self {
        let files: Vec<ccpp::CustomDirEnt> = ccpp::id_temporary_files(base_path)
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                _ => None,
            })
            .collect();

        let proj = Project {
            path: base_path.to_owned(),
            files,
            artefacts_sizes: RefCell::new(None),
        };
        proj
    }

    fn compute_artefacts_sizes(&self) {
        let sum: u64 = self
            .files
            .iter()
            .filter(|entry| entry.client_state == FileKind::Temporary)
            .map(|file| file.path())
            .filter_map(|path| path.metadata().ok())
            .map(|meta| meta.len())
            .sum();
        *self.artefacts_sizes.borrow_mut() = Some(sum);
    }

    fn get_or_compute_artefact_sizes(&self) -> u64 {
        let potential_value = *self.artefacts_sizes.borrow();
        if let Some(val) = potential_value {
            return val;
        } else {
            self.compute_artefacts_sizes();
            return self.get_or_compute_artefact_sizes();
        }
    }
    pub fn print_temp_and_deliverables(&self) {
        self.files
            .iter()
            .filter(|file| match file.client_state {
                FileKind::Temporary | FileKind::Deliverable => true,
                _ => false,
            })
            .for_each(|file| {
                if let Ok(tmp) = file.path().strip_prefix(&self.path) {
                    println!("- {} {}", tmp.display(), file.client_state)
                }
            });
    }

    pub fn pretty_print(&self) {
        let n_temporary = self
            .files
            .iter()
            .filter(|file| file.client_state == FileKind::Temporary)
            .count();
        let n_deliverable = self
            .files
            .iter()
            .filter(|file| file.client_state == FileKind::Deliverable)
            .count();

        println!("- Project {}", self.path.display());
        println!("    - {} temporary files", n_temporary);
        println!("    - {} deliverable files", n_deliverable);
        println!(
            "    - size of artefacts {} ",
            self.get_or_compute_artefact_sizes()
        );
    }
}
