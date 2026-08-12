# LAHL - lightweight argument handling library

Features

- derive macro
- depends on syraise (no syn)

## Derive

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

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
enum WithFallthrough {
	Main,
	#[fall_through]
	Else,
}

let result = WithFallthrough::from_vec(vec!["main"]);
assert_eq!(result, Ok(WithFallthrough::Main));
let result = WithFallthrough::from_vec(vec![]);
assert_eq!(result, Ok(WithFallthrough::Else));
```

Missing errors

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

Unit structs

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct UnitStruct;

let result = UnitStruct::from_vec(vec![]);
assert_eq!(result, Ok(UnitStruct));
```

Unnamed

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct UnnamedField(String);

let result = UnnamedField::from_vec(vec!["hi"]);
assert_eq!(result, Ok(UnnamedField("hi".to_owned())));
```

Named

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedFields {
	a: String,
	b: u32,
}

let result = NamedFields::from_vec(vec!["--a", "hi", "--b", "67"]);
assert_eq!(result, Ok(NamedFields { a: "hi".to_owned(), b: 67 }));

let result = NamedFields::from_vec(vec!["--b", "67", "--a", "hi"]);
assert_eq!(result, Ok(NamedFields { a: "hi".to_owned(), b: 67 }));
```

Enum variant with named variant

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

Named with optional fields. 

> Optional-ness is defined based on traits (so semantic over syntactic)

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedWithOptionalFields {
	optional_value: Option<String>,
	optional_flag: bool,
}

let result = NamedWithOptionalFields::from_vec(vec![]);
assert_eq!(result, Ok(NamedWithOptionalFields { optional_value: None, optional_flag: false }));

let result = NamedWithOptionalFields::from_vec(vec!["--optional-value", "hi"]);
assert_eq!(result, Ok(NamedWithOptionalFields { optional_value: Some("hi".into()), optional_flag: false }));

let result = NamedWithOptionalFields::from_vec(vec!["--optional-flag"]);
assert_eq!(result, Ok(NamedWithOptionalFields { optional_value: None, optional_flag: true }));
```

Equal based argument

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct NamedWithOptionalFields {
	optional_value: Option<String>,
	optional_flag: bool,
}

let result = NamedWithOptionalFields::from_vec(vec!["--optional-value=hi"]);
assert_eq!(result, Ok(NamedWithOptionalFields { optional_value: Some("hi".into()), optional_flag: false }));
```

Positional in named

> This can be `Args(String, *struct Flag { flag: bool }*)`, but this provides an easy escape hatch

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct WithPositional {
	#[positional]
	owned: String,
	flag: bool,
}

let result = WithPositional::from_vec(vec!["hi"]);
assert_eq!(result, Ok(WithPositional { owned: "hi".to_owned(), flag: false }));

let result = WithPositional::from_vec(vec!["hi", "--flag"]);
assert_eq!(result, Ok(WithPositional { owned: "hi".to_owned(), flag: true }));
```

Inner groups. Breaks the hierarchy

```rust
use lahl::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
struct WithGroup {
	owned: String,
	#[group]
	inner: Inner,
}

#[derive(FromArgs, PartialEq, Debug)]
struct Inner {
	value: Option<String>,
	flag: bool,
}

let result = WithGroup::from_vec(vec!["--owned=hi"]);
assert_eq!(
	result,
	Ok(WithGroup { owned: "hi".to_owned(), inner: Inner { value: None, flag: false } })
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
	Ok(WithGroup { owned: "hi".to_owned(), inner: Inner { value: None, flag: true } })
);
```
