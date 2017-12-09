# Changelog

This changelog was started with the 0.4.0 release, so there are no logs before
that version.

# Next

* Added method for requesting a type directly from the TOML document:
  The method returns the requested type directly, or fails with
  `Err(_)` and appropriate message:
  `document.read_string(path) -> Result<String, Error>` (for example)

* Added extension for `Result<>` for requesting a type directly from the TOML
  document, which filters the `Ok(_)` case for a type and translates the `Ok(_)`
  to either Ok if the type is correct or an `Err(_)` if the Type does not match:
  `document.read(path).as_type(Type::String) -> Result<Value, Error>`
  (for example)

# 0.4.0

* Updated the `error-chain` dependency from `0.10` to `0.11`.

