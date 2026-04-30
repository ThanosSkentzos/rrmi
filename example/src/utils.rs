use std::env;

use gethostname::gethostname;
#[derive(Debug)]
pub struct Utils {
    pub my_hostname: String,
    pub slurm_nodes: Vec<String>,
    pub slurm_coordinator: String,
    pub liacs_nodes: Vec<String>,
    pub liacs_coordinator: String,
}
impl Utils {
    pub fn new() -> Self {
        let my_hostname = get_my_hostname();
        let slurm_nodes = get_nodes_slurm();

        let slurm_coordinator = if slurm_nodes.len() > 1 {
            slurm_nodes[0].clone()
        } else {
            "".into()
        };

        let liacs_nodes = get_nodes_liacs();
        let liacs_coordinator = liacs_nodes[0].clone();

        Self {
            my_hostname,
            slurm_nodes,
            slurm_coordinator,
            liacs_nodes,
            liacs_coordinator,
        }
    }
    #[allow(unused)]
    pub fn am_i_slurm_coordinator(&self) -> bool {
        eprintln!(
            "My hostname: {}\n Coordinator: {}",
            self.my_hostname, self.slurm_coordinator
        );
        self.my_hostname == self.slurm_coordinator
    }

    pub fn am_i_liacs_coordinator(&self) -> bool {
        eprintln!(
            "My hostname: {}\nCoordinator: {}",
            self.my_hostname, self.liacs_coordinator
        );
        let res = self.liacs_coordinator.contains(&self.my_hostname);
        if res {
            eprintln!("I am the coordinator")
        } else {
            eprintln! {"I am NOT the coordinator"}
        };
        res
    }
}

pub fn get_my_hostname() -> String {
    gethostname().into_string().unwrap()
}

pub fn get_nodes_liacs() -> Vec<String> {
    vec![
        "0065073.student.liacs.nl".into(),
        "0065074.student.liacs.nl".into(),
    ]
}

pub fn get_nodes_slurm() -> Vec<String> {
    let slurm_nodelist = get_slurm_nodelist();
    if slurm_nodelist.chars().count() == 0 {
        return vec![];
    }
    parse_hostnames(&slurm_nodelist)
}

fn get_slurm_nodelist() -> String {
    match env::var("SLURM_NODELIST").map_err(|e| {
        eprintln!("Error when reading SLURM_NODELIST variable: {e}. Working locally instead")
    }) {
        Ok(slurm_nodelist) => slurm_nodelist,
        Err(_) => "".to_string(),
    }
}

fn parse_hostnames(slurm_nodelist: &str) -> Vec<String> {
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
        let res = parse_hostnames("");
        assert_eq!(vec![hostname], res);
    }
    #[test]
    fn test_single() {
        let res = parse_hostnames("node[1]");
        assert_eq!(res, ["node1"]);
        let res = parse_hostnames("node[01]");
        assert_eq!(res, ["node01"]);
        let res = parse_hostnames("node[001]");
        assert_eq!(res, ["node001"]);
    }
    #[test]
    fn test_range() {
        let res = parse_hostnames("node[09-12]");
        assert_eq!(res, ["node09", "node10", "node11", "node12"]);
        let res = parse_hostnames("node[009-012]");
        assert_eq!(res, ["node009", "node010", "node011", "node012"]);
    }
    #[test]
    fn test_comma() {
        let res = parse_hostnames("node[01,05,08]");
        assert_eq!(res, ["node01", "node05", "node08"])
    }

    #[test]
    fn test_comma_range() {
        let res = parse_hostnames("node[001,005,008-12]");
        assert_eq!(
            res,
            ["node001", "node005", "node008", "node009", "node010", "node011", "node012"]
        )
    }
}
