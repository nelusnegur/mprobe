use std::format;
use std::io::Seek;
use std::io::Write;

use crate::chart::Series;

const COMMON_RESERVED_BYTES: usize =
26 /* static characters */ +
8 /* spaces */ +
32 /* 2 * 16 bytes for two usizes */ +
1 /* new line */;

pub struct SeriesWriter<W> {
    writer: W,
    written_items: usize,
    series: Series,
}

impl<W: Write + Seek> SeriesWriter<W> {
    pub fn new(writer: W, series: Series) -> SeriesWriter<W> {
        Self {
            writer,
            written_items: 0,
            series,
        }
    }

    // TODO: Separate start from write and end
    pub fn start(&mut self) -> Result<(), std::io::Error> {
        let total_reserved_bytes =
            COMMON_RESERVED_BYTES + self.series.xs.len() + self.series.xs.len();

        self.writer.write_all(&vec![0; total_reserved_bytes])?;
        self.writer.write_all(b"\n")
    }

    pub fn write(&mut self, data: DataItem) -> Result<(), std::io::Error> {
        let next_idx = self.written_items + 1;

        let line = format!(
            "{xs}[{next_idx}] = {x}; {ys}[{next_idx}] = {y};\n",
            xs = self.series.xs,
            ys = self.series.ys,
            x = data.x,
            y = data.y
        );

        self.writer.write_all(line.as_bytes())?;
        self.written_items += 1;

        Ok(())
    }

    pub fn end(mut self) -> Result<(), std::io::Error> {
        // TODO: Handle the error when rewind fails due to buffer flush
        self.writer.rewind()?;

        let first_line = format!(
            "let {xs} = new Array({size}), {ys} = new Array({size});\n",
            xs = self.series.xs,
            ys = self.series.ys,
            size = self.written_items
        );

        self.writer.write_all(first_line.as_bytes())
    }
}

pub trait DataWriter {
    fn start(&mut self) -> Result<(), std::io::Error>;
    fn write(&mut self, data: DataItem) -> Result<(), std::io::Error>;
    fn end(self) -> Result<(), std::io::Error>;
}

#[derive(Debug)]
pub struct DataItem {
    x: f64,
    y: f64,
}

impl DataItem {
    pub fn new(x: f64, y: f64) -> DataItem {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn write_chart_data() -> Result<(), std::io::Error> {
        let buffer: Vec<u8> = Vec::new();
        let mut writer: Cursor<Vec<u8>> = Cursor::new(buffer);
        let series = Series::new(String::from("xs"), String::from("ys"));
        let mut series = SeriesWriter::new(&mut writer, series);

        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let expected_output = b"let xs = new Array(5), ys = new Array(5);
\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0
xs[1] = 1; ys[1] = 1;
xs[2] = 2; ys[2] = 2;
xs[3] = 3; ys[3] = 3;
xs[4] = 4; ys[4] = 4;
xs[5] = 5; ys[5] = 5;
";
        let expected_output = std::str::from_utf8(expected_output).unwrap();

        series.start()?;

        for (x, y) in xs.into_iter().zip(ys) {
            series.write(DataItem::new(x, y))?;
        }

        series.end()?;

        let buff = writer.into_inner();
        let content = std::str::from_utf8(&buff).unwrap();

        assert_eq!(expected_output, content);

        Ok(())
    }
}
