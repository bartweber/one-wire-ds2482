use one_wire_hal::error::ErrorKind;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    DeviceResetError,
    I2CCommunicationError,
    WriteConfigError,
    ShortDetected,
}

impl one_wire_hal::error::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::DeviceResetError => ErrorKind::Other,
            Error::I2CCommunicationError => ErrorKind::Other,
            Error::WriteConfigError => ErrorKind::Other,
            Error::ShortDetected => ErrorKind::Other,
        }
    }
}