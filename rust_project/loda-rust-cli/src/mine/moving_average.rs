pub struct MovingAverage {
    len: usize,
    values: Vec::<u64>,
    current_index: usize,
    weights: Vec::<u64>,
    weights_total: u64,
}

impl MovingAverage {
    pub fn new() -> Self {
        let len = 19;    
        let weights = vec![
            8, 16, 32, 64, 80, 85, 90, 95, 
            100,
            100,
            100,
            95, 90, 85, 80, 64, 32, 16, 8
        ];
        let mut weight_total = 0;
        for v in &weights {
            weight_total += v;
        }
        Self {
            len: len,
            values: vec![0; len],
            current_index: 0,
            weights: weights,
            weights_total: weight_total
        }
    }

    pub fn insert(&mut self, value: u64) {
        self.values[self.current_index] = value;
    }

    pub fn average(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.len {
            let offset = (self.current_index + i) % self.len;
            let weight = self.weights[offset];
            let value = self.values[offset];
            sum += value * weight;
        }
        sum / self.weights_total
    }

    pub fn rotate(&mut self) {
        self.current_index = (self.current_index + 1) % self.len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_all_zeroes() {
        let instance = MovingAverage::new();
        assert_eq!(instance.average(), 0);
    }

    #[test]
    fn test_10001_all_some_value() {
        let mut instance = MovingAverage::new();
        for _ in 0..20 {
            instance.insert(1000);
            instance.rotate();
        }
        assert_eq!(instance.average(), 1000);
    }

    #[test]
    fn test_10002_half_some_value_half_zeroes() {
        let mut instance = MovingAverage::new();
        for _ in 0..10 {
            instance.insert(1000);
            instance.rotate();
        }
        assert_eq!(instance.average(), 540);
    }
}
