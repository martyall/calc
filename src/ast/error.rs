use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Graph is empty")]
    CyclicDependency(Vec<String>),
}
