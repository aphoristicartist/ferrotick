use ferrotick_core::Bar;
use fin_primitives::signals::batch_f64::{self, AtrSeed};
use ta::indicators::{BollingerBands, MovingAverageConvergenceDivergence};
use ta::Next;

use crate::{MlError, MlResult};

pub struct MacdSeries {
    pub macd: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
}

pub struct BollingerSeries {
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn compute_rsi(closes: &[f64], period: usize) -> MlResult<Vec<Option<f64>>> {
    if period == 0 {
        return Err(MlError::InvalidConfig(String::from(
            "rsi_period must be greater than zero",
        )));
    }

    Ok(batch_f64::rsi(closes, period)
        .into_iter()
        .map(|value| (!value.is_nan()).then_some(value))
        .collect())
}

pub fn compute_macd(
    closes: &[f64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> MlResult<MacdSeries> {
    if fast == 0 || slow == 0 || signal == 0 || fast >= slow {
        return Err(MlError::InvalidConfig(String::from(
            "MACD config must satisfy fast > 0, slow > 0, signal > 0, fast < slow",
        )));
    }

    let mut indicator = MovingAverageConvergenceDivergence::new(fast, slow, signal)
        .map_err(|err| MlError::InvalidConfig(err.to_string()))?;

    let warmup = slow + signal - 1;
    let mut macd = Vec::with_capacity(closes.len());
    let mut signal_line = Vec::with_capacity(closes.len());

    for (index, close) in closes.iter().enumerate() {
        let value = indicator.next(*close);
        let valid = index + 1 >= warmup;
        macd.push(valid.then_some(value.macd));
        signal_line.push(valid.then_some(value.signal));
    }

    Ok(MacdSeries {
        macd,
        signal: signal_line,
    })
}

pub fn compute_bollinger(closes: &[f64], period: usize, stdev: f64) -> MlResult<BollingerSeries> {
    if period == 0 || !stdev.is_finite() || stdev <= 0.0 {
        return Err(MlError::InvalidConfig(String::from(
            "Bollinger config must satisfy period > 0 and stdev > 0",
        )));
    }

    let mut indicator = BollingerBands::new(period, stdev)
        .map_err(|err| MlError::InvalidConfig(err.to_string()))?;

    let mut upper = Vec::with_capacity(closes.len());
    let mut lower = Vec::with_capacity(closes.len());

    for (index, close) in closes.iter().enumerate() {
        let value = indicator.next(*close);
        let valid = index + 1 >= period;
        upper.push(valid.then_some(value.upper));
        lower.push(valid.then_some(value.lower));
    }

    Ok(BollingerSeries { upper, lower })
}

pub fn compute_atr(bars: &[Bar], period: usize) -> MlResult<Vec<Option<f64>>> {
    if period == 0 {
        return Err(MlError::InvalidConfig(String::from(
            "atr_period must be greater than zero",
        )));
    }

    let high: Vec<f64> = bars.iter().map(|bar| bar.high).collect();
    let low: Vec<f64> = bars.iter().map(|bar| bar.low).collect();
    let close: Vec<f64> = bars.iter().map(|bar| bar.close).collect();

    Ok(batch_f64::atr(&high, &low, &close, period, AtrSeed::TaLib)
        .into_iter()
        .map(|value| (!value.is_nan()).then_some(value))
        .collect())
}
