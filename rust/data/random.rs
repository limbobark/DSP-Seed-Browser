pub struct DspRandom {
    inext: usize,
    inextp: usize,
    pub seed: i32,
    seed_array: [i32; 56],
}

impl DspRandom {
    #[inline]
    // Knuth's subtractive random number generator algorithm
    /*
     * 1.store a circular list of 56 random numbers
     * 2.initialization is process of filling the list, then randomize those values with a specific deterministic algorithm
     * 3.two indices are kept which are 31 apart
     * 4.new random number is the difference of the two values at the two indices
     * 5.store new random number in the list
     */
    pub fn new(seed: i32) -> Self {
        let mut seed_array = [0; 56];
        let mut num1 = 161803398 - seed.abs();
        // 161803398 golden ratio
        /*
         * The golden ratio is a mathematical constant, approximately 1.618033988749895,
         */
        // Nothing-up-my-sleeve number
        /*
         * In cryptography, nothing-up-my-sleeve numbers are any numbers which, by their construction, are above suspicion of hidden properties. They are used in creating cryptographic functions such as hashes and ciphers. These algorithms often need randomized constants for mixing or initialization purposes. The cryptographer may wish to pick these values in a way that demonstrates the constants were not selected for a nefarious purpose, for example, to create a backdoor to the algorithm.[1] These fears can be allayed by using numbers created in a way that leaves little room for adjustment. An example would be the use of initial digits from the number π as the constants.[2] Using digits of π millions of places after the decimal point would not be considered trustworthy because the algorithm designer might have selected that starting point because it created a secret weakness the designer could later exploit—though even with natural-seeming selections, enough entropy exists in the possible choices that the utility of these numbers has been questioned.
         */
        seed_array[55] = num1;
        let mut num2 = 1;
        for index1 in 1..55 {
            let index2 = (21 * index1) % 55;
            seed_array[index2] = num2;
            num2 = num1 - num2;
            if num2 < 0 {
                num2 += i32::MAX;
            }
            num1 = seed_array[index2]
        }
        for _index3 in 1..5 {
            for index4 in 1..56 {
                let mut val = seed_array[index4].wrapping_sub(seed_array[1 + (index4 + 30) % 55]);
                if val < 0 {
                    val += i32::MAX;
                }
                seed_array[index4] = val;
            }
        }

        Self {
            inext: 0,
            inextp: 31,
            seed,
            seed_array,
        }
    }

    #[inline(always)]
    fn sample(&mut self) -> f64 {
        self.inext = (self.inext + 1) % 56;
        self.inextp = (self.inextp + 1) % 56;
        let mut num = self.seed_array[self.inext].wrapping_sub(self.seed_array[self.inextp]);
        if num < 0 {
            num = num.wrapping_add(i32::MAX);
        }
        self.seed_array[self.inext] = num;
        (num as f64) * (1.0 / (i32::MAX as f64))
    }

    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        self.sample()
    }

    #[inline(always)]
    pub fn next_f32(&mut self) -> f32 {
        self.sample() as f32
    }

    #[inline(always)]
    pub fn next_i32(&mut self, max_value: i32) -> i32 {
        (self.sample() * (max_value as f64)) as i32
    }

    #[inline(always)]
    pub fn next_usize(&mut self) -> usize {
        (self.sample() * (i32::MAX as f64)) as usize
    }

    #[inline(always)]
    pub fn next_seed(&mut self) -> i32 {
        (self.sample() * (i32::MAX as f64)) as i32
    }
}
