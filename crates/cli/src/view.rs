use mprobe_diagnostics::DiagnosticData;
use mprobe_diagnostics::MetricsFilter;
use mprobe_vis::layout::VisLayout;

use crate::cli::PathExt;
use crate::cli::ViewArgs;
use crate::error::CliError;

// TODO: Validate start_timestamp < end_timestamp, for fetch as well
pub(crate) fn view(args: ViewArgs) -> Result<(), CliError> {
    let output_path = args.output_path.or_current_dir()?;

    println!(
        "mprobe scans diagnostic data from: `{}` and generates a visual representation in: `{}`",
        args.path.display(),
        output_path.display()
    );

    let filter = MetricsFilter::new(args.node, args.start, args.end);
    let diagnostic_data = DiagnosticData::filter(&args.path, filter).expect("valid path");

    let vis = VisLayout::init(&output_path).expect("initializing data vis directory failed");
    vis.generate_report(diagnostic_data)
        .expect("generating vis report failed");

    Ok(())
}
