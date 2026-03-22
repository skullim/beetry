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

A parameter is usually introduced by defining a serializable Rust type, and then
implementing `ProvideParamSpec` for that type.

For example, a node may accept a retry limit:

```rust
# extern crate anyhow;
# extern crate beetry;
# extern crate mitsein;
use std::sync::Arc;

use anyhow::anyhow;
use beetry::plugin::{
    FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec,
    ProvideParamSpec,
};
use mitsein::iter1::IntoIterator1;

#[derive(Debug, Clone)]
pub struct RetryParams {
    retry_limit: u64,
}

impl ProvideParamSpec for RetryParams {
    fn provide() -> ParamsSpec {
        [(
            FieldName::from("retry_limit"),
            FieldDefinition {
                type_spec: FieldTypeSpec::U64(FieldMetadata::new(Arc::new(|value| {
                    if *value == 0 {
                        Err(anyhow!("retry_limit must be greater than 0"))
                    } else {
                        Ok(())
                    }
                }))),
                description: Some("Maximum number of retry attempts".into()),
            },
        )]
        .into_iter1()
        .collect1()
    }
}
```

Above, we defined a parameter with a single field, gave it a name, assigned it a
type (`FieldTypeSpec::U64`), attached a custom validator, and provided a custom
description.
