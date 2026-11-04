use crate::{Arguments, BaseError, FromArgs, Parameter, ValueOrHelp};

/// TODO allow commas
pub trait CollectionConfiguration {
	const ALLOW_EMPTY: bool = false;
	const BREAK_ON_FLAGS: bool = true;
	// const SINGLE: bool = false;
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct BucketConfig;
impl CollectionConfiguration for BucketConfig {}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct EmptyBucketConfig;
impl CollectionConfiguration for EmptyBucketConfig {
	const ALLOW_EMPTY: bool = true;
}

// #[derive(PartialEq, Debug, Clone, Default)]
// pub struct SingleBucketConfig;
// impl CollectionConfiguration for SingleBucketConfig {
// 	const ALLOW_EMPTY: bool = true;
// 	const SINGLE: bool = true;
// }

#[derive(PartialEq, Debug, Clone, Default)]
pub struct TrailingArgumentsConfig;
impl CollectionConfiguration for TrailingArgumentsConfig {
	const ALLOW_EMPTY: bool = true;
	const BREAK_ON_FLAGS: bool = false;
}

pub type Bucket = Collection<BucketConfig>;

/// Can contain no values
pub type EmptyBucket = Collection<EmptyBucketConfig>;
pub type TrailingArguments = Collection<TrailingArgumentsConfig>;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Collection<C: CollectionConfiguration>(pub Vec<String>, pub std::marker::PhantomData<C>);

impl<C: CollectionConfiguration> IntoIterator for Collection<C> {
	type Item = String;
	type IntoIter = <Vec<String> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<C: CollectionConfiguration> FromArgs for Collection<C> {
	fn information() -> Parameter<'static> {
		Parameter::Multiple
	}

	fn from_args_instep(args: &mut Arguments) -> Result<Self, BaseError> {
		let mut members = vec![];
		while let Some(arg) = args.next() {
			let arg: String = match arg {
				ValueOrHelp::Value(arg) => arg,
				ValueOrHelp::Help => {
					// TODO config
					if members.is_empty() {
						return Err(BaseError::Help);
					} else {
						"--help".to_owned()
					}
				}
			};
			if C::BREAK_ON_FLAGS && arg.starts_with("-") {
				args.reject(arg);
				break;
			} else if arg != "," {
				members.push(arg);
			}
			// if C::SINGLE {
			// 	break;
			// }
		}
		if members.is_empty() && !C::ALLOW_EMPTY {
			Err(BaseError::MissingValue)
		} else {
			Ok(Self(members, Default::default()))
		}
	}

	fn elided() -> Result<Self, BaseError> {
		if C::ALLOW_EMPTY {
			Ok(Self(Vec::new(), Default::default()))
		} else {
			Err(BaseError::MissingValue)
		}
	}

	fn append(&mut self, args: &mut Arguments) -> Result<(), BaseError> {
		if let Some(arg) = args.next() {
			let arg = match arg {
				ValueOrHelp::Value(arg) => arg,
				ValueOrHelp::Help => {
					return Err(BaseError::Help);
				}
			};
			self.0.push(arg);
			Ok(())
		} else {
			if C::ALLOW_EMPTY { Err(BaseError::MissingValue) } else { Ok(()) }
		}
	}
}
