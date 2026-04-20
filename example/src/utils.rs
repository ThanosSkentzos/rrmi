use std::env;

use gethostname::gethostname;
#[derive(Debug)]
pub struct Utils {
    pub my_hostname: String,
    pub nodes: Vec<String>,
    pub coordinator: String,
}
impl Utils {
    pub fn new() -> Self {
        let my_hostname = get_my_hostname();
        let nodes = get_nodes_slurm();
        let coordinator = nodes[0].clone();
        Self {
            my_hostname,
            nodes,
            coordinator,
        }
    }
    pub fn am_i_coordinator(&self) -> bool {
        self.my_hostname == self.coordinator
    }
}

pub fn get_my_hostname() -> String {
    gethostname().into_string().unwrap()
}

pub fn get_nodes_slurm() -> Vec<String> {
    let slurm_nodelist = get_slurm_nodelist();
    internal_get_hostnames(&slurm_nodelist)
}

fn get_slurm_nodelist() -> String {
    match env::var("SLURM_NODELIST").map_err(|e| {
        eprintln!("Error when reading SLURM_NODELIST variable: {e}. Working locally instead")
    }) {
        Ok(slurm_nodelist) => slurm_nodelist,
        Err(_) => "".to_string(),
    }
}

fn internal_get_hostnames(slurm_nodelist: &str) -> Vec<String> {
    // eprintln!("SLURM_NODELIST: {slurm_nodelist}");
    if slurm_nodelist.len() == 0 {
        return vec![get_my_hostname()];
    }
    let mut split = slurm_nodelist.split("[");
    let mut results = vec![];
    if let Some(prefix) = split.next() {
        let mut node_expression = split
            .next()
            .expect("Error getting nodes after prefix")
            .to_string();
        let _last_char = node_expression.pop();
        let elements = node_expression.split(",");
        for e in elements {
            if e.contains("-") {
                let mut split = e.split("-");
                let start = split
                    .next()
                    .expect(&format!("Error when reading start in {e}"));
                let end = split
                    .next()
                    .expect(&format!("Error when reading end in {e}"));
                let s = start.parse::<u32>().unwrap();
                let e = end.parse::<u32>().unwrap();
                let width = start.chars().count();
                for i in s..=e {
                    results.push(format!("{prefix}{i:0>width$}", width = width));
                }
            } else {
                // single node
                results.push(format!("{prefix}{e}"));
            }
        }
    };
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty() {
        let hostname = get_my_hostname();
        let res = internal_get_hostnames("");
        assert_eq!(vec![hostname], res);
    }
    #[test]
    fn test_single() {
        let res = internal_get_hostnames("node[1]");
        assert_eq!(res, ["node1"]);
        let res = internal_get_hostnames("node[01]");
        assert_eq!(res, ["node01"]);
        let res = internal_get_hostnames("node[001]");
        assert_eq!(res, ["node001"]);
    }
    #[test]
    fn test_range() {
        let res = internal_get_hostnames("node[09-12]");
        assert_eq!(res, ["node09", "node10", "node11", "node12"]);
        let res = internal_get_hostnames("node[009-012]");
        assert_eq!(res, ["node009", "node010", "node011", "node012"]);
    }
    #[test]
    fn test_comma() {
        let res = internal_get_hostnames("node[01,05,08]");
        assert_eq!(res, ["node01", "node05", "node08"])
    }
}
