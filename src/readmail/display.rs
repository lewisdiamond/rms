use std::fmt;

impl fmt::Display for OutputTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not figure out output format")
    }
}

// This is important for other errors to wrap this one.
impl std::error::Error for OutputTypeError {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl std::str::FromStr for OutputType {
    type Err = OutputTypeError;
    fn from_str(input: &str) -> Result<OutputType, Self::Err> {
        match input.to_lowercase().as_str() {
            "short" => Ok(OutputType::Short),
            "full" => Ok(OutputType::Full),
            "raw" => Ok(OutputType::Raw),
            "html" => Ok(OutputType::Html),
            "summary" => Ok(OutputType::Summary),
            _ => Err(OutputTypeError::UnknownTypeError),
        }
    }
}
impl fmt::Display for OutputType{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            OutputType::Summary => "Summary",
            OutputType::Full => "Full",
            OutputType::Html => "Html",
            OutputType::Raw => "Raw",
            OutputType::Short  => "Short",
        };
        write!(f,"{}", msg)
    }
}

#[derive(Debug)]
pub enum OutputTypeError {
    UnknownTypeError,
}

#[derive(Debug)]
pub enum OutputType {
    Summary,
    Short,
    Full,
    Raw,
    Html,
}


pub trait DisplayAs {
    fn display(&self, t: &OutputType) -> String;
}
