use derive_more::{Debug, Display, Eq, Error, PartialEq};

#[derive(Debug, Display, PartialEq, Eq)]
pub enum Fps {
    FPS24 = -24,
    FPS25 = -25,
    FPS30Drop = -29,
    FPS30 = -30,
}

#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum TryFromError {
    InvalidFPS,
}

impl TryFrom<u8> for Fps {
    type Error = TryFromError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value as i8 {
            -24 => Ok(Fps::FPS24),
            -25 => Ok(Fps::FPS25),
            -29 => Ok(Fps::FPS30Drop),
            -30 => Ok(Fps::FPS30),
            _ => Err(TryFromError::InvalidFPS),
        }
    }
}
