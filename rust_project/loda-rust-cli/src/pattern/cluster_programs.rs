use std::collections::HashMap;
use std::collections::HashSet;

type ProgramIdToClusterId = HashMap<u32, usize>;

struct Clusters {
    programid_to_clusterid: ProgramIdToClusterId,
    current_cluster_id: usize,
}

impl Clusters {
    fn new() -> Self {
        Self {
            programid_to_clusterid: HashMap::new(),
            current_cluster_id: 0,
        }
    }

    fn insert(&mut self, program_ids: Vec<u32>) {
        self.upsert_with_clusterid(&program_ids, self.current_cluster_id);
        self.current_cluster_id += 1;
    }

    fn upsert_with_clusterid(&mut self, program_ids: &Vec<u32>, cluster_id: usize) {
        for program_id in program_ids {
            self.programid_to_clusterid.insert(*program_id, cluster_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_insert() {
        let mut clusters = Clusters::new();
        clusters.insert(vec![101,102,103]);
        clusters.insert(vec![201,202,203,204]);
        assert_eq!(clusters.programid_to_clusterid.len(), 7);
    }
}
