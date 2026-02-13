#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_line_delays_by_one_sample() {
        let mut delay = DelayLine::new(8);
        delay.set_delay_samples(1);

        let out0 = delay.process(1.0);
        let out1 = delay.process(0.0);

        assert_eq!(out0, 0.0);
        assert_eq!(out1, 1.0);
    }

    #[test]
    fn comb_feedback_accumulates_energy() {
        let mut comb = FeedbackComb::new(DelayLine::new(8));
        comb.set_delay_samples(1);
        comb.set_feedback(0.9);

        let first = comb.process(1.0);
        let second = comb.process(0.0);

        assert!(first.abs() <= 1.0);
        assert!(second.abs() > 0.0);
    }

    #[test]
    fn fractional_delay_matches_ramp() {
        let mut delay = FractionalDelayLine::new(16);
        delay.set_delay_samples(2.5);

        let mut outputs = Vec::new();
        for index in 0..12 {
            let out = delay.process(index as f32);
            outputs.push(out);
        }

        for (index, out) in outputs.into_iter().enumerate().skip(3) {
            let expected = index as f32 - 2.5;
            assert!((out - expected).abs() < 0.001);
        }
    }
}
