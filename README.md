# Finsim

## Generate returns series

Possible to generate either point-wise independent returns or accumulated
returns (`-a`).

Possible to specify either `--total-seconds` or `--interval-seconds` but not both,
the other will be calculated based on `--num-points`.

### Examples

Generate hourly returns over 180 days
`cargo run --release -- --total-seconds 15552000 --num-points 4320`

Change to daily returns over 180 days
`cargo run --release -- --total-seconds 15552000 --num-points 180`

Generate accumulated (compounded) daily returns for 1000 days, with a yearly
expected geometric mean return of 10% and stddev of 2.0
`cargo run --release -- -a --interval-seconds 86400 --num-points 1000 --yearly-mean 1.10 --yearly-stddev 2.0`

Use a seed to get deterministic results
`cargo run --release -- -a --interval-seconds 60 --num-points 1000 --seed 123456789`
