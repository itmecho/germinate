# Germinate

[![Crates.io](https://img.shields.io/crates/v/germinate?style=flat-square)](https://crates.io/crates/germinate)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue?style=flat-square)](https://docs.rs/germinate)

A templating library for injecting variables from multiple external sources

## Example

This is a simple example showing how to pull values from the environment

```
use germinate::Seed;

#[tokio::main]
async fn main() {
    std::env::set_var("NAME", "John Wick");
    let mut seed = Seed::new("Hi %env:NAME%!".into());
    let output = seed.germinate().await.unwrap();

    assert_eq!(String::from("Hi John Wick!"), output);
}
```

## Sources

Currently implemented sources:

* `env` - Load values from environment variables
* `awsssm` - Load values from the [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)

### Custom sources
For an example of integrating your own value source, checkout the `Seed` struct in the [docs](https://docs.rs/germinate)

## License

[GPL-3.0](https://github.com/itmecho/germinate/blob/master/LICENSE)
