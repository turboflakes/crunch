// The MIT License (MIT)
// Copyright © 2021 Aukbit Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub fn mean(list: &Vec<f64>) -> f64 {
    if list.len() == 0 {
        return 0.0;
    }
    let sum: f64 = list.iter().sum();
    sum / (list.len() as f64)
}

pub fn standard_deviation(list: &Vec<f64>) -> f64 {
    let m = mean(list);
    let mut variance: Vec<f64> =
        list.iter().map(|&score| (score - m).powf(2.0)).collect();
    mean(&mut variance).sqrt()
}

pub fn median(list: &mut Vec<u32>) -> u32 {
    if list.len() == 0 {
        return 0;
    }
    list.sort();
    let mid = list.len() / 2;
    list[mid]
}

// Calculate 95% confidence interval
pub fn _confidence_interval_95(list: &Vec<f64>) -> (f64, f64) {
    confidence_interval(list, 1.96)
}

// Calculate 99% confidence interval
pub fn _confidence_interval_99(list: &Vec<f64>) -> (f64, f64) {
    confidence_interval(list, 2.576)
}

// Calculate 99.9% confidence interval
pub fn confidence_interval_99_9(list: &Vec<f64>) -> (f64, f64) {
    confidence_interval(list, 3.291)
}

// https://www.mathsisfun.com/data/confidence-interval.html
pub fn confidence_interval(list: &Vec<f64>, z: f64) -> (f64, f64) {
    let m = mean(list);
    let sd = standard_deviation(list);
    let v = z * (sd / ((list.len() as f64).sqrt()));
    (m - v, m + v)
}
// Find outliers by Interquartile Range(IQR)
// https://www.statisticshowto.com/statistics-basics/find-outliers/
pub fn iqr_interval(list: &mut Vec<u32>) -> (f64, f64) {
    if list.len() == 0 {
        return (0.0, 0.0);
    }
    list.sort();
    let q1 = median(&mut (&list[..&list.len() / 2]).into());
    let q3 = median(&mut (&list[&list.len() - (&list.len() / 2)..]).into());
    let iqr = q3 - q1;
    (
        (q1 as f64) - (iqr as f64 * 1.5),
        (q3 as f64) + (iqr as f64 * 1.5),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_mean() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 2.0, 6.0];
        assert_eq!(mean(&v), 3.375);
    }

    #[test]
    fn calculate_median() {
        let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(median(&mut v), 5);
    }

    #[test]
    fn calculate_confidence_interval_99_9() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 2.0, 6.0];
        assert_eq!(
            confidence_interval_99_9(&v),
            (1.5410332231275632, 5.208966776872437)
        );
    }

    #[test]
    fn calculate_iqr_interval() {
        let mut v = vec![1, 2, 3, 4, 5, 4, 2, 6, 3];
        assert_eq!(iqr_interval(&mut v), (-2.5, 9.5));
    }
}
