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

    fn clusterids_containing_programids(programid_to_clusterid: &ProgramIdToClusterId, program_ids: &Vec<u32>) -> HashSet<usize> {
        let mut cluster_ids = HashSet::<usize>::new();
        for program_id in program_ids {
            let cluster_id: usize = match programid_to_clusterid.get(&program_id) {
                Some(value) => *value,
                None => {
                    continue;
                }
            };
            cluster_ids.insert(cluster_id);
        }
        return cluster_ids;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_10000_insert_without_overlap() {
        let mut clusters = Clusters::new();
        assert_eq!(clusters.programid_to_clusterid.len(), 0);
        assert_eq!(clusters.current_cluster_id, 0);

        clusters.insert(vec![101,102,103]);
        assert_eq!(clusters.programid_to_clusterid.len(), 3);
        assert_eq!(clusters.current_cluster_id, 1);

        clusters.insert(vec![201,202,203,204]);
        assert_eq!(clusters.programid_to_clusterid.len(), 7);
        assert_eq!(clusters.current_cluster_id, 2);
    }

    fn mock_programid_to_clusterid() -> ProgramIdToClusterId {
        let mut programid_to_clusterid = ProgramIdToClusterId::new();
        programid_to_clusterid.insert(40, 1);
        programid_to_clusterid.insert(45, 1);
        programid_to_clusterid.insert(1113, 2);
        programid_to_clusterid.insert(10051, 2);
        programid_to_clusterid.insert(123456, 3);
        programid_to_clusterid
    }

    fn clusterids_containing_programids_as_string(programid_to_clusterid: &ProgramIdToClusterId, program_ids: &Vec<u32>) -> String {
        let clusterid_set: HashSet<usize> = Clusters::clusterids_containing_programids(programid_to_clusterid, program_ids);
        let mut clusterid_vec = Vec::<usize>::from_iter(clusterid_set);
        clusterid_vec.sort();
        let clusterid_strings: Vec<String> = clusterid_vec.iter().map( |&id| id.to_string()).collect(); 
        clusterid_strings.join(",")
    }

    #[test]
    fn test_20000_clusterids_containing_programids_no_clusterids_found_for_empty_input() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let programids: Vec::<u32> = vec!();
        let clusterids = clusterids_containing_programids_as_string(&programid_to_clusterid, &programids);
        assert_eq!(clusterids, "");
    }

    #[test]
    fn test_20001_clusterids_containing_programids_no_clusterids_found_for_unknown_programids() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let programids: Vec::<u32> = vec![666];
        let clusterids = clusterids_containing_programids_as_string(&programid_to_clusterid, &programids);
        assert_eq!(clusterids, "");
    }

    #[test]
    fn test_20002_clusterids_containing_programids_found_one_clusterid() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let programids: Vec::<u32> = vec![40];
        let clusterids = clusterids_containing_programids_as_string(&programid_to_clusterid, &programids);
        assert_eq!(clusterids, "1");
    }

    #[test]
    fn test_20003_clusterids_containing_programids_found_two_clusterids() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let programids: Vec::<u32> = vec![40, 1113, 10051];
        let clusterids = clusterids_containing_programids_as_string(&programid_to_clusterid, &programids);
        assert_eq!(clusterids, "1,2");
    }

    #[test]
    fn test_20005_clusterids_containing_programids_both_known_and_unknown() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let programids: Vec::<u32> = vec![40, 666];
        let clusterids = clusterids_containing_programids_as_string(&programid_to_clusterid, &programids);
        assert_eq!(clusterids, "1");
    }
}
