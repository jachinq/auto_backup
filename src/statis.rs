use crate::job::StartInfo;

#[derive(Debug, Default)]
pub struct Statis {
    pub total: usize,
    pub active: usize,
    pub archive: usize,
    pub auto_backup: usize,
    pub jobs: Vec<StartInfo>,
}
