use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

use ifc_inspector::export::{export_csv, export_json};
use ifc_inspector::parser::parse_ifc_file;
use ifc_inspector::ui::App;

#[derive(Parser, Debug)]
#[command(name = "ifc-inspector")]
#[command(about = "IFC Inspector - browse families and types from IFC files")]
#[command(version)]
struct Args {
    /// Path to IFC file
    #[arg(required = true)]
    file: PathBuf,

    /// Export to CSV (optional output path)
    #[arg(long, value_name = "FILE")]
    csv: Option<PathBuf>,

    /// Export to JSON (optional output path)
    #[arg(long, value_name = "FILE")]
    json: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let project = parse_ifc_file(&args.file)?;

    if let Some(csv_path) = &args.csv {
        export_csv(&project, csv_path)?;
        println!("Exported to CSV: {}", csv_path.display());
    }

    if let Some(json_path) = &args.json {
        export_json(&project, json_path)?;
        println!("Exported to JSON: {}", json_path.display());
    }

    if args.csv.is_some() || args.json.is_some() {
        return Ok(());
    }

    let terminal = ratatui::init();
    let result = App::new(project).run(terminal);
    ratatui::restore();
    result
}
