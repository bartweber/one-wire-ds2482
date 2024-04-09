#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use one_wire_hal::address::Address;
use one_wire_hal::device_search::DeviceSearch;
use one_wire_hal::OneWire;
use one_wire_hal::triplet::Triplet;

use crate::error::Error;

pub mod error;

/// Command "Device Reset", F0h
const COMMAND_DRST: u8 = 0xF0;

/// Command "Write Configuration", D2h
const COMMAND_WCFG: u8 = 0xD2;

/// Command "1-Wire Reset", B4h
pub const COMMAND_1WRS: u8 = 0xB4;

/// Command "1-Wire Single Bit", 87h
pub const COMMAND_1WSB: u8 = 0x87;

/// Command "1-Wire Read Byte", 96h
pub const COMMAND_1WRB: u8 = 0x96;

/// Command "1-Wire Write Byte", A5h
pub const COMMAND_1WWB: u8 = 0xA5;

/// Command "Triplet", 78h
const COMMAND_TRIPLET: u8 = 0x78;

/// Command "Set Read Pointer", E1h
const COMMAND_SRP: u8 = 0xE1;


/// Read Pointer "Status Register", F0h
const POINTER_STATUS: u8 = 0xF0;

/// Read Pointer "Data Register", E1h
const POINTER_DATA: u8 = 0xE1;

/// Read Pointer "Configuration Register", C3h
// const POINTER_CONFIG: u8 = 0xC3;


const STATUS_1WB: u8 = 1 << 0;
const STATUS_PPD: u8 = 1 << 1;
const STATUS_SD: u8 = 1 << 2;
// const STATUS_LL: u8 = 1 << 3;
const STATUS_RST: u8 = 1 << 4;
const STATUS_SBR: u8 = 1 << 5;
const STATUS_TSB: u8 = 1 << 6;
const STATUS_DIR: u8 = 1 << 7;

pub struct OneWireDS2482<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C: I2c> one_wire_hal::error::ErrorType for OneWireDS2482<I2C> {
    type Error = Error;
}

impl<I2C: I2c> OneWireDS2482<I2C> {
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self { i2c, addr }
    }

    /// Perform a device reset on the DS2482
    /// Should be called after power up or after a communication error
    pub fn ds2482_device_reset(&mut self) -> Result<(), Error> {
        let mut rx: [u8; 1] = [0];
        self.i2c.write_read(self.addr, &[COMMAND_DRST], &mut rx)
            .map_err(|_| Error::I2CCommunicationError)?;
        let status = rx[0];
        if (status & STATUS_RST) == 0 {
            return Err(Error::DeviceResetError);
        }

        Ok(())
    }

    /// Write the configuration register
    ///
    /// Configuration options: 1WS, SPU, 0, APU
    /// 1WS: 1-wire speed; 0 = standard speed, 1 = overdrive speed
    /// SPU: strong pullup; 0 = disabled, 1 = enabled
    /// n/a: reserved; must be 0
    /// APU: active pullup; 0 = disabled, 1 = enabled
    pub fn ds2482_write_config(&mut self, config: u8) -> Result<(), Error> {
        let config_byte: u8 = config | (!config << 4);
        let mut rx: [u8; 1] = [0];

        self.i2c.write_read(self.addr, &[COMMAND_WCFG, config_byte], &mut rx)
            .map_err(|_| Error::I2CCommunicationError)?;

        let read_config = rx[0];
        if config != read_config {
            self.ds2482_device_reset()?;
            return Err(Error::WriteConfigError);
        }

        Ok(())
    }

    /// Read the status register consisting of 8 bits
    ///
    /// DIR TSB  SBR  RST  LL  SD  PPD  1WB
    pub fn ds2482_read_status(&mut self) -> Result<u8, Error> {
        self.ds2482_set_read_pointer(POINTER_STATUS)?;
        self.ds2482_read_byte()
    }

    /// Read the data register
    pub fn ds2482_read_data_register(&mut self) -> Result<u8, Error> {
        self.ds2482_set_read_pointer(POINTER_DATA)?;
        self.ds2482_read_byte()
    }

    /// Wait for the DS2482 to be ready
    /// This is done by polling the status register
    /// until the 1WB bit is cleared
    ///
    /// Returns the status register
    fn ds2482_wait_on_busy(&mut self, delay: &mut impl DelayNs) -> Result<u8, Error> {
        let mut status = 0;

        let mut poll_count = 0;
        while poll_count < 1000 {
            status = self.ds2482_read_status()?;
            if (status & STATUS_1WB) == 0 {
                break;
            }
            poll_count = poll_count + 1;

            delay.delay_us(20);
        }

        Ok(status)
    }

    fn ds2482_set_read_pointer(&mut self, read_pointer: u8) -> Result<(), Error> {
        self.i2c.write(self.addr, &[COMMAND_SRP, read_pointer])
            .map_err(|_| Error::I2CCommunicationError)?;
        Ok(())
    }

    pub fn ds2482_read_byte(&mut self) -> Result<u8, Error> {
        let mut rx: [u8; 1] = [0];
        self.i2c.read(self.addr, &mut rx)
            .map_err(|_| Error::I2CCommunicationError)?;
        Ok(rx[0])
    }

    pub fn ds2482_write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.i2c.write(self.addr, bytes)
            .map_err(|_| Error::I2CCommunicationError)?;
        Ok(())
    }
}

impl<I2C: I2c> OneWire for OneWireDS2482<I2C> {
    fn reset(&mut self, delay: &mut impl DelayNs) -> Result<bool, Error> {
        self.ds2482_wait_on_busy(delay)?;

        // TODO: please implement
        // clear_strong_pullup();

        // self._wait_on_busy(delay);

        // 1-wire reset command
        self.ds2482_write_bytes(&[COMMAND_1WRS])?;

        let status = self.ds2482_wait_on_busy(delay)?;

        if (status & STATUS_SD) != 0 {
            return Err(Error::ShortDetected);
        }

        Ok((status & STATUS_PPD) != 0)
    }

    fn read_bit(&mut self, delay: &mut impl DelayNs) -> Result<bool, Error> {
        // Bit 7 is what matters and in this case we need it to be a 1
        // this will generate a read-data time slot
        let bit_byte = 0xFF;
        self.ds2482_wait_on_busy(delay)?;
        self.ds2482_write_bytes(&[COMMAND_1WSB, bit_byte])?;
        self.ds2482_wait_on_busy(delay)?;
        let status = self.ds2482_read_status()?;
        Ok((status & STATUS_SBR) != 0)
    }

    fn read_byte(&mut self, delay: &mut impl DelayNs) -> Result<u8, Error> {
        self.ds2482_wait_on_busy(delay)?;
        self.ds2482_write_bytes(&[COMMAND_1WRB])?;
        self.ds2482_wait_on_busy(delay)?;
        Ok(self.ds2482_read_data_register()?)
    }

    fn write_bit(&mut self, bit: bool, delay: &mut impl DelayNs) -> Result<(), Error> {
        // set bit_byte by setting bit 7 to 1 or 0
        let bit_byte = if bit { 0xFF } else { 0x00 };
        self.ds2482_wait_on_busy(delay)?;
        Ok(self.ds2482_write_bytes(&[COMMAND_1WSB, bit_byte])?)
    }

    fn write_byte(&mut self, value: u8, delay: &mut impl DelayNs) -> Result<(), Error> {
        self.ds2482_wait_on_busy(delay)?;
        Ok(self.ds2482_write_bytes(&[COMMAND_1WWB, value])?)
    }

    fn triplet(&mut self, dir_bit: bool, delay: &mut impl DelayNs) -> Result<Triplet, Error> {
        let dir_byte = if dir_bit { 0xFF } else { 0x00 };
        self.ds2482_wait_on_busy(delay)?;
        self.ds2482_write_bytes(&[COMMAND_TRIPLET, dir_byte])?;
        let status = self.ds2482_wait_on_busy(delay)?;
        let bit = (status & STATUS_SBR) != 0;
        let complement_bit = (status & STATUS_TSB) != 0;
        let direction_bit = (status & STATUS_DIR) != 0;
        Ok(Triplet::new(bit, complement_bit, direction_bit))
    }

    fn devices<'a>(&'a mut self, delay: &'a mut impl DelayNs) -> impl Iterator<Item=Result<Address, Self::Error>> + 'a {
        DeviceSearch::new(false, self, delay)
    }

    fn alarming_devices<'a>(&'a mut self, delay: &'a mut impl DelayNs) -> impl Iterator<Item=Result<Address, Self::Error>> + 'a {
        DeviceSearch::new(true, self, delay)
    }
}
