use std::sync::{Arc, RwLock};

use std::thread;

#[derive(Debug,Clone,Copy)]
pub struct Node<T: Sync + Send + 'static> { pub data: T }


impl<T: Sync + Send> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Self {
            data
        }
    }
}


pub trait InterResolve<T: Sync + Send + 'static>: Send + Sync + 'static + Fn(&Node<T>, &Node<T>) -> (Node<T>,Node<T>) {
    fn resolve(&self, primary_node: &Node<T>, secondary_node: &Node<T>) -> (Node<T>, Node<T>);
}

impl<T: Sync + Send + 'static, F> InterResolve<T> for F where F: Send + Sync + 'static + Fn(&Node<T>, &Node<T>) -> (Node<T>,Node<T>) {
    fn resolve(&self, primary_node: &Node<T>, secondary_node: &Node<T>) -> (Node<T>, Node<T>) {
        self(primary_node, secondary_node)
    }
}

pub trait SelfResolve<T: Sync + Send + 'static>: Send + Sync + 'static + Fn(&Node<T>) -> Node<T> {
    fn resolve(&self, node: &Node<T>) -> Node<T>;
}

impl<T: Sync + Send + 'static, F> SelfResolve<T> for F where F: Send + Sync + 'static + Fn(&Node<T>) -> Node<T> {
    fn resolve(&self, node: &Node<T>) -> Node<T> {
        self(node)
    }
}

pub type SharedNode<T> = Arc<RwLock<Node<T>>>;
pub type SharedNodes<T> = Vec<SharedNode<T>>;

#[derive(Clone)]
pub struct InterConnector<T: Sync + Send + 'static> {
    pub node: SharedNode<T>,
    pub constraints: Option<SharedNodes<T>>,
    relation: Arc<dyn InterResolve<T> + 'static>,
    // hashmap for custom properties depending on trait?
}

#[derive(Clone)]
pub struct SelfConnector<T: Sync + Send + 'static> {
    pub name: String,
    pub node: SharedNode<T>,
    relation: Arc<dyn SelfResolve<T> + 'static>,
}


// A connector can connect a node with other node(s) with a specific constraint function
impl<T: Sync + Send + 'static> InterConnector<T> {
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
                    } else {
                        panic!("Interconnector node update failed");
                    }
                    if let Ok(mut contraint) = constraint.write() {
                        *contraint = updated_constraint
                    } else {
                        panic!("Interconnector constraint update failed");
                    }
                }
            );
        }
    }
}

impl<T: Sync + Send + 'static> SelfConnector<T> {
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
        } else {
            panic!{"Selfconnector node resolve failed"}
        }
    }
}



pub struct Mesh<T: Sync + Send + 'static> {
    pub nodes: Vec<SharedNode<T>>,
    pub interconnectors: Vec<InterConnector<T>>,
    pub selfconnectors: Vec<SelfConnector<T>>,
}

impl<T: Sync + Send + 'static> Mesh<T> {
    pub fn new(nodes: Vec<SharedNode<T>>, interconnectors: Vec<InterConnector<T>>, selfconnectors: Vec<SelfConnector<T>>) -> Self {
        Mesh {
            nodes,
            interconnectors,
            selfconnectors,
        }
    }

    pub fn update(&self, delta: f64, physics_accuracy: u8) {
        (0..physics_accuracy).for_each(|_i| {
            self.interconnectors.iter().for_each(|connector|
                connector.resolve());
        });
        self.selfconnectors.iter().for_each(|connector|
            connector.resolve());
    }

}

