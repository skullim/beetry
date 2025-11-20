use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use beetry_serde::ser::node::{LeafSchema, LeafSpec, NodeName};
use beetry_serde::ser::parameter;

use crate::definitions::NodeId;

pub struct NodeSpecStorage;

impl NodeSpecStorage {
    fn leaf_spec() {}

    fn control_spec() {}
}

// not ideal to have NodeName (~String) in both maps, but self referential structs are discouraged
// one could implement new map to define NodeId NodeNameId mapping, but for now the less performant (but simpler) approach is chosen
// apparently slotmap crate can help with that too
type SpecMap = HashMap<NodeName, LeafSpec>;
type IdMap = HashMap<NodeId, NodeName>;

struct LeafSpecStorage {
    spec_map: RefCell<SpecMap>,
    id_map: RefCell<IdMap>,
}

impl LeafSpecStorage {
    fn new() -> Self {
        Self {
            spec_map: RefCell::new(SpecMap::new()),
            id_map: RefCell::new(IdMap::new()),
        }
    }

    fn store(&self, id: NodeId, spec: &LeafSpec) {
        // https://doc.rust-lang.org/reference/destructors.html#temporary-scopes
        // spec_map reference dropped once condition evaluated, safe to use hence no aliasing
        if !self.spec_map().contains_key(spec.name()) {
            self.spec_map_mut()
                .entry(spec.name().clone())
                .or_insert(spec.clone());
        }

        self.id_map_mut().insert(id, spec.name.clone());
    }

    fn spec(&self, id: NodeId) -> Option<Ref<'_, LeafSpec>> {
        let id_map_ref = self.id_map();
        let name = id_map_ref.get(&id)?;
        Ref::filter_map(self.spec_map(), |map| map.get(name)).ok()
    }

    fn remove(&self, id: NodeId) {
        self.id_map_mut().remove(&id);
    }

    fn spec_map_mut(&self) -> RefMut<'_, SpecMap> {
        self.spec_map.borrow_mut()
    }

    fn spec_map(&self) -> Ref<'_, SpecMap> {
        self.spec_map.borrow()
    }

    fn id_map_mut(&self) -> RefMut<'_, IdMap> {
        self.id_map.borrow_mut()
    }

    fn id_map(&self) -> Ref<'_, IdMap> {
        self.id_map.borrow()
    }
}

#[cfg(test)]
mod tests {
    use beetry_serde::ser::node::ActionLeafSchema;

    use super::*;

    #[test]
    fn test_spec_safe_reference() {
        let storage = LeafSpecStorage::new();
        let id = 42usize;
        let expected_spec_name = "test_spec";
        let spec = LeafSpec::builder()
            .name(expected_spec_name.to_string())
            .schema(ActionLeafSchema::builder().build())
            .build();

        storage.store(id, &spec);

        if let Some(spec_ref) = storage.spec(id) {
            let spec_name = spec_ref.name();
            assert_eq!(&spec_name.0, expected_spec_name);
            println!("Accessed spec name: {}", spec_name);
        } else {
            panic!("Should have found the spec");
        }
    }
}
