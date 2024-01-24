use one_wire_bus::OneWireError;

pub type DS2482Result<T, E> = Result<T, DS2482Error<E>>;

#[derive(Debug, Copy, Clone)]
pub enum DS2482Error<E> {
    DeviceResetError,
    I2CCommunicationError(E),
    WriteConfigError,
}

impl<E> From<E> for DS2482Error<E> {
    fn from(err: E) -> DS2482Error<E> {
        DS2482Error::I2CCommunicationError(err)
    }
}

impl<E> From<DS2482Error<E>> for OneWireError<E> {
    fn from(err: DS2482Error<E>) -> Self {
        match err {
            DS2482Error::DeviceResetError => OneWireError::InitializationError,
            DS2482Error::I2CCommunicationError(err) => OneWireError::CommunicationError(err),
            DS2482Error::WriteConfigError => OneWireError::InitializationError,
        }
    }
}

