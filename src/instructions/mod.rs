pub mod arm;

#[derive(Debug)]
pub enum PipelineStatus {
    Flush,
    Continue,
}
