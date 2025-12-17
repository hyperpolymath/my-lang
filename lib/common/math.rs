//! Common Math Operations
//!
//! Generic mathematical functions and constants.

/// Mathematical constants
pub mod constants {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const TAU: f64 = std::f64::consts::TAU;
    pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
    pub const LN_2: f64 = std::f64::consts::LN_2;
    pub const LN_10: f64 = std::f64::consts::LN_10;
}

/// Absolute value for integers
pub fn abs_int(n: i64) -> i64 {
    n.abs()
}

/// Absolute value for floats
pub fn abs_float(f: f64) -> f64 {
    f.abs()
}

/// Minimum of two integers
pub fn min_int(a: i64, b: i64) -> i64 {
    a.min(b)
}

/// Maximum of two integers
pub fn max_int(a: i64, b: i64) -> i64 {
    a.max(b)
}

/// Minimum of two floats
pub fn min_float(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Maximum of two floats
pub fn max_float(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Floor of a float
pub fn floor(f: f64) -> i64 {
    f.floor() as i64
}

/// Ceiling of a float
pub fn ceil(f: f64) -> i64 {
    f.ceil() as i64
}

/// Round to nearest integer
pub fn round(f: f64) -> i64 {
    f.round() as i64
}

/// Truncate towards zero
pub fn trunc(f: f64) -> i64 {
    f.trunc() as i64
}

/// Square root
pub fn sqrt(f: f64) -> f64 {
    f.sqrt()
}

/// Cube root
pub fn cbrt(f: f64) -> f64 {
    f.cbrt()
}

/// Power (float base, integer exponent)
pub fn powi(base: f64, exp: i32) -> f64 {
    base.powi(exp)
}

/// Power (float base, float exponent)
pub fn powf(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Integer power
pub fn pow_int(base: i64, exp: u32) -> i64 {
    base.pow(exp)
}

/// Modulo for integers
pub fn mod_int(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else {
        Some(a % b)
    }
}

/// Modulo for floats
pub fn mod_float(a: f64, b: f64) -> f64 {
    a % b
}

/// Sine
pub fn sin(f: f64) -> f64 {
    f.sin()
}

/// Cosine
pub fn cos(f: f64) -> f64 {
    f.cos()
}

/// Tangent
pub fn tan(f: f64) -> f64 {
    f.tan()
}

/// Arcsine
pub fn asin(f: f64) -> f64 {
    f.asin()
}

/// Arccosine
pub fn acos(f: f64) -> f64 {
    f.acos()
}

/// Arctangent
pub fn atan(f: f64) -> f64 {
    f.atan()
}

/// Arctangent of y/x
pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Hyperbolic sine
pub fn sinh(f: f64) -> f64 {
    f.sinh()
}

/// Hyperbolic cosine
pub fn cosh(f: f64) -> f64 {
    f.cosh()
}

/// Hyperbolic tangent
pub fn tanh(f: f64) -> f64 {
    f.tanh()
}

/// Natural logarithm
pub fn ln(f: f64) -> f64 {
    f.ln()
}

/// Base 2 logarithm
pub fn log2(f: f64) -> f64 {
    f.log2()
}

/// Base 10 logarithm
pub fn log10(f: f64) -> f64 {
    f.log10()
}

/// Arbitrary base logarithm
pub fn log(f: f64, base: f64) -> f64 {
    f.log(base)
}

/// Exponential (e^x)
pub fn exp(f: f64) -> f64 {
    f.exp()
}

/// 2^x
pub fn exp2(f: f64) -> f64 {
    f.exp2()
}

/// Hypotenuse (sqrt(x^2 + y^2))
pub fn hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

/// Sign of a number (-1, 0, or 1)
pub fn signum_int(n: i64) -> i64 {
    n.signum()
}

/// Sign of a float (-1.0, 0.0, or 1.0)
pub fn signum_float(f: f64) -> f64 {
    f.signum()
}

/// Check if float is NaN
pub fn is_nan(f: f64) -> bool {
    f.is_nan()
}

/// Check if float is infinite
pub fn is_infinite(f: f64) -> bool {
    f.is_infinite()
}

/// Check if float is finite
pub fn is_finite(f: f64) -> bool {
    f.is_finite()
}

/// Clamp integer to range
pub fn clamp_int(n: i64, min: i64, max: i64) -> i64 {
    n.clamp(min, max)
}

/// Clamp float to range
pub fn clamp_float(f: f64, min: f64, max: f64) -> f64 {
    f.clamp(min, max)
}

/// Greatest common divisor
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Least common multiple
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a.abs() / gcd(a, b)) * b.abs()
    }
}

/// Factorial (for small numbers)
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Degrees to radians
pub fn to_radians(degrees: f64) -> f64 {
    degrees.to_radians()
}

/// Radians to degrees
pub fn to_degrees(radians: f64) -> f64 {
    radians.to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(abs_int(-5), 5);
        assert_eq!(abs_int(5), 5);
        assert!((abs_float(-3.14) - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(min_int(3, 7), 3);
        assert_eq!(max_int(3, 7), 7);
    }

    #[test]
    fn test_pow() {
        assert_eq!(pow_int(2, 10), 1024);
        assert!((powf(2.0, 0.5) - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_gcd_lcm() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(lcm(4, 6), 12);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(0), 1);
    }
}
