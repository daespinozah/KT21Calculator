use core::hash::Hash;
use num::traits::Num;
use std::collections::HashMap;
use wasm_bindgen::JsValue;

pub trait ToJsMap {
    fn to_js_map(&self) -> js_sys::Map;
}

impl<KeyType, ValType> ToJsMap for HashMap<KeyType, ValType>
where
    JsValue: From<KeyType> + From<ValType>,
    KeyType: Copy,
    ValType: Copy,
{
    fn to_js_map(&self) -> js_sys::Map {
        let js_map = js_sys::Map::new();
        for (key, val) in self.iter() {
            js_map.set(&JsValue::from(*key), &JsValue::from(*val));
        }
        js_map
    }
}

/*
impl ToJsMap for HashMap<i32, f64> {
    fn to_js_map(&self) -> js_sys::Map {
        let js_map = js_sys::Map::new();
        for (key, val) in self.iter() {
            js_map.set(&JsValue::from(*key), &JsValue::from(*val));
        }
        js_map
    }
}
*/

pub fn add_to_map_value<KeyType: Eq + PartialEq + Hash + Copy, ValType: Num + Copy>(
    map: &mut HashMap<KeyType, ValType>,
    key: &KeyType,
    val: ValType,
) {
    let zero = ValType::zero();
    let old_map_val = map.get(key).unwrap_or(&zero);
    map.insert(*key, *old_map_val + val);
}

#[allow(dead_code)]
pub fn normalize_map_values<
    KeyType: Eq + PartialEq + Hash + Copy,
    ValType: std::ops::DivAssign + Copy,
>(
    map: &mut HashMap<KeyType, ValType>,
    divisor: ValType,
) {
    for (_key, val) in map.iter_mut() {
        *val /= divisor;
    }
}

pub fn binomial_pmf(num_trials: i32, num_successes: i32, prob_success: f64) -> f64 {
    // often the variables are named numTrials=n, numSuccesses=k, probSuccess=p
    return
        // combinations(num_trials, num_successes) as f64
        n_choose_k(num_trials, num_successes) as f64
        * prob_success.powf(num_successes.into())
        * (1.0 - prob_success).powf((num_trials - num_successes).into());
}

// can only handle up to num_trials=29 (29*28*..*16 < max i64 < 30*29*..*16)
//
// with s=min(k, n-k), this does 2*s-1 multiplications and 1 division;
// other implementations do way more operations to handle bigger numbers ...
// - https://docs.rs/compute/latest/compute/functions/fn.binom_coeff.html
//   - does 2*s multiplications and 5*s divisions (2*s divisions are for overflow checking)
// - https://docs.rs/num-integer/latest/num_integer/fn.binomial.html
//   - does the recursive implementation; probably roughly same number of operations as above
//
pub fn n_choose_k(n: i32, k: i32) -> i64 {
    // often the variables are named numTrials=n, numSuccesses=k
    const MAX_NUM_TRIALS: usize = 29;
    static mut LOOKUP_TABLE: [[i64; MAX_NUM_TRIALS]; MAX_NUM_TRIALS + 1] =
        [[0; MAX_NUM_TRIALS]; MAX_NUM_TRIALS + 1];

    let lookup_entry: &mut i64;
    // unsafe to access a static mut variable because it's not thread-safe; we have only one thread
    unsafe {
        lookup_entry = &mut LOOKUP_TABLE[n as usize - 1][k as usize];
    }
    if *lookup_entry != 0 {
        return *lookup_entry;
    }

    // let [smaller_divisor, bigger_divisor] = std::cmp::minmax(n, n - k); // requires rustc nightly
    let n_minus_k = n - k;
    let [smaller_divisor, bigger_divisor] = if k < n_minus_k {
        [k, n_minus_k]
    } else {
        [n_minus_k, k]
    };

    let mut numerator = 1i64;

    for numerator_factor in (bigger_divisor + 1)..=n {
        numerator *= numerator_factor as i64;
    }

    let mut denominator = 1i64;

    for denominator_factor in 2..=smaller_divisor {
        denominator *= denominator_factor as i64;
    }

    let result = numerator / denominator;
    *lookup_entry = result;
    result
}

pub fn calc_multi_round_damage(
    single_round_dmg_probs: &HashMap<i32, f64>,
    num_rounds: i32,
) -> HashMap<i32, f64> {
    let mut latest_round_dmg_probs = single_round_dmg_probs.clone();
    let mut prev_round_dmg_probs = HashMap::<i32, f64>::new();

    for _round_number in 2..=num_rounds {
        // swap, then clear the "frontier" map
        [prev_round_dmg_probs, latest_round_dmg_probs] =
            [latest_round_dmg_probs, prev_round_dmg_probs];
        latest_round_dmg_probs.clear();

        for (prev_rounds_dmg, prev_rounds_prob) in prev_round_dmg_probs.iter() {
            for (single_round_dmg, single_round_prob) in single_round_dmg_probs.iter() {
                add_to_map_value(
                    &mut latest_round_dmg_probs,
                    &(prev_rounds_dmg + single_round_dmg),
                    prev_rounds_prob * single_round_prob,
                );
            }
        }
    }
    return latest_round_dmg_probs;
}
