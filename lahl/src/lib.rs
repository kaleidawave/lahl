#![doc = include_str!("../../README.md")]
pub mod collections;
pub mod extras;
pub mod help;

pub use collections::Bucket;

pub use lahl_derive::FromArgs;

pub enum ArgumentsSource {
	Env(std::env::Args),
	Dyn(Box<dyn Iterator<Item = String>>),
}

impl Iterator for ArgumentsSource {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Env(args) => args.next(),
			Self::Dyn(args) => args.next(),
		}
	}
}

pub struct Arguments {
	source: ArgumentsSource,
	holding: Option<String>,
	/// When encountered an unknown value or `--help`. At the end if excess: print this as an unknown ?
	pub location: Option<Parameter<'static>>,
}

pub enum ValueOrHelp {
	Value(String),
	Help,
}

impl Arguments {
	// TODO cfg
	pub fn args() -> Self {
		let mut args = std::env::args();
		let _file = args.next();
		Self { source: ArgumentsSource::Env(args), holding: None, location: None }
	}

	pub fn reject(&mut self, value: String) {
		if let Some(ref holding) = self.holding {
			panic!("already holding {holding:?}");
		}
		self.holding = Some(value);
	}

	pub fn next(&mut self) -> Option<ValueOrHelp> {
		self.holding
			.take()
			.or_else(|| self.source.next())
			.map(|arg| if arg == "--help" { ValueOrHelp::Help } else { ValueOrHelp::Value(arg) })
	}

	pub fn next_arg<T: FromArgs>(&mut self) -> Result<String, BaseError> {
		match self.next() {
			Some(ValueOrHelp::Value(value)) => Ok(value),
			Some(ValueOrHelp::Help) => {
				self.location = Some(T::information());
				Err(BaseError::Help)
			}
			None => {
				self.location = Some(T::information());
				Err(BaseError::MissingValue)
			}
		}
	}

	// WIP
	pub fn next_arg2<T: FromArgs>(&mut self) -> Result<Option<String>, BaseError> {
		match self.next() {
			Some(ValueOrHelp::Value(value)) => {
				if value.starts_with("--") {
					self.reject(value);
					Ok(None)
				} else {
					Ok(Some(value))
				}
			}
			Some(ValueOrHelp::Help) => {
				self.location = Some(T::information());
				Err(BaseError::Help)
			}
			None => Ok(None),
		}
	}

	pub fn key<T: FromArgs>(&mut self) -> Option<ValueOrHelp> {
		let value = self.next()?;
		match value {
			ValueOrHelp::Value(value) => {
				if value.starts_with('-')
					&& let Some((before, after)) = value.split_once('=')
				{
					let _ = self.holding.insert(after.to_owned());
					Some(ValueOrHelp::Value(before.to_owned()))
				} else {
					Some(ValueOrHelp::Value(value))
				}
			}
			ValueOrHelp::Help => {
				self.location = Some(T::information());
				Some(ValueOrHelp::Help)
			}
		}
	}

	// TODO cfg
	pub fn into_vec(self) -> Vec<String> {
		self.source.into_iter().collect()
	}
}

// pub type Parameters<'a> = &'a [Parameter<'a>];

/// Also for naned fields
#[derive(Debug, PartialEq)]
pub struct Branch<'a> {
	pub key: &'a str,
	pub aliases: Vec<&'a str>,
	pub description: &'a str,
	pub result: Parameter<'a>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NameDescription<'a> {
	pub name: &'a str,
	pub description: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum Parameter<'a> {
	Group {
		name: Option<NameDescription<'a>>,
		unnamed: Vec<Parameter<'a>>,
		named: Vec<Branch<'a>>,
	},
	Switch {
		name: Option<NameDescription<'a>>,
		branches: Vec<Branch<'a>>,
	},
	Leaf {
		name: Option<NameDescription<'a>>,
		optional: bool,
	},
	// TODO merge with leaf
	/// For bucket.
	Multiple,
	/// From unit structs etc
	Unit,
}

impl<'a> Parameter<'a> {
	pub fn get_name(&self) -> Option<NameDescription<'a>> {
		match self {
			Parameter::Group { name, .. }
			| Parameter::Switch { name, .. }
			| Parameter::Leaf { name, .. } => name.clone(),
			_ => None,
		}
	}
}

pub trait FromArgs: Sized {
	// Printable information about this and how to form.
	fn information() -> Parameter<'static>;

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError>;

	fn from_env_args() -> Result<Self, Error> {
		Self::from_args(Arguments::args())
	}

	/// For testing
	fn from_vec(vec: Vec<&'static str>) -> Result<Self, Error> {
		Self::from_args(Arguments {
			holding: None,
			source: ArgumentsSource::Dyn(Box::new(vec.into_iter().map(str::to_owned))),
			location: None,
		})
	}

	fn from_args(mut args: Arguments) -> Result<Self, Error> {
		let value = Self::from_args_instep(&mut args);
		match value {
			Ok(value) => {
				match args.next() {
					Some(ValueOrHelp::Value(excess)) => {
						// TODO could be self?
						Err(Error { base: BaseError::Excess(excess), location: args.location })
					}
					Some(ValueOrHelp::Help) => {
						Err(Error { base: BaseError::Help, location: dbg!(args.location) })
					}
					None => Ok(value),
				}
			}
			Err(base) => Err(Error { base, location: args.location }),
		}
	}

	fn elided() -> Result<Self, BaseError> {
		Err(BaseError::MissingValue)
	}

	fn append(&mut self, _args: &mut Arguments) -> Result<(), BaseError> {
		Err(BaseError::Duplicate)
	}
}

pub trait Formable {
	type Partial: Partial;

	fn new() -> Self::Partial {
		<Self::Partial as Partial>::new()
	}

	fn will_eat(name: &str) -> bool {
		<Self::Partial as Partial>::will_eat(name)
	}
}

pub trait Partial {
	type Out;

	fn will_eat(name: &str) -> bool;

	fn new() -> Self;

	fn set(&mut self, key: &str, args: &mut Arguments) -> Result<(), PartialSetError>;

	fn finish(self) -> Result<Self::Out, BaseError>;
}

/// TODO injest PartialFinishError
#[derive(Debug, PartialEq)]
pub enum BaseError {
	MissingValue,
	ParseError,
	Duplicate,
	Missing(String),
	Excess(String),
	Help,
	/// From `enum`
	Unknown(String),
}

#[derive(Debug, PartialEq)]
pub enum PartialSetError {
	Inner(BaseError),
	Unknown(String),
}

impl From<BaseError> for PartialSetError {
	fn from(base: BaseError) -> Self {
		Self::Inner(base)
	}
}

#[derive(Debug, PartialEq)]
pub enum PartialFinishError {
	/// TODO as array
	Missing(String),
}

#[derive(Debug, PartialEq)]
pub struct Error {
	pub base: BaseError,
	pub location: Option<Parameter<'static>>,
}

impl FromArgs for String {
	fn information() -> Parameter<'static> {
		Parameter::Leaf {
			optional: false,
			name: Some(NameDescription { name: "string", description: "argument verbatim" }),
		}
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		args.next_arg::<Self>()
	}
}

impl FromArgs for std::path::PathBuf {
	fn information() -> Parameter<'static> {
		Parameter::Leaf {
			optional: false,
			name: Some(NameDescription { name: "path", description: "a path" }),
		}
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		args.next_arg::<Self>().map(|arg| std::path::PathBuf::from(arg))
	}
}

impl FromArgs for usize {
	fn information() -> Parameter<'static> {
		Parameter::Leaf {
			optional: false,
			name: Some(NameDescription { name: "usize", description: "unsigned integer" }),
		}
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		args.next_arg::<Self>().and_then(|arg| arg.parse().map_err(|_| BaseError::ParseError))
	}
}

impl<T: FromArgs, U: FromArgs> FromArgs for (T, U) {
	fn information() -> Parameter<'static> {
		Parameter::Group {
			name: None, // Some(NameDescription { name: "pair", description: "" }),
			unnamed: vec![T::information(), U::information()],
			named: Vec::new(),
		}
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		let t = T::from_args_instep(args)?;
		let u = U::from_args_instep(args)?;
		Ok((t, u))
	}
}

impl FromArgs for bool {
	fn elided() -> Result<Self, BaseError> {
		Ok(false)
	}

	fn information() -> Parameter<'static> {
		Parameter::Leaf {
			optional: false,
			name: Some(NameDescription { name: "boolean", description: "flag" }),
		}
	}

	fn from_args_instep(_args: &mut Arguments) -> Result<Self, BaseError> {
		Ok(true)
	}
}

impl FromArgs for () {
	fn elided() -> Result<Self, BaseError> {
		Ok(())
	}

	fn information() -> Parameter<'static> {
		Parameter::Unit
	}

	fn from_args_instep(_args: &mut Arguments) -> Result<Self, BaseError> {
		Ok(())
	}
}

impl<T: FromArgs> FromArgs for Option<T> {
	fn information() -> Parameter<'static> {
		Parameter::Leaf { optional: true, name: T::information().get_name() }
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		let inner = T::from_args_instep(args);
		match inner {
			Ok(value) => Ok(Some(value)),
			Err(BaseError::MissingValue) => Ok(None),
			Err(err) => Err(err),
		}
	}

	fn elided() -> Result<Self, BaseError> {
		Ok(None)
	}

	// TODO
	fn append(&mut self, args: &mut Arguments) -> Result<(), BaseError> {
		if let Some(this) = self {
			this.append(args)
		} else {
			*self = Some(T::from_args_instep(args)?);
			Ok(())
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Parseable<T: std::str::FromStr>(pub T);

impl<T: std::str::FromStr> FromArgs for Parseable<T> {
	fn information() -> Parameter<'static> {
		Parameter::Leaf { optional: false, name: None }
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		args.next_arg::<Self>().and_then(|arg| match arg.parse() {
			Ok(v) => Ok(Self(v)),
			Err(_) => Err(BaseError::ParseError),
		})
	}
}

impl<T: std::str::FromStr> Parseable<T> {
	pub fn value(self) -> T {
		self.0
	}
}

pub static EMPTY: &str = "";

/// checks for things ahead of time
pub fn check(_parameters: Parameter<'_>) {}

impl Error {
	pub fn print_with_exit_code(self) -> std::process::ExitCode {
		self.print();
		std::process::ExitCode::FAILURE
	}

	pub fn print(self) {
		match self.base {
			BaseError::MissingValue => {
				eprintln!("missing value");
			}
			BaseError::ParseError => {
				eprintln!("parse error");
			}
			BaseError::Duplicate => {
				eprintln!("parse error");
			}
			BaseError::Missing(missing) => {
				eprintln!("missing {missing:?}");
			}
			BaseError::Excess(excess) => {
				eprintln!("unexpected {excess:?} (did not get passed)");
			}
			BaseError::Help => {
				assert!(self.location.is_some(), "--help but no location");
			}
			// From `enum` etc
			BaseError::Unknown(arg) => {
				eprintln!("unknown {arg:?}");
				assert!(self.location.is_some(), "--help but no location");
			}
		}
		if let Some(location) = self.location {
			help::print_information(location);
		}
	}
}
