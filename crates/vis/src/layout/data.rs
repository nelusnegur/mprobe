use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;
use mprobe_diagnostics::metrics::MetricsChunk;

use crate::chart::Chart;
use crate::chart::Series;
use crate::id::Id;
use crate::layout::series::SeriesWriter;

const DATA_FILE_NAME: &str = "data";

pub struct DataEngine<'a> {
    path: &'a Path,
}

impl<'a> DataEngine<'a> {
    pub fn new(path: &'a Path) -> DataEngine<'a> {
        Self { path }
    }

    pub fn render<I>(&mut self, metrics: I) -> Result<Vec<Chart>, std::io::Error>
    where
        I: Iterator<Item = MetricsChunk>,
    {
        if !self.path.exists() {
            fs::create_dir(self.path)?;
        }

        let mut writers: HashMap<String, SeriesWriter<File, Timestamp, f64>> =
            HashMap::with_capacity(200);
        let mut charts: Vec<Chart> = Vec::with_capacity(500);

        for chunk in metrics {
            for metric in chunk.metrics {
                // TODO: Put the metric name behind the arc
                let writer = match writers.entry(metric.name.clone()) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(vacant_entry) => {
                        let id = Id::next();
                        let file_name = format!("{DATA_FILE_NAME}{id}.js");
                        let file_path: Arc<Path> = Arc::from(self.path.join(file_name));
                        let series = Arc::new(Series::from(id));
                        let writer = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(false)
                            .open(&file_path)?;

                        let chart = Chart::new(
                            id,
                            metric.name.clone(),
                            metric.groups,
                            Arc::clone(&series),
                            file_path,
                        );
                        charts.push(chart);

                        let mut writer = SeriesWriter::new(writer, Arc::clone(&series));
                        writer.start()?;

                        vacant_entry.insert(writer)
                    }
                };

                for measurement in metric.measurements {
                    let x = Timestamp(measurement.timestamp);
                    let y = measurement.value.into();
                    writer.write(x, y)?;
                }
            }
        }

        for (_, writer) in writers {
            writer.end()?;
        }

        Ok(charts)
    }
}

struct Timestamp(DateTime<Utc>);

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0.to_rfc3339())
    }
}
