//! Measuring the cache levels for a desktop
//! getconf -a | rg -i cache_size
//! LEVEL1_ICACHE_SIZE                 32768
//! LEVEL1_DCACHE_SIZE                 32768
//! LEVEL2_CACHE_SIZE                  1048576
//! LEVEL3_CACHE_SIZE                  17301504

// Wrapper for rdtsc
fn rdtsc() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

// Xorshift64 for prng of creating "random" data to sum
struct Rng {
    x: u64,
}

impl Rng {
    pub fn new() -> Rng {
        Rng { x: rdtsc() }
    }

    pub fn next(&mut self) -> u64 {
        let res = self.x;
        self.x ^= self.x << 13;
        self.x ^= self.x >> 7;
        self.x ^= self.x << 17;
        res
    }
}

const LEVEL1_CACHE_SIZE: usize = 32768;
const LEVEL2_CACHE_SIZE: usize = 1048576;
const LEVEL3_CACHE_SIZE: usize = 17301504;

const LEVEL1_DATA_LEN: usize = (LEVEL1_CACHE_SIZE - 1024) / std::mem::size_of::<u64>();
const LEVEL2_DATA_LEN: usize = (LEVEL2_CACHE_SIZE - 1024) / std::mem::size_of::<u64>();
const LEVEL3_DATA_LEN: usize = (LEVEL3_CACHE_SIZE - 1024) / std::mem::size_of::<u64>();

fn main() {
    // Get a prng for this harness
    let mut rng = Rng::new();

    for (level, (data_len, iters)) in [
        (LEVEL1_DATA_LEN, 0xfffff),
        (LEVEL2_DATA_LEN, 0xffff),
        (LEVEL3_DATA_LEN, 0xfff),
        // Force out to main memory
        (LEVEL3_DATA_LEN * 8, 0xff),
    ]
    .iter()
    .enumerate()
    {
        // Generate the data for this test
        let data: Vec<_> = (0..*data_len).map(|_| rng.next()).collect();

        // Calculate the
        let value = data.iter().sum();

        // Keep a running best cycle count for this test case
        let mut best = u64::MAX;

        // Perform the work
        for _ in 0..*iters {
            let mut sum = 0;

            // Start the timer for this iteration
            let start = rdtsc();

            for i in 0..*data_len {
                // Actual work being measured here
                sum += data[i];
            }

            // Stop the timer for this test case and get the number of clock cycles
            // for this iteration
            let elapsed = rdtsc() - start;

            // Use the `sum` value to not get optimized out
            assert!(sum == value);

            // If this was the fastest cycle count we've seen so far, keep it
            if elapsed < best {
                best = elapsed;
            }
        }

        // Print the stats for this cache level test
        println!(
            "Level {}: Elements/cycle: {:4.2}",
            level + 1,
            *data_len as f64 / best as f64
        );
    }
}
