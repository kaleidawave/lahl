#![doc = include_str!("../../README.md")]

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

pub struct ArgumentsIter {
	args: ArgumentsSource,
	holding: Option<String>,
}

impl ArgumentsIter {
	// TODO cfg
	pub fn args() -> Self {
		let mut args = std::env::args();
		let _file = args.next();
		Self { args: ArgumentsSource::Env(args), holding: None }
	}

	// TODO or `--help`?
	pub fn next(&mut self) -> Option<String> {
		self.holding.take().or_else(|| self.args.next())
	}

	pub fn key(&mut self) -> Option<String> {
		let value = self.next()?;
		if value.starts_with('-')
			&& let Some((before, after)) = value.split_once('=')
		{
			let _ = self.holding.insert(after.to_owned());
			Some(before.to_owned())
		} else {
			Some(value)
		}
	}
}

pub type Parameters<'a> = &'a [Parameter<'a>];

pub trait FromArgs: Sized {
	// Printable information about this and how to form.
	// TODO return Parameters | Leaf
	fn information() -> Parameters<'static>;

	fn from_env_args() -> Result<Self, Error> {
		Self::from_args(ArgumentsIter::args())
	}

	/// For testing
	fn from_vec(vec: Vec<&'static str>) -> Result<Self, Error> {
		Self::from_args(ArgumentsIter {
			holding: None,
			args: ArgumentsSource::Dyn(Box::new(vec.into_iter().map(str::to_owned))),
		})
	}

	fn from_args(mut args: ArgumentsIter) -> Result<Self, Error> {
		let value = Self::from_args_instep(&mut args)?;
		if let Some(value) = args.next() { Err(Error::Excess(value)) } else { Ok(value) }
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error>;

	// TODO append for Vec
	fn elided() -> Result<Self, Error> {
		Err(Error::MissingValue)
	}
}

pub trait Formable {
	type Partial: Partial;

	fn new() -> Self::Partial {
		<Self::Partial as Partial>::new()
	}

	fn options() -> &'static [&'static str] {
		<Self::Partial as Partial>::options()
	}
}

pub trait Partial {
	type Out;

	fn options() -> &'static [&'static str];

	fn new() -> Self;

	fn set(&mut self, key: &str, args: &mut ArgumentsIter) -> Result<(), Error>;

	fn finish(self) -> Result<Self::Out, Error>;
}

#[derive(Debug, PartialEq)]
pub enum Error {
	Unknown(String, &'static [&'static str]),
	MissingValue,
	ParseError,
	Missing(String),
	Excess(String),
}

impl FromArgs for String {
	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error> {
		args.next().ok_or(Error::MissingValue)
	}
}

impl FromArgs for std::path::PathBuf {
	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error> {
		if let Some(arg) = args.next() {
			// TODO exists?
			Ok(std::path::PathBuf::from(arg))
		} else {
			Err(Error::MissingValue)
		}
	}
}

impl FromArgs for u32 {
	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error> {
		if let Some(arg) = args.next() {
			arg.parse().map_err(|_| Error::ParseError)
		} else {
			Err(Error::MissingValue)
		}
	}
}

impl FromArgs for f32 {
	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error> {
		if let Some(arg) = args.next() {
			arg.parse().map_err(|_| Error::ParseError)
		} else {
			Err(Error::MissingValue)
		}
	}
}

impl FromArgs for bool {
	fn elided() -> Result<Self, Error> {
		Ok(false)
	}

	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(_args: &mut ArgumentsIter) -> Result<Self, Error> {
		Ok(true)
	}
}

impl<T: FromArgs> FromArgs for Option<T> {
	fn information() -> Parameters<'static> {
		// TODO leaf::Leaf { optional: false }
		&[]
	}

	fn from_args_instep(args: &mut ArgumentsIter) -> Result<Self, Error> {
		// TODO if not empty -> then some
		T::from_args_instep(args).map(Some)
	}

	fn elided() -> Result<Self, Error> {
		Ok(None)
	}
}

// #[derive(Debug)]
// pub struct Switch<'a> {
// 	name: &'a str,
// 	description: &'a str,
// 	branches: &'a [Branch<'a>],
// }
//
// ---

#[derive(Debug)]
pub enum Parameter<'a> {
	Switch {
		name: &'a str,
		description: &'a str,
		branches: &'a [Branch<'a>],
	},
	Group {
		name: &'a str,
		description: &'a str,
		parameters: Parameters<'a>,
	},
	Boolean {
		// TODO shorthand
		name: &'a str,
		description: &'a str,
	},
	KeyValue {
		// TODO shorthand
		name: &'a str,
		required: bool,
		description: &'a str,
		multiple: bool,
	},
	Multiple {
		// `*preposition*` or none
		// (on, with, in, after, at, by, for, into, under) or --
		name: &'a str,
		description: &'a str,
		required: bool,
	},
}

#[derive(Debug)]
pub struct Branch<'a> {
	pub prefix: Option<&'a str>,
	/// this is the returned value
	pub key: &'a str,
	pub description: &'a str,
	pub aliases: &'a [&'a str],
	pub result: Parameters<'a>,
}

// ---

/// TODO <W>
pub fn print_information(parameters: Parameters<'_>) {
	println!("{parameters:#?}");
	// let mut branch =
}

pub fn check(_parameters: Parameters<'_>) {}
