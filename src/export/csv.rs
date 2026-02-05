use crate::error::ExportError;
use crate::model::IfcProject;
use std::fs::File;
use std::path::Path;

pub fn export_csv<P: AsRef<Path>>(project: &IfcProject, path: P) -> Result<(), ExportError> {
    let path_ref = path.as_ref();
    let file = File::create(path_ref).map_err(|source| ExportError::FileCreate {
        path: path_ref.to_path_buf(),
        source,
    })?;

    let mut writer = csv::Writer::from_writer(file);

    writer.write_record(["Category", "Type Name", "Instance Count", "Global ID"])?;

    for category in &project.categories {
        for element_type in &category.types {
            writer.write_record([
                &category.name,
                &element_type.name,
                &element_type.instance_count.to_string(),
                &element_type.global_id,
            ])?;
        }
    }

    writer.flush().map_err(|e| ExportError::WriteError {
        message: e.to_string(),
    })?;

    Ok(())
}
