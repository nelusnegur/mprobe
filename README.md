# mprobe

`mprobe` is a diagnostic probe for MongoDB. It enables reading and visualizing
the Full Time Diagnostic Data Capture (FTDC) from the MongoDB data files.
This project is currently in progress: the diagnostics API is ready, but it may
still change; the visualization tool is still very much in progress.

The project consists of the following crates:

- [diagnostics](./crates/diagnostics/)
- [vis](./crates/vis/)
- [cli](./crates/cli/)

## License

`mprobe` project is licensed under [MIT license](LICENSE).
