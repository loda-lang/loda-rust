use super::CheckFixedLengthSequence;
use super::FunnelConfig;
use super::WildcardChecker;
use loda_rust_core::util::Analyze;
use loda_rust_core::util::BigIntVec;

// Funnel of bloomfilters for use during mining.
//
// Originally LODA-RUST could only mine for sequences that had 40 or more terms.
// The OEIS 'stripped' file contains around 360k sequences of average length 38 terms.
// So only half of the OEIS sequences got added to the bloomfilter.
// Later, I added support for shorter than 40 terms sequences, 
// by introducing a "Wildcard" magic value in the bloomfilter.
// 
// A potential candiate program must pass these 5 stages:
// - basic
// - 10 terms
// - 20 terms
// - 30 terms
// - 40 terms
//
// The sooner a potential candidate program can be rejected,
// the sooner it's possible to mutate the genome and try again.
//
// The "Wildcard" magic value is the zero value or `BigInt::zero()`.
//
// If there is a sequence that has only 33 terms in the OEIS 'stripped' file, 
// then the first 33 terms are followed by 7 wildcard terms (zeroes).
// This sequence can now be inserted into the bloomfilter.
//
// When wildcard-checking if a sequence is contained in the bloomfilter, it works like this:
// When a candiate program passes 10terms, 20terms, 30terms bloomfilters, and gets rejected by the 40terms bloomfilter.
// Then replace the last term with a wildcard symbol, and try again.
// Then replace the second last term with a wildcard symbol, and try again.
// Then replace the third last term with a wildcard symbol, and try again.
// After around 10 attempts it can be determined if it's in the bloomfilter or not.
pub struct Funnel {
    checker10: CheckFixedLengthSequence,
    checker20: CheckFixedLengthSequence,
    checker30: CheckFixedLengthSequence,
    checker40: CheckFixedLengthSequence,

    metric_number_of_candidates_with_basiccheck: u64,
    metric_number_of_candidates_with_10terms: u64,
    metric_number_of_candidates_with_20terms: u64,
    metric_number_of_candidates_with_30terms: u64,
    metric_number_of_candidates_with_40terms: u64,
}

impl Funnel {
    pub fn new(
        checker10: CheckFixedLengthSequence, 
        checker20: CheckFixedLengthSequence,
        checker30: CheckFixedLengthSequence,
        checker40: CheckFixedLengthSequence,
    ) -> Self {
        Self {
            checker10: checker10, 
            checker20: checker20,
            checker30: checker30,
            checker40: checker40,
            metric_number_of_candidates_with_basiccheck: 0,
            metric_number_of_candidates_with_10terms: 0,
            metric_number_of_candidates_with_20terms: 0,
            metric_number_of_candidates_with_30terms: 0,
            metric_number_of_candidates_with_40terms: 0,
        }
    }

    pub fn check_basic(&mut self, terms: &BigIntVec) -> bool {
        if !is_possible_candidate_basic_checks(terms) {
            return false;
        }
        self.metric_number_of_candidates_with_basiccheck += 1;
        true
    }

    pub fn check10(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker10.check(terms) {
            return false;
        }
        self.metric_number_of_candidates_with_10terms += 1;
        true
    }

    #[allow(dead_code)]
    pub fn check20(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker20.check(terms) {
            return false;
        }
        self.metric_number_of_candidates_with_20terms += 1;
        true
    }

    #[allow(dead_code)]
    pub fn check30(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker30.check(terms) {
            return false;
        }
        self.metric_number_of_candidates_with_30terms += 1;
        true
    }

    #[allow(dead_code)]
    pub fn check40(&mut self, terms: &BigIntVec) -> bool {
        if !self.checker40.check(terms) {
            return false;
        }
        self.metric_number_of_candidates_with_40terms += 1;
        true
    }

    pub fn check20_with_wildcards(&mut self, terms: &BigIntVec) -> Option<usize> {
        let result: Option<usize> = self.checker20.check_with_wildcards(terms, FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS);
        if result.is_some() {
            self.metric_number_of_candidates_with_20terms += 1;
        }
        result
    }

    pub fn check30_with_wildcards(&mut self, terms: &BigIntVec) -> Option<usize> {
        let result: Option<usize> = self.checker30.check_with_wildcards(terms, FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS);
        if result.is_some() {
            self.metric_number_of_candidates_with_30terms += 1;
        }
        result
    }

    pub fn mut_check40_with_wildcards(&mut self, terms: &mut BigIntVec) -> Option<usize> {
        let result: Option<usize> = self.checker40.mut_check_with_wildcards(terms, FunnelConfig::MINIMUM_NUMBER_OF_REQUIRED_TERMS);
        if result.is_some() {
            self.metric_number_of_candidates_with_40terms += 1;
        }
        result
    }

    pub fn metric_number_of_candidates_with_basiccheck(&self) -> u64 {
        self.metric_number_of_candidates_with_basiccheck
    }

    pub fn metric_number_of_candidates_with_10terms(&self) -> u64 {
        self.metric_number_of_candidates_with_10terms
    }

    pub fn metric_number_of_candidates_with_20terms(&self) -> u64 {
        self.metric_number_of_candidates_with_20terms
    }

    pub fn metric_number_of_candidates_with_30terms(&self) -> u64 {
        self.metric_number_of_candidates_with_30terms
    }

    pub fn metric_number_of_candidates_with_40terms(&self) -> u64 {
        self.metric_number_of_candidates_with_40terms
    }

    pub fn reset_metrics(&mut self) {
        self.metric_number_of_candidates_with_basiccheck = 0;
        self.metric_number_of_candidates_with_10terms = 0;
        self.metric_number_of_candidates_with_20terms = 0;
        self.metric_number_of_candidates_with_30terms = 0;
        self.metric_number_of_candidates_with_40terms = 0;
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
