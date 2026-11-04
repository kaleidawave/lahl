# LAHL - lightweight argument handling library

[![crates.io badge](https://img.shields.io/crates/v/lahl)](https://crates.io/crates/lahl)
[![docs.rs badge](https://img.shields.io/docsrs/lahl)](https://docs.rs/syraise/latest/lahl/)
[![compiler-club](https://f005.backblazeb2.com/file/6948446f806c172a098591088e0715b8258b380b/cc-discord-badge.png)](https://kaleidawave-public-pages.val.run/r/cc-discord)

Features

- derive macro
- depends on [syraise](https://crates.io/crates/syraise) (no syn)

## Benefits

- Using traits makes this semantic, not syntactic. 
	- Type aliases and generics work
	- No syntatic analysis of `bool` and `Option`
- Low overhead?

## Problems

- Booleans do not eat arguments
	- `struct X(bool)` has to accept no arguments, will always be `X(true)`
- [`--help` is *currently* a mess for large items](https://github.com/kaleidawave/lahl/issues/2)
- [no completition generation at the moment](https://github.com/kaleidawave/lahl/issues/3)

## Comparison

|crate|`clap`|`argh`|`lahl`|
|---|---|---|---|
|derive API|✓|✓|✓|c
|`#[default]`|✓|✓|❌|
|dependencies|syn|syn|syraise|
|doc-comment|✓|✓|✓|
|differentiation between <br>command or subcommands|✓|✓|❌|

## Derive

### Structures

#### Unit structs

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct UnitStruct;

let result = UnitStruct::from_vec(vec![]);
assert_eq!(result, Ok(UnitStruct));
```

#### Unnamed

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct UnnamedField(String);

let result = UnnamedField::from_vec(vec!["hi"]);
assert_eq!(result, Ok(UnnamedField("hi".to_owned())));
```

#### Named

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedFields {
	a: String,
	b: usize,
}

let result = NamedFields::from_vec(vec!["--a", "hi", "--b", "67"]);
assert_eq!(result, Ok(NamedFields { a: "hi".to_owned(), b: 67 }));

let result = NamedFields::from_vec(vec!["--b", "67", "--a", "hi"]);
assert_eq!(result, Ok(NamedFields { a: "hi".to_owned(), b: 67 }));
```

#### Enums

Values from variant name. Transforms variant name to kebab case

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
enum Command {
	List,
	ChangeDirectory,
}

let result = Command::from_vec(vec!["list"]);
assert_eq!(result, Ok(Command::List));

let result = Command::from_vec(vec!["change-directory"]);
assert_eq!(result, Ok(Command::ChangeDirectory));
```

#### Enum variant with named variant

```rust
use lahl::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs, PartialEq, Debug)]
enum MixedEnum {
	Unit,
	Named(String),
	ChangeDirectory { to: PathBuf },
}

let result = MixedEnum::from_vec(vec!["unit"]);
assert_eq!(result, Ok(MixedEnum::Unit));

let result = MixedEnum::from_vec(vec!["named", "hi"]);
assert_eq!(result, Ok(MixedEnum::Named("hi".into())));

let result = MixedEnum::from_vec(vec!["change-directory", "--to", "test"]);
assert_eq!(result, Ok(MixedEnum::ChangeDirectory { to: PathBuf::from("test") }));
```

#### Named with optional fields

> Optional-ness is defined based on traits (so semantic over syntactic)

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedWithOptionalFields {
	optional_value: Option<String>,
	optional_flag: bool,
}

let result = NamedWithOptionalFields::from_vec(vec![]);
assert_eq!(result, Ok(NamedWithOptionalFields { 
	optional_value: None, optional_flag: false 
}));

let result = NamedWithOptionalFields::from_vec(vec!["--optional-value", "hi"]);
assert_eq!(result, Ok(NamedWithOptionalFields { 
	optional_value: Some("hi".into()), optional_flag: false 
}));

let result = NamedWithOptionalFields::from_vec(vec!["--optional-flag"]);
assert_eq!(result, Ok(NamedWithOptionalFields { 
	optional_value: None, optional_flag: true 
}));
```

#### Nesting

> Optional-ness is defined based on traits (so semantic over syntactic)

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct Inner {
	#[lahl(positional)]
	req: String,
	opt: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
struct Outer {
	named: Option<Inner>,
	value: Option<String>,
}

let result = Outer::from_vec(vec![]);
assert_eq!(result, Ok(Outer { named: None, value: None }));

let result = Outer::from_vec(vec!["--value", "123"]);
assert_eq!(result, Ok(Outer { named: None, value: Some("123".to_owned()) }));

let result = Outer::from_vec(vec!["--named", "req1", "--value", "123"]);
assert_eq!(
	result,
	Ok(Outer {
		named: Some(Inner { req: "req1".to_owned(), opt: None }),
		value: Some("123".to_owned())
	})
);

let result = Outer::from_vec(vec!["--named", "req1", "--opt", "v", "--value", "123"]);
assert_eq!(
	result,
	Ok(Outer {
		named: Some(Inner { req: "req1".to_owned(), opt: Some("v".to_owned()) }),
		value: Some("123".to_owned())
	})
);
```

### Input

#### `--key=value`

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedWithOptionalFields {
	optional_value: Option<String>,
	optional_flag: bool,
}

let result = NamedWithOptionalFields::from_vec(vec!["--optional-value=hi"]);

assert_eq!(result, Ok(NamedWithOptionalFields { 
	optional_value: Some("hi".into()), optional_flag: false 
}));
```

### Errors

#### Missing

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
enum Command {
	List,
	ChangeDirectory,
}

let result = Command::from_vec(vec!["missing"]);
assert!(result.is_err());

let result = Command::from_vec(vec![]);
assert!(result.is_err());
```

#### Duplicate

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct Command {
	flag: Option<String>
}

let result = Command::from_vec(vec![]);
assert!(result.is_ok());

let result = Command::from_vec(vec!["--flag", "abc"]);
assert!(result.is_ok());

let result = Command::from_vec(vec!["--flag", "abc", "--flag", "abc"]);
assert!(result.is_err());
```

### Attributes

#### Positional in named

> This can be `Args(String, *struct Flag { flag: bool }*)`, but this provides an easy escape hatch

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct WithPositional {
	#[lahl(positional)]
	owned: String,
	flag: bool,
}

let result = WithPositional::from_vec(vec!["hi"]);
assert_eq!(result, Ok(WithPositional { owned: "hi".to_owned(), flag: false }));

let result = WithPositional::from_vec(vec!["hi", "--flag"]);
assert_eq!(result, Ok(WithPositional { owned: "hi".to_owned(), flag: true }));
```

#### `#[lahl(fall_through)]` means gives a default for unmatched

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
enum WithFallthrough {
	Main,
	#[lahl(fall_through)]
	Else,
}

let result = WithFallthrough::from_vec(vec!["main"]);
assert_eq!(result, Ok(WithFallthrough::Main));

let result = WithFallthrough::from_vec(vec![]);
assert_eq!(result, Ok(WithFallthrough::Else));
```

#### `#[lahl(literal)]` removes `--` from prefix

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct X {
	limit: usize,
	#[lahl(literal)]
	on: Option<String>
}

let result = X::from_vec(vec!["--limit", "10", "on", "files"]);
assert_eq!(result, Ok(X { limit: 10, on: Some("files".to_owned()) }));
```

#### Inner `#[lahl(group)]`. Breaks the hierarchy

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct WithGroup {
	owned: String,
	#[lahl(group)]
	inner: Inner
}

#[derive(FromArgs, PartialEq, Debug)]
struct Inner {
	value: Option<String>,
	flag: bool
}

let result = WithGroup::from_vec(vec!["--owned=hi"]);
assert_eq!(
	result,
	Ok(WithGroup { 
		owned: "hi".to_owned(), 
		inner: Inner { value: None, flag: false } 
	})
);

let result = WithGroup::from_vec(vec!["--value", "40", "--owned=hi"]);
assert_eq!(
	result,
	Ok(WithGroup {
		owned: "hi".to_owned(),
		inner: Inner { value: Some("40".to_owned()), flag: false }
	})
);

let result = WithGroup::from_vec(vec!["--owned=hi", "--flag"]);
assert_eq!(
	result,
	Ok(WithGroup { 
		owned: "hi".to_owned(), 
		inner: Inner { value: None, flag: true } 
	})
);
```

### Collections

#### Bucket

Collects all string arguments upto `-` based arguments

```rust
use lahl::FromArgs;

let result = lahl::Bucket::from_vec(vec!["a", "b", "c"]);
assert_eq!(
	result, 
	Ok(
		lahl::collections::Collection(
			vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
			Default::default()
		)
	)
);
```

#### Bucket in struct?

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct Options {
	options: lahl::Bucket
}

let result = Options::from_vec(vec!["--options", "a", "b", "c"]);
assert_eq!(result, Ok(Options {
	options: lahl::collections::Collection(
		vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
		Default::default()
	)
}));
```

#### By multiple flags

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct Command {
	value: lahl::Bucket
}

let result = Command::from_vec(vec!["--value", "abc"]);
assert_eq!(result.unwrap().value.0, vec!["abc".to_owned()]);
let result = Command::from_vec(vec!["--value", "abc", "--value", "123"]);
assert_eq!(result.unwrap().value.0, vec!["abc".to_owned(), "123".to_owned()]);
```

#### Comma-seperated

```rust
use lahl::FromArgs;

let result = lahl::Bucket::from_vec(vec!["a", ",", "b", ",", "c"]);
assert_eq!(
	result,
	Ok(
		lahl::collections::Collection(
			vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
			Default::default()
		)
	)
);
```

> Used in `cargo`

### Collect all

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	a: Option<String>,
	#[lahl(unknown_or_after_dash_dash)]
	collect: Option<lahl::collections::TrailingArguments>
}

let result = X::from_vec(vec!["--a", "b", "asdsasd"]);
assert_eq!(result, Ok(X {
	a: Some("b".to_owned()),
	collect: Some(
		lahl::collections::Collection(
			vec!["asdsasd".to_owned()],
			Default::default()
		)
	)
}));

let result = X::from_vec(vec!["--", "--a", "b", "asdsasd"]);
assert_eq!(result, Ok(X {
	a: None,
	collect: Some(
		lahl::collections::Collection(
			vec!["--a".to_owned(), "b".to_owned(), "asdsasd".to_owned()],
			Default::default()
		)
	)
}));
```

### Aliases / shorthand for struct fields

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	#[lahl(aliases("--b"))]
	a: Option<String>
}

let result = X::from_vec(vec!["--a", "b"]);
assert_eq!(result, Ok(X {
	a: Some("b".to_owned()),
}));

let result = X::from_vec(vec!["--b", "c"]);
assert_eq!(result, Ok(X {
	a: Some("c".to_owned()),
}));
```

### Aliases / shorthand for enums

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
enum X {
	#[lahl(aliases("c"))]
	A,
	B
}

let result = X::from_vec(vec!["a"]);
assert_eq!(result, Ok(X::A));

let result = X::from_vec(vec!["b"]);
assert_eq!(result, Ok(X::B));

let result = X::from_vec(vec!["c"]);
assert_eq!(result, Ok(X::A));
```

### Types

#### `()` type

> Mainly from generics

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	a: Option<String>,
	collect: ()
}

let result = X::from_vec(vec!["--a", "b"]);
assert_eq!(result, Ok(X {
	a: Some("b".to_owned()),
	collect: ()
}));

let result = X::from_vec(vec![]);
assert_eq!(result, Ok(X {
	a: None,
	collect: ()
}));
```

#### `Parseable`

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	a: lahl::Parseable<std::net::Ipv4Addr>,
}

let result = X::from_vec(vec!["--a", "127.0.0.1"]);
assert_eq!(result, Ok(X {
	a: lahl::Parseable(std::net::Ipv4Addr::new(127, 0, 0, 1)),
}));

let result = X::from_vec(vec!["--a", "aaa.0.0.1"]);
assert!(result.is_err());
```

#### `PathBuf`

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	a: std::path::PathBuf,
}

let result = X::from_vec(vec!["--a", "src/main.rs"]);
assert_eq!(result, Ok(X {
	a: std::path::PathBuf::from("src/main.rs"),
}));
```

#### More

#### `r#` keywords

```rust
use lahl::FromArgs;

#[derive(FromArgs, Debug, PartialEq)]
struct X {
	r#loop: bool,
}

let result = X::from_vec(vec!["--loop"]);
assert_eq!(result, Ok(X { r#loop: true, }));
```
