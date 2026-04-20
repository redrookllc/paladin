/// Technical indicators computed natively on f64 slices.
/// All functions return a Vec<f64> aligned to the input length; leading
/// positions that cannot be computed contain f64::NAN.

pub fn ema(values: &[f64], span: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if values.is_empty() || span == 0 {
        return out;
    }
    let alpha = 2.0 / (span as f64 + 1.0);
    let mut prev = values[0];
    out[0] = prev;
    for i in 1..values.len() {
        prev = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = prev;
    }
    out
}

pub fn sma(values: &[f64], window: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if window == 0 || values.len() < window {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..values.len() {
        sum += values[i];
        if i >= window {
            sum -= values[i - window];
        }
        if i + 1 >= window {
            out[i] = sum / window as f64;
        }
    }
    out
}

pub fn rsi(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if n <= period || period == 0 {
        return out;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=period {
        let d = values[i] - values[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    gain /= period as f64;
    loss /= period as f64;
    let rs = if loss == 0.0 { f64::INFINITY } else { gain / loss };
    out[period] = 100.0 - 100.0 / (1.0 + rs);
    for i in (period + 1)..n {
        let d = values[i] - values[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        gain = (gain * (period as f64 - 1.0) + g) / period as f64;
        loss = (loss * (period as f64 - 1.0) + l) / period as f64;
        let rs = if loss == 0.0 { f64::INFINITY } else { gain / loss };
        out[i] = 100.0 - 100.0 / (1.0 + rs);
    }
    out
}

pub struct Macd {
    pub macd: Vec<f64>,
    pub signal: Vec<f64>,
    pub hist: Vec<f64>,
}

pub fn macd(values: &[f64], fast: usize, slow: usize, signal: usize) -> Macd {
    let ema_fast = ema(values, fast);
    let ema_slow = ema(values, slow);
    let macd_line: Vec<f64> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(f, s)| f - s)
        .collect();
    let signal_line = ema(&macd_line, signal);
    let hist: Vec<f64> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(m, s)| m - s)
        .collect();
    Macd {
        macd: macd_line,
        signal: signal_line,
        hist,
    }
}

pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut tr = vec![0.0; n];
    if n == 0 {
        return vec![];
    }
    tr[0] = high[0] - low[0];
    for i in 1..n {
        let a = high[i] - low[i];
        let b = (high[i] - close[i - 1]).abs();
        let c = (low[i] - close[i - 1]).abs();
        tr[i] = a.max(b).max(c);
    }
    ema(&tr, period)
}

pub struct Bollinger {
    pub mid: Vec<f64>,
    pub upper: Vec<f64>,
    pub lower: Vec<f64>,
}

pub fn bollinger(values: &[f64], window: usize, k: f64) -> Bollinger {
    let mid = sma(values, window);
    let mut upper = vec![f64::NAN; values.len()];
    let mut lower = vec![f64::NAN; values.len()];
    for i in 0..values.len() {
        if i + 1 < window {
            continue;
        }
        let start = i + 1 - window;
        let mean = mid[i];
        let var: f64 = values[start..=i]
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / window as f64;
        let sd = var.sqrt();
        upper[i] = mean + k * sd;
        lower[i] = mean - k * sd;
    }
    Bollinger { mid, upper, lower }
}

pub fn adx(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    if n < period + 1 {
        return out;
    }
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];
    for i in 1..n {
        let up = high[i] - high[i - 1];
        let down = low[i - 1] - low[i];
        plus_dm[i] = if up > down && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if down > up && down > 0.0 { down } else { 0.0 };
        let a = high[i] - low[i];
        let b = (high[i] - close[i - 1]).abs();
        let c = (low[i] - close[i - 1]).abs();
        tr[i] = a.max(b).max(c);
    }
    let atr_s = ema(&tr, period);
    let plus_s = ema(&plus_dm, period);
    let minus_s = ema(&minus_dm, period);
    let mut dx = vec![f64::NAN; n];
    for i in 0..n {
        if atr_s[i].is_nan() || atr_s[i] == 0.0 {
            continue;
        }
        let plus_di = 100.0 * plus_s[i] / atr_s[i];
        let minus_di = 100.0 * minus_s[i] / atr_s[i];
        let sum = plus_di + minus_di;
        if sum == 0.0 {
            dx[i] = 0.0;
        } else {
            dx[i] = 100.0 * (plus_di - minus_di).abs() / sum;
        }
    }
    let adx_s = ema(&dx, period);
    for i in 0..n {
        out[i] = adx_s[i];
    }
    out
}

pub fn last_finite(values: &[f64]) -> Option<f64> {
    values.iter().rev().find(|v| v.is_finite()).copied()
}
