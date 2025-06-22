# mprobe

This crate provides a CLI for fetching and visualizing MongoDB diagnostic data (FTDC).

## Usage

### Fetch the diagnostic metrics

For fetching the diagnostic metrics from the MongoDB Atlas, you can
use the `fetch` command as follows:

```bash
mprobe fetch \
    -p <MongoDB Atlas project ID> \
    -k <API key> \
    -s <API secret> \
    -t <resource type (e.g. cluster, replica-set, process)> \
    -n <resource name (e.g. for a replica-set, use: test-123agc-shard-0)> \
    -f [ path where the FTDC will be stored ] \
    -r [ start timestamp ] \
    -o [ end timestamp ]
```

### Visualize the diagnostic metrics

In order to visualize the downloaded diagnostic metrics, you can
use the `view` command as follows:

```bash
mprobe view \
    -p <path to the FTDC directory> \
    -n <node name> \
    -s [ start timestamp ] \
    -e [ end timestamp ]
```

This command will generate an HTML report that includes
all the diagnostic metrics of the specified node in a given time window.
All the data is stored in the `vis` directory relative to the current working
directory, unless otherwise specified via the `-o` option.

To start exploring the metrics, open the `./vis/index.html` page in the browser.

### Help

If you need help with one of the commands or simply would like to see
what other options are available, you can use the `help` command:

```bash
mprobe help
```

To see the options of a specific command, use the following instruction:

```bash
mprobe <command> --help
```

## License

This project is licensed under [MIT license](./LICENSE).
