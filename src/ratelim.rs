use ratelimit_meter::algorithms::NonConformance;
use ratelimit_meter::{DirectRateLimiter, LeakyBucket, NegativeMultiDecision};

pub struct RateLimiter {
    lim: DirectRateLimiter<LeakyBucket>,
}

impl RateLimiter {
    pub fn new(capacity_per_second: u32) -> Self {
        Self {
            lim: DirectRateLimiter::<LeakyBucket>::per_second(
                std::num::NonZeroU32::new(capacity_per_second)
                    .expect("RateLimiter capacity to be non-zero"),
            ),
        }
    }

    pub fn blocking_admittance_check(&mut self, amount: u32) {
        loop {
            match self.lim.check_n(amount) {
                Ok(_) => return,
                Err(NegativeMultiDecision::BatchNonConforming(_, over)) => {
                    let duration = over.wait_time_from(std::time::Instant::now());
                    log::trace!("RateLimiter: sleep for {:?}", duration);
                    std::thread::sleep(duration);
                }
                Err(err) => panic!("{}", err),
            }
        }
    }
}
