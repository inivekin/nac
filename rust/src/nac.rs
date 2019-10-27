use std::sync::{Arc, RwLock};

#[derive(Debug,Clone,Copy)]
pub struct Node<T> { pub data: T }


impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Self {
            data
        }
    }
}


pub trait InterResolve<T>: Fn(&Node<T>, &Node<T>) -> (Node<T>,Node<T>) {
    fn resolve(&self, primary_node: &Node<T>, secondary_node: &Node<T>) -> (Node<T>, Node<T>);
}

impl<T, F> InterResolve<T> for F where F: Fn(&Node<T>, &Node<T>) -> (Node<T>,Node<T>) {
    fn resolve(&self, primary_node: &Node<T>, secondary_node: &Node<T>) -> (Node<T>, Node<T>) {
        self(primary_node, secondary_node)
    }
}

pub trait SelfResolve<T>: Fn(&Node<T>) -> Node<T> {
    fn resolve(&self, node: &Node<T>) -> Node<T>;
}

impl<T, F> SelfResolve<T> for F where F: Fn(&Node<T>) -> Node<T> {
    fn resolve(&self, node: &Node<T>) -> Node<T> {
        self(node)
    }
}

pub type SharedNode<T> = Arc<RwLock<Node<T>>>;
pub type SharedNodes<T> = Vec<SharedNode<T>>;

pub struct InterConnector<T> {
    pub node: SharedNode<T>,
    pub constraints: Option<SharedNodes<T>>,
    relation: Arc<dyn InterResolve<T>>,
    // hashmap for custom properties depending on trait?
}

pub struct SelfConnector<T> {
    pub name: String,
    pub node: SharedNode<T>,
    relation: Arc<dyn SelfResolve<T>>,
}


// A connector can connect a node with other node(s) with a specific constraint function
impl<T> InterConnector<T> {
    pub fn new(node: SharedNode<T>, constraints: Option<SharedNodes<T>>, relation: Arc<dyn InterResolve<T>>) -> Self {
        Self {
            node,
            constraints,
            relation
        }
    }

    pub fn resolve(&self) {
        let resolver_relation = &self.relation;
        if let Some(constraints) = &self.constraints {
            constraints.iter().for_each(
                |constraint| {
                    let (updated_node, updated_constraint) = resolver_relation(&self.node.read().unwrap(), &constraint.read().unwrap());
                    if let Ok(mut node) = self.node.write() {
                        *node = updated_node
                    }
                    if let Ok(mut contraint) = constraint.write() {
                        *contraint = updated_constraint
                    }
                }
            );
        }
    }
}

impl<T> SelfConnector<T> {
    pub fn new(name: &str, node: SharedNode<T>, relation: Arc<dyn SelfResolve<T>>) -> Self {
        Self {
            name: name.to_owned(),
            node,
            relation
        }
    }

    pub fn resolve(&self) {
        let resolver_relation = &self.relation;
        let updated_node = resolver_relation(&self.node.read().unwrap());
        if let Ok(mut node) = self.node.write() {
            *node = updated_node
        }
    }
}



pub struct Mesh<T> {
    pub nodes: Vec<SharedNode<T>>,
    pub interconnectors: Vec<InterConnector<T>>,
    pub selfconnectors: Vec<SelfConnector<T>>,
}

impl<T> Mesh<T> {
    pub fn new(nodes: Vec<SharedNode<T>>, interconnectors: Vec<InterConnector<T>>, selfconnectors: Vec<SelfConnector<T>>) -> Self {
        Mesh {
            nodes,
            interconnectors,
            selfconnectors,
        }
    }

    pub fn update(&self, delta: f64, physics_accuracy: u8) {
        (0..physics_accuracy).for_each(|_i|
            {
            self.interconnectors.iter().for_each(|connector|
                connector.resolve());
           });
         self.selfconnectors.iter().for_each(|connector|
            connector.resolve());
    }
}

