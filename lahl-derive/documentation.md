## How `lahl-derive` works / what is outputs

## Basic

Unit are trivial

```rust
struct X;

impl FromArgs {
	fn from_args_instep() {
		Ok(X)
	}
}
```

Unnamed are very simple

Named are harder, we need to provide out

TODO

- Duplicates
- Positional
- Group
- etc

## Attributes

## Other

See syraise: for `cfg`, `generics`, `Result`, `#[automaticall_derive]` and `seal`
