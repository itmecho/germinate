![{{crate}}](https://raw.githubusercontent.com/itmecho/germinate/master/logo.png)

[![Crates.io](https://img.shields.io/crates/v/germinate?style=flat-square&logo=rust)](https://crates.io/crates/germinate)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue?style=flat-square)](https://docs.rs/germinate)
[![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/itmecho/germinate/CI/master?style=flat-square&logo=github)](https://github.com/itmecho/germinate/actions?query=workflow%3ACI)
![Crates.io](https://img.shields.io/crates/d/germinate?style=flat-square)

{{readme}}

## Binary
Germinate provides a CLI for templating files, available from the Github releases. To run the CLI, cimply download the binary for your system and check the usage with `germinate --help`

### Example

The CLI can be used to parse an template file and output it either to `stdout` or optionally, an output file

```
# To print the parsed result to stdout
germinate myfile.txt.tmpl

# To write the output to a file
germinate myfile.txt.tmpl -o myfile.txt
```

## License

[GPL-3.0](https://github.com/itmecho/germinate/blob/master/LICENSE)
