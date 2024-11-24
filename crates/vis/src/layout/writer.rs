use std::fmt::Display;
use std::format;
use std::io::Seek;
use std::io::Write;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::chart::Series;

const COMMON_RESERVED_BYTES: usize =
26 /* static characters */ +
8 /* spaces */ +
32 /* 2 * 16 bytes for two usizes */ +
1 /* new line */;

pub struct SeriesWriter<W, X, Y> {
    writer: W,
    index: usize,
    series: Arc<Series>,
    xtype: PhantomData<X>,
    ytype: PhantomData<Y>,
}

impl<W: Write + Seek, X: Display, Y: Display> SeriesWriter<W, X, Y> {
    pub fn new(writer: W, series: Arc<Series>) -> Self {
        Self {
            writer,
            index: 0,
            series,
            xtype: PhantomData,
            ytype: PhantomData,
        }
    }

    pub fn start(&mut self) -> Result<(), std::io::Error> {
        let total_reserved_bytes =
            COMMON_RESERVED_BYTES + self.series.xs.len() + self.series.xs.len();
        let whitespaces = b" ".repeat(total_reserved_bytes);

        self.writer.write_all(&whitespaces)?;
        self.writer.write_all(b"\n")
    }

    pub fn write(&mut self, x: X, y: Y) -> Result<(), std::io::Error> {
        let line = format!(
            "{xs}[{idx}] = \"{x}\"; {ys}[{idx}] = {y};\n",
            xs = self.series.xs,
            ys = self.series.ys,
            idx = self.index,
            x = x,
            y = y
        );

        self.writer.write_all(line.as_bytes())?;
        self.index += 1;

        Ok(())
    }

    pub fn end(mut self) -> Result<(), std::io::Error> {
        // TODO: Handle the error when rewind fails due to buffer flush
        self.writer.rewind()?;

        let first_line = format!(
            "let {xs} = new Array({size}), {ys} = new Array({size});\n",
            xs = self.series.xs,
            ys = self.series.ys,
            size = self.index
        );

        self.writer.write_all(first_line.as_bytes())
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
        let series = Arc::new(Series::new(String::from("xs"), String::from("ys")));
        let mut series = SeriesWriter::new(&mut writer, series);

        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let expected_output = b"let xs = new Array(5), ys = new Array(5);
                             
xs[0] = 1; ys[0] = 1;
xs[1] = 2; ys[1] = 2;
xs[2] = 3; ys[2] = 3;
xs[3] = 4; ys[3] = 4;
xs[4] = 5; ys[4] = 5;
";
        let expected_output = std::str::from_utf8(expected_output).unwrap();

        series.start()?;

        for (x, y) in xs.into_iter().zip(ys) {
            series.write(x, y)?;
        }

        series.end()?;

        let buff = writer.into_inner();
        let content = std::str::from_utf8(&buff).unwrap();

        assert_eq!(expected_output, content);

        Ok(())
    }
}
