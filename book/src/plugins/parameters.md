# Parameters

Nodes often need additional input at construction time.

To support this, Beetry defines two main design goals for such inputs:

- they must be serializable and deserializable, so they can survive persistence
- they must support validation, so incorrect values can be detected as early as
  possible

Beetry addresses this with `Parameter`. A `Parameter` describes the input value
expected by a node using an enum that models the primitive types. A `Parameter`
can also attach an optional validator that checks additional constraints beyond
the basic value type.

This gives nodes part of the flexibility of ordinary constructors, but in a form
that remains structured and inspectable by the framework.

## Registration

A parameter is usually introduced by defining a serializable Rust type, and then
deriving a `ParamsSpec` for that type.

For example, a node may accept a retry limit:

```rust, no_run
use std::sync::Arc;

use anyhow::anyhow;
use beetry::plugin::{
    FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec,
    ProvideParamSpec,
};
use mitsein::iter1::IntoIterator1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

This example shows both parts of parameter definition:

- `RetryParams` is the typed value that will later be deserialized and passed to
  the node
- `ProvideParamSpec` publishes the editor-facing schema for that value

The field type, here `FieldTypeSpec::U64`, defines the basic accepted value
kind. The validator attached through `FieldMetadata::new(...)` adds an extra
constraint, in this case requiring `retry_limit` to be greater than zero.

This separation is important. The type tells the framework how to store and
reconstruct the value, while the validator allows domain-specific correctness
checks to be enforced early.
