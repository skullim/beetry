use crate::Indices;
use crate::control::RunningNodesAborter;
use beetry_core::{Node, NonEmptyNodes, TickStatus};
use bon::Builder;

#[derive(Debug, Clone, Copy, Builder)]
#[cfg_attr(feature = "registry", derive(serde::Deserialize))]
pub struct ParallelThresholdParams {
    pub success_count: u16,
    pub failure_count: u16,
}

pub struct ParallelThreshold {
    nodes: NonEmptyNodes,
    aborter: RunningNodesAborter,
    params: ParallelThresholdParams,
}

impl ParallelThreshold {
    #[must_use]
    pub fn new(nodes: NonEmptyNodes, params: ParallelThresholdParams) -> Self {
        Self {
            nodes,
            aborter: RunningNodesAborter::new(),
            params,
        }
    }
}

impl Node for ParallelThreshold {
    fn tick(&mut self) -> TickStatus {
        let aborter: &mut RunningNodesAborter = &mut self.aborter;
        let mut success_count = 0u16;
        let mut failure_count = 0u16;

        for idx in self.nodes.indices() {
            let node = &mut self.nodes[idx];
            match node.tick() {
                TickStatus::Success => {
                    success_count += 1;
                    aborter.untrack(idx);
                }
                TickStatus::Running => {
                    aborter.track(idx);
                }
                TickStatus::Failure => {
                    failure_count += 1;
                    aborter.untrack(idx);
                }
            }
        }

        if failure_count >= self.params.failure_count {
            aborter.abort_all(&mut self.nodes);
            return TickStatus::Failure;
        }

        if success_count >= self.params.success_count {
            aborter.abort_all(&mut self.nodes);
            return TickStatus::Success;
        }

        if aborter.is_any_tracked() {
            TickStatus::Running
        } else {
            TickStatus::Failure
        }
    }

    fn abort(&mut self) {
        self.aborter.clear();
        for node in &mut self.nodes {
            node.abort();
        }
    }

    fn reset(&mut self) {
        self.aborter.clear();
        for node in &mut self.nodes {
            node.reset();
        }
    }
}

#[cfg(feature = "registry")]
mod registry_support {
    use super::ParallelThresholdParams;
    use anyhow::{Error, anyhow};
    use beetry_editor_types::output::node::Parameters;
    use beetry_editor_types::spec::node::{
        FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec, ProvideParamSpec,
    };
    use beetry_reconstruction_types::params::ParamsReconstructor;
    use mitsein::iter1::IntoIterator1;
    use std::sync::Arc;

    impl ProvideParamSpec for ParallelThresholdParams {
        fn provide() -> ParamsSpec {
            [
                (
                    FieldName::from("success_count"),
                    FieldDefinition {
                        type_spec: FieldTypeSpec::U16(FieldMetadata::new(Arc::new(
                            Self::validate_success_count,
                        ))),
                        description: Some(
                            "Number of successful children required to succeed".into(),
                        ),
                    },
                ),
                (
                    FieldName::from("failure_count"),
                    FieldDefinition {
                        type_spec: FieldTypeSpec::U16(FieldMetadata::new(Arc::new(
                            Self::validate_failure_count,
                        ))),
                        description: Some("Number of failed children required to fail".into()),
                    },
                ),
            ]
            .into_iter1()
            .collect1()
        }
    }

    impl ParallelThresholdParams {
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn validate_success_count(value: &u16) -> Result<(), Error> {
            if *value == 0 {
                return Err(anyhow!("success_count must be greater than 0"));
            }
            Ok(())
        }

        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn validate_failure_count(value: &u16) -> Result<(), Error> {
            if *value == 0 {
                return Err(anyhow!("failure_count must be greater than 0"));
            }
            Ok(())
        }

        pub(crate) fn reconstruct(parameters: Parameters) -> Result<Self, Error> {
            let params: Self = ParamsReconstructor::reconstruct(parameters).map_err(|err| {
                anyhow!("failed to reconstruct ParallelThreshold parameters: {err}")
            })?;
            Ok(params)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_test::{boxed, mock_returns};

    fn params(success_count: u16, failure_count: u16) -> ParallelThresholdParams {
        ParallelThresholdParams::builder()
            .success_count(success_count)
            .failure_count(failure_count)
            .build()
    }

    #[test]
    fn succeeds_on_success_threshold() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = ParallelThreshold::new(nodes, params(2, 3));

        assert_eq!(pl.tick(), TickStatus::Success);
    }

    #[test]
    fn fails_on_failure_threshold() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Success])),
        ]);
        let mut pl = ParallelThreshold::new(nodes, params(3, 2));

        assert_eq!(pl.tick(), TickStatus::Failure);
    }

    #[test]
    fn returns_running_when_no_threshold_reached_and_any_running() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Running])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = ParallelThreshold::new(nodes, params(2, 2));

        assert_eq!(pl.tick(), TickStatus::Running);
    }

    #[test]
    fn fails_when_all_terminal_and_no_threshold_reached() {
        let nodes = NonEmptyNodes::from([
            boxed(mock_returns([TickStatus::Success])),
            boxed(mock_returns([TickStatus::Failure])),
            boxed(mock_returns([TickStatus::Failure])),
        ]);
        let mut pl = ParallelThreshold::new(nodes, params(3, 3));

        assert_eq!(pl.tick(), TickStatus::Failure);
    }

    #[test]
    fn aborts_tracked_nodes_when_threshold_reached() {
        let mut m1 = mock_returns([TickStatus::Running]);
        let m2 = mock_returns([TickStatus::Success]);
        let m3 = mock_returns([TickStatus::Success]);

        m1.expect_abort().once().return_const(());

        let nodes = NonEmptyNodes::from([boxed(m1), boxed(m2), boxed(m3)]);
        let mut pl = ParallelThreshold::new(nodes, params(2, 3));

        assert_eq!(pl.tick(), TickStatus::Success);
    }
}
