use crate::mine::CheckFixedLengthSequence;
use crate::util::Analyze;
use crate::oeis::stripped_sequence::BigIntVec;

pub struct Funnel<'a> {
    checker10: &'a CheckFixedLengthSequence,
    checker20: &'a CheckFixedLengthSequence,
    checker30: &'a CheckFixedLengthSequence,
    checker40: &'a CheckFixedLengthSequence,

    number_of_candidates_with_basiccheck: u64,
    number_of_candidates_with_10terms: u64,
    number_of_candidates_with_20terms: u64,
    number_of_candidates_with_30terms: u64,
    number_of_candidates_with_40terms: u64,
}

impl<'a> Funnel<'a> {
    pub fn new(
        checker10: &'a CheckFixedLengthSequence, 
        checker20: &'a CheckFixedLengthSequence,
        checker30: &'a CheckFixedLengthSequence,
        checker40: &'a CheckFixedLengthSequence,
    ) -> Self {
        Self {
            checker10: checker10, 
            checker20: checker20,
            checker30: checker30,
            checker40: checker40,
            number_of_candidates_with_basiccheck: 0,
            number_of_candidates_with_10terms: 0,
            number_of_candidates_with_20terms: 0,
            number_of_candidates_with_30terms: 0,
            number_of_candidates_with_40terms: 0,
        }
    }

    pub fn funnel_info(&self) -> String {
        format!(
            "[{},{},{},{},{}]",
            self.number_of_candidates_with_basiccheck,
            self.number_of_candidates_with_10terms,
            self.number_of_candidates_with_20terms,
            self.number_of_candidates_with_30terms,
            self.number_of_candidates_with_40terms,
        )
    }

    pub fn check_basic(&mut self, terms: &BigIntVec) -> bool {
        if !is_possible_candidate_basic_checks(terms) {
            return false;
        }
        self.number_of_candidates_with_basiccheck += 1;
        true
    }

    pub fn check10(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker10.check(terms) {
            return false;
        }
        self.number_of_candidates_with_10terms += 1;
        true
    }

    pub fn check20(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker20.check(terms) {
            return false;
        }
        self.number_of_candidates_with_20terms += 1;
        true
    }

    pub fn check30(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker30.check(terms) {
            return false;
        }
        self.number_of_candidates_with_30terms += 1;
        true
    }

    pub fn check40(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker40.check(terms) {
            return false;
        }
        self.number_of_candidates_with_40terms += 1;
        true
    }
}

fn is_possible_candidate_basic_checks(terms: &BigIntVec) -> bool {
    if Analyze::count_unique(&terms) < 8 {
        // there are many results where all terms are just zeros.
        // there are many results where all terms are a constant value.
        // there are many results where most of the terms is a constant value.
        // there are many results where the terms alternates between 2 values.
        // debug!("too few unique terms");
        return false;
    }
    if Analyze::is_almost_natural_numbers(&terms) {
        // there are many result that are like these
        // [0, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        // [1, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        // it's the natural numbers with 1 term different
        // debug!("too close to being the natural numbers");
        return false;
    }
    if Analyze::count_zero(&terms) >= 7 {
        // debug!("there are too many zero terms");
        return false;
    }
    if Analyze::is_all_the_same_value(&terms) {
        // debug!("all terms are the same");
        return false;
    }
    if Analyze::is_constant_step(&terms) {
        // debug!("the terms use constant step");
        return false;
    }
    true
}
