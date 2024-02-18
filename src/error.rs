#[derive(Debug)]
pub enum Error {
  UnexpectedError,
  KeyOverflowError,
  ValueOverflowError,
}

impl std::convert::From<std::io::Error> for Error {
  fn from(_e: std::io::Error) -> Error {
      Error::UnexpectedError
  }
}
