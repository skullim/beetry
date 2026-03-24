use anyhow::{Result, bail};
use mitsein::btree_map1::BTreeMap1;
use mitsein::iter1::IntoIterator1;
use mitsein::iter1::IteratorExt;
use mitsein::string1::String1;
use std::collections::HashSet;
use std::sync::Arc;

pub type FieldName = String1;
pub type FieldDescription = String1;

#[derive(Debug, Clone)]
pub struct ParametersSpec {
    fields: BTreeMap1<FieldName, FieldSpec>,
}

impl ParametersSpec {
    /// It would be nicer to impl FromIterator but it does not currently allow to return an error.
    /// Here an error must be emitted if there are duplicated field names in the entries.
    pub fn new(iter: impl IntoIterator1<Item = FieldEntry>) -> Result<Self> {
        let mut iter = iter.into_iter1().multipeek();
        {
            let mut seen = HashSet::new();
            while let Some(entry) = iter.peek() {
                if !seen.insert(entry.name.clone()) {
                    bail!("duplicate parameter field: {}", &entry.name);
                }
            }
        }
        Ok(Self {
            fields: iter.into_iter1().map(FieldEntry::decompose).collect1(),
        })
    }

    pub fn get(&self, field_name: &FieldName) -> Option<&FieldSpec> {
        self.fields.get(field_name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FieldName, &FieldSpec)> {
        self.fields.iter1().into_iter()
    }

    pub fn merge(lhs: ParametersSpec, rhs: ParametersSpec) -> Result<ParametersSpec> {
        Self::new(lhs.into_iter().chain(rhs).try_into_iter1().unwrap())
    }
}

impl IntoIterator for ParametersSpec {
    type Item = FieldEntry;
    type IntoIter = std::iter::Map<
        std::collections::btree_map::IntoIter<FieldName, FieldSpec>,
        fn((FieldName, FieldSpec)) -> FieldEntry,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter().map(FieldEntry::from_pair)
    }
}

#[derive(Debug, Clone)]
pub struct FieldEntry {
    name: FieldName,
    spec: FieldSpec,
}

impl FieldEntry {
    pub fn new(name: FieldName, spec: FieldSpec) -> Self {
        Self { name, spec }
    }

    pub fn name(&self) -> &FieldName {
        &self.name
    }

    pub fn spec(&self) -> &FieldSpec {
        &self.spec
    }

    pub fn decompose(self) -> (FieldName, FieldSpec) {
        (self.name, self.spec)
    }

    fn from_pair((name, spec): (FieldName, FieldSpec)) -> Self {
        Self { name, spec }
    }
}

impl From<FieldEntry> for (FieldName, FieldSpec) {
    fn from(value: FieldEntry) -> Self {
        value.decompose()
    }
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    spec: FieldValidator,
    description: FieldDescription,
}

impl FieldSpec {
    pub fn new(spec: FieldValidator, description: FieldDescription) -> Self {
        Self { spec, description }
    }

    pub fn spec(&self) -> &FieldValidator {
        &self.spec
    }

    pub fn description(&self) -> &FieldDescription {
        &self.description
    }
}

#[derive(Debug, Clone)]
pub enum FieldValidator {
    Bool(BoolFieldValidator),
    U64(U64FieldValidator),
    I64(I64FieldValidator),
    F64(F64FieldValidator),
    String(StringFieldValidator),
}

type SharedValidationFn<T> = Arc<dyn Fn(&T) -> Result<()>>;

#[derive(Default, Clone)]
pub struct ValueValidator<T> {
    validation_fn: Option<SharedValidationFn<T>>,
}

impl<T> ValueValidator<T> {
    pub fn new(validation_fn: SharedValidationFn<T>) -> Self {
        Self {
            validation_fn: Some(validation_fn),
        }
    }

    pub fn validate(&self, val: &T) -> Result<()> {
        self.validation_fn
            .as_ref()
            .map_or(Ok(()), |func| (func)(val))
    }
}

impl<T> std::fmt::Debug for ValueValidator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueValidator")
            .field("validation_fn", &"{...}")
            .finish()
    }
}

pub type BoolFieldValidator = ValueValidator<bool>;
pub type U64FieldValidator = ValueValidator<u64>;
pub type I64FieldValidator = ValueValidator<i64>;
pub type F64FieldValidator = ValueValidator<f64>;
pub type StringFieldValidator = ValueValidator<String>;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, anyhow, bail};
    use mitsein::string1::String1;

    fn name(value: &str) -> FieldName {
        String1::try_from(value.to_string()).unwrap()
    }

    fn desc(value: &str) -> FieldDescription {
        String1::try_from(value.to_string()).unwrap()
    }

    fn u64_validator(max: u64) -> FieldValidator {
        let validator = ValueValidator::new(Arc::new(move |value: &u64| {
            if *value == 0 {
                bail!("timeout must be greater than 0");
            }
            if *value > max {
                bail!("timeout must be at most {max}");
            }
            Ok(())
        }));
        FieldValidator::U64(validator)
    }

    fn timeout_field_spec(max: u64) -> FieldSpec {
        FieldSpec::new(u64_validator(max), desc("Timer timeout in milliseconds"))
    }

    fn timer_param_specs(max: u64) -> Result<ParametersSpec> {
        ParametersSpec::new([FieldEntry::new(name("timeout"), timeout_field_spec(max))])
    }

    fn charging_param_specs(
        charging_timeout_spec: FieldSpec,
        standby_timeout_spec: FieldSpec,
        runtime_timeout_spec: FieldSpec,
    ) -> Result<ParametersSpec> {
        ParametersSpec::new([
            FieldEntry::new(name("charging_timeout"), charging_timeout_spec),
            FieldEntry::new(name("standby_timeout"), standby_timeout_spec),
            FieldEntry::new(name("runtime_timeout"), runtime_timeout_spec),
        ])
    }

    fn validate_u64_param(spec: &ParametersSpec, key: &str, value: u64) -> Result<()> {
        let key = name(key);
        let Some(field) = spec.get(&key) else {
            bail!("field not found: {key}");
        };

        match field.spec() {
            FieldValidator::U64(validator) => validator.validate(&value),
            _ => Err(anyhow!("field '{key}' is not a u64 timer")),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TimeoutParam {
        timeout: u64,
    }

    impl TimeoutParam {
        fn new(timeout: u64) -> Self {
            Self { timeout }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ChargingParams {
        charging_timeout: TimeoutParam,
        standby_timeout: TimeoutParam,
        runtime_timeout: TimeoutParam,
    }

    impl ChargingParams {
        fn new(
            charging_timeout: TimeoutParam,
            standby_timeout: TimeoutParam,
            runtime_timeout: TimeoutParam,
        ) -> Self {
            Self {
                charging_timeout,
                standby_timeout,
                runtime_timeout,
            }
        }
    }

    fn validate_timeout_param(spec: &ParametersSpec, key: &str, param: TimeoutParam) -> Result<()> {
        validate_u64_param(spec, key, param.timeout)
    }

    fn validate_charging_params(spec: &ParametersSpec, params: ChargingParams) -> Result<()> {
        validate_timeout_param(spec, "charging_timeout", params.charging_timeout)?;
        validate_timeout_param(spec, "standby_timeout", params.standby_timeout)?;
        validate_timeout_param(spec, "runtime_timeout", params.runtime_timeout)?;
        Ok(())
    }

    #[test]
    fn timer_param_enforces_upper_bound() {
        let spec = timer_param_specs(100).unwrap();

        assert!(validate_u64_param(&spec, "timeout", 10).is_ok());
        assert!(validate_u64_param(&spec, "timeout", 100).is_ok());
        assert!(validate_u64_param(&spec, "timeout", 0).is_err());
        assert!(validate_u64_param(&spec, "timeout", 101).is_err());
    }

    #[test]
    fn charging_params_reuses_timer_param() {
        let spec = charging_param_specs(
            timeout_field_spec(500),
            timeout_field_spec(500),
            timeout_field_spec(500),
        )
        .unwrap();

        assert!(validate_u64_param(&spec, "charging_timeout", 1).is_ok());
        assert!(validate_u64_param(&spec, "standby_timeout", 250).is_ok());
        assert!(validate_u64_param(&spec, "runtime_timeout", 500).is_ok());
        assert!(validate_u64_param(&spec, "runtime_timeout", 501).is_err());
    }

    #[test]
    fn charging_params_hierarchy_validates_against_spec() {
        let spec = charging_param_specs(
            timeout_field_spec(500),
            timeout_field_spec(500),
            timeout_field_spec(500),
        )
        .unwrap();

        let ok = ChargingParams::new(
            TimeoutParam::new(100),
            TimeoutParam::new(200),
            TimeoutParam::new(300),
        );
        assert!(validate_charging_params(&spec, ok).is_ok());

        let invalid = ChargingParams::new(
            TimeoutParam::new(100),
            TimeoutParam::new(0),
            TimeoutParam::new(300),
        );
        assert!(validate_charging_params(&spec, invalid).is_err());
    }
}

/// Hierarchical Parameter Schema
///
/// Main idea: Build AST for parameter specs.
/// There are two node kinds:
/// 1. Group - parameter container with optional context
/// 2. Leaf - one concrete named parameter with primitive data type (see ParameterValue)
///
/// Each parameter implements NodeSpecProvider where an associated node is added to the tree.
///
/// Workflow:
/// User defines param structs, most likely manually.
/// Spec structs are ideally generated from macro.
///
/// naming: ParamName + Spec suffix
/// Probably two macros needed for each node kind:
/// Leaf param field have to be mapped to corresponding FieldValidator + there is always FieldDescription
/// Group should have the same fields as ParamGroup but with Spec suffix. Assumption: Other specs should have been defined with consistent names
///
/// To be done:
/// 1. AST parser to flatten (get all leaf) registered param fields with correct context message.
/// 2. ParameterValue has to be extended to add Map variant. Parameter map (Name, ParameterValue) has to be dispatched correctly by taking groups into account
/// 3. Define macros to avoid boilerplate code when defining params and their specs.
mod prototype {
    use super::*;

    pub enum ParamNode {
        Group(BTreeMap1<FieldName, FieldBinding>),
        Leaf(FieldSpec),
    }

    pub struct FieldBinding {
        context: Option<String>,
        node: ParamNode,
    }

    impl FieldBinding {
        fn leaf(spec: FieldSpec) -> Self {
            Self {
                context: None,
                node: ParamNode::Leaf(spec),
            }
        }

        fn group(context: impl Into<String>, node: ParamNode) -> Self {
            Self {
                context: Some(context.into()),
                node,
            }
        }
    }

    impl ParamNode {
        pub fn leaf(spec: FieldSpec) -> Self {
            Self::Leaf(spec)
        }

        pub fn group(
            children: impl IntoIterator1<Item = (FieldName, FieldBinding)>,
        ) -> Result<Self> {
            let mut iter = children.into_iter1().multipeek();

            {
                let mut seen = std::collections::HashSet::new();
                while let Some((name, _)) = iter.peek() {
                    if !seen.insert(name.clone()) {
                        bail!("duplicate child field: {name}");
                    }
                }
            }

            Ok(Self::Group(iter.into_iter1().collect1()))
        }
    }

    trait NodeSpecProvider {
        fn node(&self) -> Result<ParamNode>;
    }

    fn name(value: &str) -> FieldName {
        String1::try_from(value.to_string()).unwrap()
    }

    #[derive(Clone)]
    struct TimeoutParamSpec {
        timeout_validator: U64FieldValidator,
        description: FieldDescription,
    }

    impl TimeoutParamSpec {
        fn new(timeout_validator: U64FieldValidator, description: FieldDescription) -> Self {
            Self {
                timeout_validator,
                description,
            }
        }
    }

    impl NodeSpecProvider for TimeoutParamSpec {
        fn node(&self) -> Result<ParamNode> {
            ParamNode::group([(
                name("timeout"),
                FieldBinding::leaf(FieldSpec::new(
                    FieldValidator::U64(self.timeout_validator.clone()),
                    self.description.clone(),
                )),
            )])
        }
    }

    #[derive(Clone)]
    struct ChargingParamsSpec {
        charging_timeout: TimeoutParamSpec,
        standby_timeout: TimeoutParamSpec,
        runtime_timeout: TimeoutParamSpec,
    }

    impl ChargingParamsSpec {
        fn new(
            charging_timeout: TimeoutParamSpec,
            standby_timeout: TimeoutParamSpec,
            runtime_timeout: TimeoutParamSpec,
        ) -> Self {
            Self {
                charging_timeout,
                standby_timeout,
                runtime_timeout,
            }
        }
    }

    impl NodeSpecProvider for ChargingParamsSpec {
        fn node(&self) -> Result<ParamNode> {
            ParamNode::group([
                (
                    name("charging_timeout"),
                    FieldBinding::group("Timeout while charging", self.charging_timeout.node()?),
                ),
                (
                    name("standby_timeout"),
                    FieldBinding::group("Timeout in standby", self.standby_timeout.node()?),
                ),
                (
                    name("runtime_timeout"),
                    FieldBinding::group("Timeout during runtime", self.runtime_timeout.node()?),
                ),
            ])
        }
    }
}
