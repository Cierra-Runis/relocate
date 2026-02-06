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
