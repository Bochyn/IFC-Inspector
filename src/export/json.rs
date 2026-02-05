use crate::error::ExportError;
use crate::model::IfcProject;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_json<P: AsRef<Path>>(project: &IfcProject, path: P) -> Result<(), ExportError> {
    let path_ref = path.as_ref();
    let json = serde_json::to_string_pretty(project)?;

    let mut file = File::create(path_ref).map_err(|source| ExportError::FileCreate {
        path: path_ref.to_path_buf(),
        source,
    })?;

    file.write_all(json.as_bytes())
        .map_err(|e| ExportError::WriteError {
            message: e.to_string(),
        })?;

    Ok(())
}
