use one_wire_bus::OneWireError;

pub type DS2482Result<T, E> = Result<T, DS2482Error<E>>;

#[derive(Debug, Copy, Clone)]
pub enum DS2482Error<E> {
    DeviceResetFailed,
    I2CCommunicationError(E),
    WriteConfigFailed,
}

impl<E> From<E> for DS2482Error<E> {
    fn from(err: E) -> DS2482Error<E> {
        DS2482Error::I2CCommunicationError(err)
    }
}

impl<E> From<DS2482Error<E>> for OneWireError {
    fn from(err: DS2482Error<E>) -> OneWireError {
        match err {
            DS2482Error::DeviceResetFailed => OneWireError::BusError,
            DS2482Error::I2CCommunicationError(_) => OneWireError::BusError,
            DS2482Error::WriteConfigFailed => OneWireError::BusError,
        }
    }
}
