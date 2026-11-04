use crate::{Arguments, BaseError, FromArgs, Parameter};

// struct CommaSeperatedList
// struct ExistsPath
// struct URL
// #[cfg] struct Glob

impl FromArgs for f32 {
	fn information() -> Parameter<'static> {
		Parameter::Leaf { optional: false, name: None }
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		args.next_arg::<Self>().and_then(|arg| arg.parse().map_err(|_| BaseError::ParseError))
	}
}
