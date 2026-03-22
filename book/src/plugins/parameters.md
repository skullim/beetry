# Parameters

Nodes often need additional input at construction time.

To support this, Beetry defines two main design goals for such inputs:

- they must be serializable and deserializable, so they can survive persistence
- they must support validation, so incorrect values can be detected as early as
  possible

Beetry addresses this with parameters. Parameter values are described through
typed field specifications, which can also attach optional validators for
constraints beyond the basic value type.

This gives nodes part of the flexibility of ordinary constructors, but in a form
that remains structured and inspectable by the framework.

## Registration

A parameter is usually introduced by defining a deserializable Rust type, and
then implementing `ProvideParamSpec` for that type.

For the most up-to-date minimal example, check the `ProvideParamSpec` API docs
in the `beetry::plugin` module.
