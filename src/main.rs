use clap::Parser;
use rand::SeedableRng;
use rand_distr::Distribution;

const SECONDS_PER_YEAR: f64 = 31556952.0;

#[derive(Parser, Debug)]
pub struct Args {
    /// Simulation time in seconds (from first data point to last). Incomatiable with interval_seconds
    #[arg(short, long, conflicts_with("interval_seconds"), required_unless_present("interval_seconds"))]
    total_seconds: Option<usize>,

    /// Time between data points in seconds. Incomatiable with --total-seconds
    #[arg(short, long, conflicts_with("total_seconds"), required_unless_present("total_seconds"))]
    interval_seconds: Option<usize>,

    /// How many data points to generate (equally spaced in time)
    #[arg(short, long)]
    num_points: usize,

    /// The yearly (geometric) mean return
    #[arg(long, default_value_t = 1.0)]
    yearly_mean: f64,

    /// The yearly standard deviation (geometric)
    #[arg(long, default_value_t = 1.5)]
    yearly_stddev: f64,

    /// The seed to use for random number generation (for reproducible results)
    #[arg(long)]
    seed: Option<u64>,

    /// Whether to accumulate returns
    #[arg(short, long, default_value_t = false)]
    accumulate: bool,

    /// The value to begin accumulating from at t=0
    #[arg(long, default_value_t = 1.0)]
    start_value: f64,
}

fn main() {
    let args = Args::parse();
    let returns = gen_returns(&args);
    if !args.accumulate {
        returns.for_each(|r| println!("{}", r));
        return;
    }
    
    accumulate(returns, args.start_value).iter().for_each(|r| println!("{}", r));
}

fn gen_returns(args: &Args) -> impl Iterator<Item = f64> {
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

fn accumulate(returns: impl Iterator<Item = f64>, start_value: f64) -> Vec<f64> {
    let mut acc = start_value;
    returns.map(|r| {let v = acc * r; acc = v; v}).collect()
}

#[cfg(test)]
mod tests {
    use crate::gen_returns;

    #[test]
    fn gen_returns_with_fixed_seed() {
        let args = super::Args {
            total_seconds: None,
            interval_seconds: Some(1),
            num_points: 10,
            yearly_mean: 1.1,
            yearly_stddev: 1.5,
            seed: Some(123456789),
            accumulate: false,
            start_value: 1.0,
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
        let returns: Vec<f64> = vec![1.04, 1.01, 0.99, 0.98, 1.05, 1.1, 0.4];
        let res = super::accumulate(returns.into_iter(), 100.0);
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
}