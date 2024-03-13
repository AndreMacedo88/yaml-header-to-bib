use clap::Parser;
use std::error::Error;
use std::fs::{self, metadata, File};
use std::io::Write;
use std::path::Path;
use std::result::Result;
use walkdir::{DirEntry, WalkDir};
use yaml_front_matter::Document;
mod cli;
mod file_handling;
mod front_matter_styles;
mod process_metadata;
use cli::cli::Args;
use file_handling::create_open_output_file;
use front_matter_styles::article_bio_like::{generate_bib_metadata_lines, MetadataBio};
use front_matter_styles::get_yaml_front_matter;
use process_metadata::{get_first_author_last_name, wrap_metadata_lines};

fn main() -> Result<(), Box<dyn Error>> {
    // parse CLI arguments
    let args: Args = Args::parse();

    // cycle the files and directories in the provided path
    for entry in WalkDir::new(args.input_directory.as_str())
        .follow_links(true)
        .into_iter()
        .filter_map(|entry: Result<DirEntry, walkdir::Error>| entry.ok())
    {
        // turn the DirEntry object into a Path object
        let path: &Path = entry.path();

        // check if the path is pointing to a directory or to a file that is not a markdown
        if metadata(path).unwrap().is_dir()
            || !path.to_str().unwrap().to_lowercase().ends_with(".md")
        {
            continue;
        }

        // read the file and parse the YAML front matter
        let f: String = fs::read_to_string(path)?;
        let parsed_document: Result<Document<MetadataBio>, Box<dyn Error>> =
            get_yaml_front_matter(f, args.style.as_str());
        let yaml_front_matter: Document<MetadataBio> = match parsed_document {
            Ok(content) => content,
            Err(_) => continue,
        };

        // get the first author's last name to use as the Key in the .bib format
        let last_name: &str = get_first_author_last_name(&yaml_front_matter.metadata.author)?;

        // build the .bib formatted string to write to file
        let lines: Vec<String> = generate_bib_metadata_lines(&yaml_front_matter.metadata);
        let output: String =
            wrap_metadata_lines(&yaml_front_matter.metadata.year, last_name, lines);

        // create/open a file in the output directory
        let mut output_handle: File =
            create_open_output_file(&args.output_path.as_str(), args.overwrite);

        // append these lines to the file
        output_handle.write(output.as_bytes())?;
    }
    Ok(())
}
