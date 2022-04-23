mod compression;
mod metrics;

use bson::Document;
use std::fs;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;

use crate::diagnostics::metrics::MetricsChunk;

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub path: &'a Path,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> Self {
        // TODO: Validate input path
        DiagnosticData { path }
    }

    pub fn scan(&self) -> io::Result<()> {
        println!("Scanning diagnostic data from {:?}", self.path);

        for entry in fs::read_dir(self.path)? {
            let dir = entry?;
            println!("{:?}", dir.path());

            DiagnosticData::scan_dir(&dir.path())?;
        }

        Ok(())
    }

    fn scan_dir(path: &'a Path) -> io::Result<()> {
        println!("Scanning directory {path:?}");

        for entry in fs::read_dir(path)? {
            let dir = entry?;
            let dir_path = &dir.path();

            println!("{:?}", dir_path);

            if let Some(extension) = dir_path.extension() {
                if extension != "interim" {
                    DiagnosticData::scan_file(&dir.path())?;
                }
            }
        }

        Ok(())
    }

    fn scan_file(path: &'a Path) -> io::Result<()> {
        println!("Scanning file {path:?}");

        let file = fs::File::open(path)?;
        let mut buf_reader = BufReader::new(file);

        let mut metric_docs: Vec<Document> = Vec::new();

        while let Ok(doc) = Document::from_reader(&mut buf_reader) {
            //println!("{doc:?}");

            if let Ok(doc_type) = doc.get_i32("type") {
                if doc_type == 1 {
                    metric_docs.push(doc);
                }
            }
        }

        DiagnosticData::scan_metrics(&metric_docs)?;

        // Err(Io(Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }))
        //let x = Document::from_reader(&mut buf_reader);
        //println!("{x:?}");

        Ok(())
    }

    fn scan_metrics(metric_docs: &Vec<Document>) -> io::Result<()> {
        for metric_doc in metric_docs {
            let mut data = match metric_doc.get_binary_generic("data") {
                Ok(it) => Cursor::new(it),
                Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
            };

            let x = MetricsChunk::from_reader(&mut data)?;
            println!(
                "{:?}",
                x.metrics
                    .iter()
                    .find(|x| x.name.ends_with("wiredTiger/block-manager/blocks read"))
            )
        }

        Ok(())
    }
}
