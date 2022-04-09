use core::fmt;
use std::error;

#[derive(Debug)]
enum Parent<T> {
    Vertex(usize),
    Outstanding(T)
}

#[derive(Debug)]
struct Vertex<T> {
    /// Value of the node
    value: T,
    /// Links to vertices who have at least
    /// one inbound child edge
    parents: Vec<Parent<T>>,
    /// Links to child vertices
    children: Vec<usize>
}

impl<T> Vertex<T> {
    fn new(value: T, parents: Vec<Parent<T>>, children: Vec<usize>) -> Self {
        Vertex { value, parents, children }
    }
}

impl<T> PartialEq for Vertex<T>
where
    T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}

impl <T> Eq for Vertex<T>
where
    T: Eq
{ }

#[derive(Debug)]
pub struct Graph<T> {
    /// Graph vertices
    vertices: Vec<Vertex<T>>,
}

impl<T> Graph<T>
where
    T: fmt::Debug + Clone + PartialEq
{
    pub fn new() -> Self {
        Graph { vertices: Vec::with_capacity(32) }
    }

    fn is_unique(&self, value: &T) -> bool {
        match self.vertices.iter().find(|&x| *value == x.value) {
            Some(_) => false,
            None => true
        }
    }

    fn find_parent_vertices(&self, depends: &[T]) -> Vec<Parent<T>> {
        let mut parents: Vec<Parent<T>> = vec![];
        let mut found: bool;
        for dep in depends {
            found = false;
            for (i, vert) in self.vertices.iter().enumerate() {
                if vert.value == *dep {
                    parents.push(Parent::Vertex(i));
                    found = true;
                    break;
                }
            }
            if !found {
                parents.push(Parent::Outstanding(dep.clone()));
            }
        }
        return parents;
    }

    fn link_children(&mut self, parents: &[Parent<T>]) {
        let idx = self.vertices.len();
        for p in parents {
            if let Parent::Vertex(i) = p {
                self.vertices[*i].children.push(idx);
            }
        }
    }

    fn find_child_vertices(&self, value: &T) -> Vec<(usize, usize)> {
        let mut children: Vec<(usize, usize)> = vec![];
        for (i, vert) in self.vertices.iter().enumerate() {
            for (j, p) in vert.parents.iter().enumerate() {
                if let Parent::Outstanding(name) = p {
                    if name == value {
                        children.push((i, j));
                    }
                }
            }
        }
        return children;
    }

    fn link_parents(&mut self, children: &Vec<(usize, usize)>) {
        let idx = self.vertices.len();
        for (i, j) in children {
            self.vertices[*i].parents[*j] = Parent::Vertex(idx);
        }
    }

    fn contains_duplicates(slice: &[T]) -> bool {
        (1..slice.len()).any(|i| slice[i..].contains(&slice[i - 1]))
    }

    /// Insert a vertex in the graph, supplying its dependencies
    ///
    /// Dependencies do not have to exist in the graph at the time of
    /// insertion but must have been inserted by the time [`self.dependencies_of`]
    /// is invoked.
    ///
    /// # Arguments
    ///
    /// * `value`- The value of the node to insert
    ///
    /// * `depends` - Values on which the node to be inserted depends. Must not
    ///               contain duplicates or `value`.
    ///
    pub fn insert(&mut self, value: T, depends: &[T]) -> Result<(), Box<dyn error::Error>> {
        if Graph::<T>::contains_duplicates(depends) {
            return Err(format!("Dependencies {:?} contains duplicates", depends).into());
        }
        if !self.is_unique(&value) {
            return Err(format!("Vertex {:?} already in graph", value).into());
        }
        if depends.contains(&value) {
            return Err(format!("Vertex {:?} cannot depend on itself", value).into());
        }

        let parents = self.find_parent_vertices(depends);
        self.link_children(&parents);
        let children = self.find_child_vertices(&value);
        self.link_parents(&children);
        let children = children.iter().map(|(i, _)| *i).collect();
        self.vertices.push(Vertex::new(value, parents, children));

        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.vertices
            .iter()
            .find(|&v| !v.parents
                        .iter()
                        .find(|&p| match p {
                            Parent::Outstanding(_) => true,
                            _ => false
                        })
                        .is_none())
            .is_none()
    }

    /// Return a list of dependencies for the supplied value
    pub fn dependencies_of(&self, value: &T) -> Result<Vec<T>, Box<dyn error::Error>> {
        if !self.is_complete() {
            return Err("Cannot search incomplete graph".into());
        }
        let vert = match self.vertices.iter().position(|v| v.value == *value) {
            Some(vert) => vert,
            None => return Err(format!("No vertex matches {:?}", value).into())
        };

        let mut to_traverse:  Vec<usize> = vec![vert];
        let mut traversed: Vec<usize> = Vec::with_capacity(16);
        let mut deps: Vec<T> = Vec::with_capacity(16);

        loop {
            if to_traverse.len() == 0 as usize {
                break;
            }
            let vert = &self.vertices[to_traverse.pop().unwrap()];
            let unique: Vec<usize> = vert.parents
                            .iter()
                            .map(|x| match x {
                                Parent::Vertex(x) => *x,
                                _ => panic!("Impossible")
                            })
                            .filter(|x| traversed
                                        .iter()
                                        .find(|&y| x == y)
                                        .is_none())
                            .collect();
            deps.extend_from_slice(unique
                                    .iter()
                                    .map(|&i| self.vertices[i].value.clone())
                                    .collect::<Vec<T>>()
                                    .as_slice());
            to_traverse.extend_from_slice(unique.as_slice());
            traversed.extend_from_slice(unique.as_slice());
        }
        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;

    #[test]
    fn single_dependency() -> Result<(), Box<dyn error::Error>> {
        let mut graph: Graph<&str> = Graph::new();
        graph.insert("CONFIG_DEPENDENT", &vec!["CONFIG_PARENT"])?;
        graph.insert("CONFIG_PARENT", &vec![])?;
        let deps = graph.dependencies_of(&"CONFIG_DEPENDENT")?;

        assert_eq!(deps, vec!["CONFIG_PARENT"]);
        let deps = graph.dependencies_of(&"CONFIG_PARENT")?;
        assert_eq!(deps.len(), 0);
        Ok(())
    }

    #[test]
    fn search_of_incomplete_graph_disallowed() -> Result<(), Box<dyn error::Error>> {
        let mut graph: Graph<&str> = Graph::new();
        graph.insert("CONFIG_TEST", &vec!["CONFIG_NEVER_INSERTED"])?;
        assert!(graph.dependencies_of(&"CONFIG_TEST").is_err());
        Ok(())
    }

    #[test]
    fn multiple_intra_independent_dependencies() -> Result<(), Box<dyn error::Error>> {
        let mut graph: Graph<&str> = Graph::new();
        let opts = ["CONFIG_TEST", "CONFIG_FIRST_ROOT", "CONFIG_SECOND_ROOT"];
        graph.insert(opts[1], &[])?;
        graph.insert(opts[2], &[])?;
        graph.insert(opts[0], &opts[1..=2])?;
        let deps = graph.dependencies_of(&"CONFIG_TEST")?;
        assert_eq!(deps, &opts[1..=2]);
        Ok(())
    }

    #[test]
    fn deep_dependency_tree() -> Result<(), Box<dyn error::Error>> {
        let mut graph: Graph<&str> = Graph::new();
        let mut opts: Vec<String> = Vec::with_capacity(10);
        for i in 0..opts.capacity() {
            opts.push(format!("TEST_CONFIG{}", i));
        }

        if let Some((last, rest)) = opts.split_last() {
            for (i, opt) in rest.iter().enumerate() {
                graph.insert(&opt, &[&opts[i + 1]])?;
            }
            graph.insert(last, &[])?;
        }
        else {
            assert!(false);
        }

        for (i, opt) in opts.iter().enumerate() {
            let deps = graph.dependencies_of(&opt.as_str())?;
            assert_eq!(deps, &opts[i + 1..opts.len()]);
        }

        Ok(())
    }

    #[test]
    fn disjointed_graph() -> Result<(), Box<dyn error::Error>> {
        let mut graph: Graph<&str> = Graph::new();
        let opts = ["CONFIG0", "CONFIG1", "CONFIG2", "CONFIG3"];
        graph.insert(&opts[0],&[&opts[1]])?;
        graph.insert(&opts[1], &[])?;
        graph.insert(&opts[2], &[&opts[3]])?;
        graph.insert(&opts[3], &[])?;
        let deps = graph.dependencies_of(&opts[0])?;
        assert_eq!(deps, &[opts[1]]);
        let deps = graph.dependencies_of(&opts[1])?;
        assert_eq!(deps.len(), 0 as usize);
        let deps = graph.dependencies_of(&opts[2])?;
        assert_eq!(deps, &[opts[3]]);
        let deps = graph.dependencies_of(&opts[3])?;
        assert_eq!(deps.len(), 0 as usize);
        Ok(())
    }

    #[test]
    fn vertex_cannot_depend_on_itself() {
        let mut graph: Graph<&str> = Graph::new();
        assert!(graph.insert(&"CONFIG_TEST", &[&"CONFIG_TEST"]).is_err())
    }

    #[test]
    fn dependencies_cannot_include_duplciates() {
        let mut graph: Graph<&str> = Graph::new();
        assert!(graph.insert(&"CONFIG_TEST", &[&"CONFIG_ANOTHER", &"CONFIG_ANOTHER"]).is_err());
    }
}
