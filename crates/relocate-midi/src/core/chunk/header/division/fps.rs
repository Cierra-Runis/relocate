use derive_more::{Debug, Display, Eq, Error, PartialEq};

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FPS {
    FPS24 = -24,
    FPS25 = -25,
    FPS30Drop = -29,
    FPS30 = -30,
}

#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum TryFromError {
    InvalidFPS,
}

impl TryFrom<u8> for FPS {
    type Error = TryFromError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value as i8 {
            -24 => Ok(FPS::FPS24),
            -25 => Ok(FPS::FPS25),
            -29 => Ok(FPS::FPS30Drop),
            -30 => Ok(FPS::FPS30),
            _ => Err(TryFromError::InvalidFPS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_try_from_valid() {
        const TEST_CASES: &[(u8, FPS)] = &[
            (232, FPS::FPS24),
            (231, FPS::FPS25),
            (227, FPS::FPS30Drop),
            (226, FPS::FPS30),
        ];
        for (input, expected) in TEST_CASES {
            let result = FPS::try_from(*input).unwrap();
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn test_fps_try_from_invalid() {
        const INVALID_INPUTS: &[u8] = &[0, 1, 100, 255];
        for input in INVALID_INPUTS {
            let result = FPS::try_from(*input);
            assert!(matches!(result, Err(TryFromError::InvalidFPS)));
        }
    }
}
