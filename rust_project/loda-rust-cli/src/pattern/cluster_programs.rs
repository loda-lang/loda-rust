use std::collections::HashMap;
use std::collections::HashSet;

type ProgramIdToClusterId = HashMap<u32, usize>;

pub struct Clusters {
    programid_to_clusterid: ProgramIdToClusterId,
    current_cluster_id: usize,
}

impl Clusters {
    pub fn new() -> Self {
        Self {
            programid_to_clusterid: HashMap::new(),
            current_cluster_id: 0,
        }
    }

    pub fn insert(&mut self, program_ids: &Vec<u32>) {
        if program_ids.is_empty() {
            return;
        }
        let clusterids: HashSet<usize> = Self::clusterids_containing_programids(&self.programid_to_clusterid, &program_ids);
        if clusterids.is_empty() {
            self.upsert_with_clusterid(program_ids, self.current_cluster_id);
            self.current_cluster_id += 1;
            return;
        }
        let mut first_clusterid: usize = 0;
        for clusterid in &clusterids {
            first_clusterid = *clusterid;
            self.upsert_with_clusterid(program_ids, *clusterid);
            break;
        }
        if clusterids.len() < 2 {
            return;
        }
        let mut lowest_clusterid: usize = first_clusterid;
        for clusterid in &clusterids {
            if *clusterid < lowest_clusterid {
                lowest_clusterid = *clusterid;
            }
        }
        for clusterid in &clusterids {
            // assign all the programs the same clusterid
            Self::replace_clusterid(&mut self.programid_to_clusterid, *clusterid, lowest_clusterid);
        }
    }

    pub fn clusters_of_programids(&self) -> Vec<HashSet<u32>> {
        let clusterid_to_programid = &self.programid_to_clusterid.transpose_key_value();
        let mut result = Vec::<HashSet<u32>>::new();
        for (_, value) in clusterid_to_programid {
            result.push(value.clone());
        }
        result
    }

    pub fn lowest_program_id_in_set(program_id_set: &HashSet<u32>) -> Option<u32> {
        program_id_set.lowest_value()
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

trait TransposeKeyValue {
    fn transpose_key_value(&self) -> HashMap<usize, HashSet<u32>>;
}

impl TransposeKeyValue for ProgramIdToClusterId {
    fn transpose_key_value(&self) -> HashMap<usize, HashSet<u32>> {
        let mut result = HashMap::<usize, HashSet<u32>>::new();
        for (key, value) in self {
            let entry = result.entry(*value).or_insert_with(|| HashSet::new());
            entry.insert(*key);
        }
        result
    }
}

trait LowestValue {
    fn lowest_value(&self) -> Option<u32>;
}

impl LowestValue for HashSet<u32> {
    fn lowest_value(&self) -> Option<u32> {
        if self.is_empty() {
            return None;
        }
        let mut lowest_value: u32 = 0;
        let mut index: usize = 0;
        for value in self {
            if index == 0 || *value < lowest_value {
                lowest_value = *value;
            }
            index += 1;
        }
        Some(lowest_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    fn mock_clusters() -> Clusters {
        let mut clusters = Clusters::new();
        clusters.insert(&vec![101,102]);
        clusters.insert(&vec![201,202]);
        clusters.insert(&vec![301,302]);
        clusters.insert(&vec![401,402]);
        clusters.insert(&vec![501,502]);
        clusters
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

    #[test]
    fn test_10000_insert_separate_clusters() {
        let mut clusters = Clusters::new();
        assert_eq!(clusters.programid_to_clusterid.convert_to_string(), "");
        clusters.insert(&vec![101,102,103]);
        assert_eq!(clusters.programid_to_clusterid.convert_to_string(), "101:0,102:0,103:0");
        clusters.insert(&vec![201,202,203,204]);
        assert_eq!(clusters.programid_to_clusterid.convert_to_string(), "101:0,102:0,103:0,201:1,202:1,203:1,204:1");
    }

    #[test]
    fn test_10001_insert_and_merge_clusters_simple() {
        let mut clusters = mock_clusters();
        let before_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(before_merge, "101:0,102:0,201:1,202:1,301:2,302:2,401:3,402:3,501:4,502:4");
        clusters.insert(&vec![102,302]); // merge 2 clusters into one cluster
        let after_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(after_merge, "101:0,102:0,201:1,202:1,301:0,302:0,401:3,402:3,501:4,502:4");
    }

    #[test]
    fn test_10002_insert_and_merge_clusters_order_doesnt_matter1() {
        let mut clusters = mock_clusters();
        let before_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(before_merge, "101:0,102:0,201:1,202:1,301:2,302:2,401:3,402:3,501:4,502:4");
        clusters.insert(&vec![101,302,502]); // merge 3 clusters into one cluster
        let after_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(after_merge, "101:0,102:0,201:1,202:1,301:0,302:0,401:3,402:3,501:0,502:0");
    }

    #[test]
    fn test_10003_insert_and_merge_clusters_order_doesnt_matter2() {
        let mut clusters = mock_clusters();
        let before_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(before_merge, "101:0,102:0,201:1,202:1,301:2,302:2,401:3,402:3,501:4,502:4");
        clusters.insert(&vec![502,302,101]); // merge 3 clusters into one cluster
        let after_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(after_merge, "101:0,102:0,201:1,202:1,301:0,302:0,401:3,402:3,501:0,502:0");
    }

    #[test]
    fn test_10004_insert_extend_existing_cluster() {
        let mut clusters = mock_clusters();
        let before_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(before_merge, "101:0,102:0,201:1,202:1,301:2,302:2,401:3,402:3,501:4,502:4");
        clusters.insert(&vec![202,203,204]); // extend the existing cluster with new programids
        let after_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(after_merge, "101:0,102:0,201:1,202:1,203:1,204:1,301:2,302:2,401:3,402:3,501:4,502:4");
    }

    #[test]
    fn test_10005_insert_extend_existing_cluster_and_merge_with_other_cluster() {
        let mut clusters = mock_clusters();
        let before_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(before_merge, "101:0,102:0,201:1,202:1,301:2,302:2,401:3,402:3,501:4,502:4");
        clusters.insert(&vec![202,203,204,402]); // programids from two different clusters and new programids
        let after_merge = clusters.programid_to_clusterid.convert_to_string();
        assert_eq!(after_merge, "101:0,102:0,201:1,202:1,203:1,204:1,301:2,302:2,401:1,402:1,501:4,502:4");
    }

    #[test]
    fn test_10006_insert_empty_array() {
        let mut clusters = mock_clusters();
        assert_eq!(clusters.current_cluster_id, 5);
        assert_eq!(clusters.programid_to_clusterid.len(), 10);
        clusters.insert(&vec!());
        assert_eq!(clusters.current_cluster_id, 5);
        assert_eq!(clusters.programid_to_clusterid.len(), 10);
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

    #[test]
    fn test_40001_lowest_program_id_some() {
        let program_ids: Vec<u32> = vec![99,999,7,1000,18,8];
        let hashset: HashSet<u32> = HashSet::from_iter(program_ids.iter().cloned());
        let result = Clusters::lowest_program_id_in_set(&hashset);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn test_40002_lowest_program_id_none() {
        let hashset: HashSet<u32> = HashSet::new();
        let result = Clusters::lowest_program_id_in_set(&hashset);
        assert_eq!(result, None);
    }

    #[test]
    fn test_50001_transpose_key_value() {
        let programid_to_clusterid: ProgramIdToClusterId = mock_programid_to_clusterid();
        let clusterid_to_programid = programid_to_clusterid.transpose_key_value();
        assert_eq!(clusterid_to_programid.len(), 3);
    }
}
