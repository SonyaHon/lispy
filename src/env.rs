use std::collections::HashMap;
use crate::core_ns::apply_core_ns;
use crate::types::LispyType;

#[derive(Clone, Debug)]
pub struct LispyEnv {
    store: HashMap<String, LispyType>,
    parent: Option<Box<LispyEnv>>,
}

impl LispyEnv {
    pub fn root() -> Self {
        let mut this = Self {
            store: HashMap::new(),
            parent: None,
        };
        apply_core_ns(&mut this);
        this
    }

    pub fn child(parent: &mut LispyEnv) -> Self {
        Self {
            store: HashMap::new(),
            parent: Some(Box::from(parent.clone())),
        }
    }

    pub fn child_lambda(parent: Box<LispyEnv>) -> Self {
        Self {
            store: HashMap::new(),
            parent: Some(Box::from(parent.clone())),
        }
    }

    pub fn get_item(&self, key: &String) -> Option<&LispyType> {
        let result = self.store.get(key);

        if result.is_some() { return result; }
        if self.parent.is_some() { return self.parent.as_ref().unwrap().get_item(key); }
        None
    }

    pub fn set_item(&mut self, key: String, value: LispyType) {
        self.store.insert(key, value);
    }

    pub fn set(&mut self, key: &str, value: LispyType) {
        self.store.insert(key.to_string(), value);
    }
}