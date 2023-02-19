use clap::Parser;
use rand::SeedableRng;
use rand_distr::Distribution;

const SECONDS_PER_YEAR: f64 = 31556952.0;

#[derive(Parser)]
pub struct GenReturnsArgs {
    /// Simulation time in seconds (from first data point to last). Incomatiable with interval_seconds
    #[arg(short, long, conflicts_with("interval_seconds"), required_unless_present("interval_seconds"))]
    pub total_seconds: Option<usize>,

    /// Time between data points in seconds. Incomatiable with --total-seconds
    #[arg(short, long, conflicts_with("total_seconds"), required_unless_present("total_seconds"))]
    pub interval_seconds: Option<usize>,

    /// How many data points to generate (equally spaced in time)
    #[arg(short, long)]
    pub num_points: usize,

    /// The yearly (geometric) mean return
    #[arg(long, default_value_t = 1.0)]
    pub yearly_mean: f64,

    /// The yearly standard deviation (geometric)
    #[arg(long, default_value_t = 1.5)]
    pub yearly_stddev: f64,

    /// The seed to use for random number generation (for reproducible results)
    #[arg(long)]
    pub seed: Option<u64>,
}

pub fn gen_returns(args: &GenReturnsArgs) -> impl Iterator<Item = f64> {
    let mut interval_seconds: f64 = 0.0;
    let mut total_seconds: f64 = 0.0;
    let num_points_f = args.num_points as f64;
    if let Some(s) = args.total_seconds {
        total_seconds = s as f64;
        interval_seconds = total_seconds / num_points_f;
    } else if let Some(s) = args.interval_seconds {
        interval_seconds = s as f64;
        total_seconds = interval_seconds * num_points_f;
    }

    let yearly_mu = args.yearly_mean.ln();
    let yearly_sigma = args.yearly_stddev.ln();

    let ticks_per_year = SECONDS_PER_YEAR / interval_seconds;
    let tick_mu = yearly_mu / ticks_per_year;
    let tick_sigma = (yearly_sigma.powi(2) / ticks_per_year).sqrt();

    let tick_distr = rand_distr::LogNormal::new(tick_mu, tick_sigma).unwrap();

    let rng = if let Some(seed) = args.seed {
        rand::rngs::StdRng::seed_from_u64(seed)
    } else {
        rand::rngs::StdRng::from_entropy()
    };

    tick_distr.sample_iter(rng).take(args.num_points)
}

#[derive(Parser)]
pub struct AccumulateArgs {
    /// Whether to accumulate returns
    #[arg(short, long, default_value_t = false)]
    pub accumulate: bool,

    /// The value to begin accumulating from at t=0
    #[arg(long, default_value_t = 1.0)]
    pub start_value: f64,

    /// Leverage to be held constant over the entire series (releverages continuously between points)
    #[arg(long, conflicts_with_all(["pointwise_leverage", "initial_leverage"]), allow_hyphen_values(true))]
    pub continuous_leverage: Option<f64>,

    /// Leverage to be held constant over the entire series (releverages discretely at every point)
    #[arg(long, conflicts_with_all(["continuous_leverage", "initial_leverage"]), allow_hyphen_values(true))]
    pub pointwise_leverage: Option<f64>,

    /// Leverage at t=0, never releveraged
    #[arg(long, conflicts_with_all(["continuous_leverage", "pointwise_leverage"]), allow_hyphen_values(true))]
    pub initial_leverage: Option<f64>,
}

pub fn accumulate(returns: impl Iterator<Item = f64>, args: &AccumulateArgs) -> Vec<f64> {
    if !args.accumulate {
        return returns.collect();
    }
    let mut acc = args.start_value;
    if let Some(continuous_leverage) = args.continuous_leverage {
        returns
            .map(|r| r.powf(continuous_leverage))
            .map(|r| {let v = acc * r; acc = v; v})
            .collect()
    } else if let Some(pointwise_leverage) = args.pointwise_leverage {
        returns
            .map(|r| (1.0 + ((r - 1.0) * pointwise_leverage)).max(0.0))
            .map(|r| {let v = acc * r; acc = v; v})
            .collect()
    } else if let Some(initial_leverage) = args.initial_leverage {
        acc = args.start_value * initial_leverage;
        returns
            .map(|r| {let v = acc * r; acc = v; v})
            .map(|a| a - args.start_value * (initial_leverage - 1.0))
            .collect()
    } else {
        returns
            .map(|r| {let v = acc * r; acc = v; v})
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::gen_returns;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn gen_returns_with_fixed_seed() {
        let args = super::GenReturnsArgs {
            total_seconds: None,
            interval_seconds: Some(1),
            num_points: 10,
            yearly_mean: 1.1,
            yearly_stddev: 1.5,
            seed: Some(123456789),
        };

        let res = gen_returns(&args);
        assert_eq!(vec![
            1.0000429075842392,
            0.999960403828504,
            0.9999473836672608,
            0.9999852885724231,
            0.9999308265121937,
            0.9999956874033457,
            1.0000545633156286,
            1.0000529797693074,
            0.9999630744056991,
            0.9999348459587809,
        ], res.collect::<Vec<f64>>());
    }

    #[test]
    fn accumulate_test() {
        let args = super::AccumulateArgs {
            accumulate: true,
            start_value: 100.0,
            continuous_leverage: None,
            pointwise_leverage: None,
            initial_leverage: None,
        };
        let returns: Vec<f64> = vec![1.04, 1.01, 0.99, 0.98, 1.05, 1.1, 0.4];
        let res = super::accumulate(returns.into_iter(), &args);
        assert_eq!(vec![
            100.0 * 1.04,
            100.0 * 1.04 * 1.01,
            100.0 * 1.04 * 1.01 * 0.99,
            100.0 * 1.04 * 1.01 * 0.99 * 0.98,
            100.0 * 1.04 * 1.01 * 0.99 * 0.98 * 1.05,
            100.0 * 1.04 * 1.01 * 0.99 * 0.98 * 1.05 * 1.1,
            100.0 * 1.04 * 1.01 * 0.99 * 0.98 * 1.05 * 1.1 * 0.4,
        ], res);
    }

    #[test]
    fn accumulate_with_continuous_leverage_test() {
        let leverage = 5.0;
        let args = super::AccumulateArgs {
            accumulate: true,
            start_value: 1.0,
            continuous_leverage: Some(leverage),
            pointwise_leverage: None,
            initial_leverage: None,
        };
        let returns: Vec<f64> = vec![1.04, 1.01, 0.99, 0.98, 1.05, 1.1, 0.4];
        let leveraged_returns: Vec<f64> = returns.clone().iter().map(|r| r.powf(leverage)).collect();
        let res = super::accumulate(returns.into_iter(), &args);
        assert_eq!(vec![
            1.0 * leveraged_returns.iter().take(1).product::<f64>(),
            1.0 * leveraged_returns.iter().take(2).product::<f64>(),
            1.0 * leveraged_returns.iter().take(3).product::<f64>(),
            1.0 * leveraged_returns.iter().take(4).product::<f64>(),
            1.0 * leveraged_returns.iter().take(5).product::<f64>(),
            1.0 * leveraged_returns.iter().take(6).product::<f64>(),
            1.0 * leveraged_returns.iter().take(7).product::<f64>(),
        ], res);
    }

    #[test]
    fn accumulate_with_initial_leverage_test() {
        let leverage = 5.0;
        let args = super::AccumulateArgs {
            accumulate: true,
            start_value: 10.0,
            continuous_leverage: None,
            pointwise_leverage: None,
            initial_leverage: Some(leverage),
        };
        let returns: Vec<f64> = vec![1.04, 1.01, 0.99, 0.98, 1.05, 1.1, 0.4];
        let res = super::accumulate(returns.clone().into_iter(), &args);
        let mut ret_product = 1.0;
        for (ret, acc) in std::iter::zip(returns, res) {
            ret_product *= ret;
            assert_approx_eq!(50.0 * ret_product - 40.0, acc);
        }
    }
}
