use crate::common::OeisIdStringMap;
use crate::oeis::OeisId;
use loda_rust_core::execute::ProgramSerializerContext;
use std::convert::TryFrom;

pub struct ProgramSerializerContextWithSequenceName {
    oeis_id_name_map: OeisIdStringMap
}

impl ProgramSerializerContextWithSequenceName {
    pub fn new(oeis_id_name_map: OeisIdStringMap) -> Self {
        ProgramSerializerContextWithSequenceName {
            oeis_id_name_map: oeis_id_name_map
        }
    }
}

impl ProgramSerializerContext for ProgramSerializerContextWithSequenceName {
    fn sequence_name_for_oeis_id(&self, oeis_id_u64: u64) -> Option<String> {
        let oeis_id: OeisId = match u32::try_from(oeis_id_u64) {
            Ok(oeis_id_raw) => {
                OeisId::from(oeis_id_raw)
            },
            Err(_error) => {
                return None;
            }
        };
        match self.oeis_id_name_map.get(&oeis_id) {
            Some(name_ref) => {
                let sequence_name: String = name_ref.clone();
                return Some(sequence_name);
            },
            None => {
                return None;
            }
        }
    }
}
