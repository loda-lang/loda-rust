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
        let clusterids: HashSet<usize> = Self::clusterids_containing_programids(&self.programid_to_clusterid, &program_ids);
        if clusterids.is_empty() {
            self.upsert_with_clusterid(&program_ids, self.current_cluster_id);
            self.current_cluster_id += 1;
            return;
        }
        for clusterid in clusterids {
            self.upsert_with_clusterid(&program_ids, clusterid);
            break;
        }
    }

    fn upsert_with_clusterid(&mut self, program_ids: &Vec<u32>, cluster_id: usize) {
        for program_id in program_ids {
            self.programid_to_clusterid.insert(*program_id, cluster_id);
        }
    }

    fn replace_clusterid(programid_to_clusterid: &mut ProgramIdToClusterId, old_clusterid: usize, new_clusterid: usize) {
        programid_to_clusterid.replace_value(old_clusterid, new_clusterid);
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

trait ConvertToString {
    fn convert_to_string(&self) -> String;
}

impl ConvertToString for ProgramIdToClusterId {
    fn convert_to_string(&self) -> String {
        let mut program_ids: Vec<u32> = self.iter().map(|entry| *entry.0).collect();
        program_ids.sort();
        let mut strings = Vec::<String>::new();
        for program_id in program_ids {
            match self.get(&program_id) {
                Some(clusterid) => {
                    strings.push(format!("{}:{}", program_id, clusterid));
                },
                None => {
                    strings.push("BOOM".to_string());
                }
            }
        }
        strings.join(",")
    }
}

trait ReplaceValue {
    fn replace_value(&mut self, old_value: usize, new_value: usize);
}

impl ReplaceValue for ProgramIdToClusterId {
    fn replace_value(&mut self, old_value: usize, new_value: usize) {
        if old_value == new_value {
            return;
        }
        for (_, value) in self.iter_mut() {
            if *value == old_value {
                *value = new_value;
            }
        }
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

    #[test]
    fn test_30001_replace_clusterid() {
        let mut programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        Clusters::replace_clusterid(&mut programid_to_clusterid, 1, 5);
        assert_eq!(programid_to_clusterid.convert_to_string(), "40:5,45:5,1113:2,10051:2,123456:3");
        Clusters::replace_clusterid(&mut programid_to_clusterid, 2, 5);
        assert_eq!(programid_to_clusterid.convert_to_string(), "40:5,45:5,1113:5,10051:5,123456:3");
        Clusters::replace_clusterid(&mut programid_to_clusterid, 3, 5);
        assert_eq!(programid_to_clusterid.convert_to_string(), "40:5,45:5,1113:5,10051:5,123456:5");
    }
}
