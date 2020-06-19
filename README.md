![germinate](https://raw.githubusercontent.com/itmecho/germinate/master/logo.png)

[![Crates.io](https://img.shields.io/crates/v/germinate?style=flat-square&logo=rust)](https://crates.io/crates/germinate)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue?style=flat-square)](https://docs.rs/germinate)
[![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/itmecho/germinate/CI/master?style=flat-square&logo=github)](https://github.com/itmecho/germinate/actions?query=workflow%3ACI)
![Crates.io](https://img.shields.io/crates/d/germinate?style=flat-square)

This crate provides a method of injecting variables from multiple external sources into a
template string. Sources can be anything as long as they implement the
[`Loader`](https://docs.rs/germinate/*/germinate/trait.Loader.html) trait which handles the
loading of the variables in a standard way.

## Sources
### Optional features
* `default` - `["aws"]`
* `aws` - Enable the AWS value sources

### Built In
These are the currently implemented sources and their associated template keys

| Source | Key | Feature | Description |
|-|-|-|-|
| [AWS EC2 Instance Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | `awsec2tag` | `aws` | Load the value of AWS EC2 Instance Tags by their key |
| [AWS EC2 Metadata Service](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html) | `awsec2metadata` | `aws` | Load a value from the AWS EC2 Metadata Service by it's path |
| [AWS EC2 Tag](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | `awsec2tag` | `aws` | Load a value from an AWS EC2 Tag by it's key |
| Environment Variables | `env` | `-` | Load the value of an environment variable |

#### Example
```rust
let mut seed = Seed::new("Hi %env:NAME%!");
let output = seed.germinate().await?;

assert_eq!("Hi John!", output);
```

### Custom Sources
You can also include your own sources using the
[`Seed::add_custom_loader`](https://docs.rs/germinate/*/germinate/struct.Seed.html#method.add_custom_loader)
method. The only requirement is that the custom loader must implement the
[`Loader`](https://docs.rs/germinate/*/germinate/trait.Loader.html) trait

#### Example
```rust
let mut seed = Seed::new("Hi %name:name%");

// Add a custom loader for the name key. This is the loader that will be used whenever
// germinate finds %name:...% in the template string
seed.add_custom_loader("name".to_string(), Box::new(NameLoader {}));

let output = seed.germinate().await?;

assert_eq!("Hi John", output);
```

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
