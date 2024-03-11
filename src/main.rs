use clap::Parser;
use std::error::Error;
use std::fs::{self, metadata, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::result::Result;
use walkdir::WalkDir;
mod cli;
mod front_matter_types;
pub use cli::cli::Args;
pub use front_matter_types::article_bio_like::{generate_bib_lines, MetadataBio};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    let out_path: &Path = Path::new(&args.output_path);

    // create/open a file in the output directory
    let file_result: Result<File, std::io::Error> = if out_path.exists() && !args.overwrite {
        OpenOptions::new().append(true).open(out_path)
    } else {
        File::create(out_path)
    };
    let mut file: File = match file_result {
        Ok(f) => f,
        Err(e) => panic!("Unable to open file: {:?}", e),
    };

    // cycle the files and directories in the provided path
    for entry in WalkDir::new(&args.input_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry: Result<walkdir::DirEntry, walkdir::Error>| entry.ok())
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
        let parsed_document: Result<yaml_front_matter::Document<MetadataBio>, Box<dyn Error>> =
            match args.style.as_str() {
                "article_bio_like" => front_matter_types::parse_document_bio(f),
                _ => panic!("Type not yet implemented"),
            };
        let yaml_front_matter: yaml_front_matter::Document<MetadataBio> = match parsed_document {
            Ok(content) => content,
            Err(_) => continue,
        };
        // get the first author's last name to use as the Key in the .bib format
        let authors: &String = &yaml_front_matter.metadata.author;
        let authors: Vec<&str> = authors.split("and").collect();
        let first: &str = authors[0].trim();
        let first: Vec<&str> = first.split(" ").collect();
        let last_name: &str = first[first.len() - 1];
        // build the .bib formatted string to write to file
        let lines: Vec<String> = generate_bib_lines(&yaml_front_matter);
        let mut output: String = format!(
            "@article{{{}{},\n",
            last_name, &yaml_front_matter.metadata.year
        );
        for val in lines {
            output = format!("{}    {},\n", output, val);
        }
        output = format!("{}}}\n", output);
        // append these lines to the file
        file.write(output.as_bytes())?;
    }
    Ok(())
}
