# Example: DS18B20 temperature sensor with Pico

This example shows how to use the DS18B20 temperature sensor with the Raspberry Pi Pico.

> [!NOTE]  
> This crate is dependent on `embedded-hal:1.0`. This example is dependent on
> cortex-m which - at the time of writing - is not yet compatible with `embedded-hal:1.0`. Therefore, this example
> will not work out of the box until `cortex-m` is updated to be compatible with `embedded-hal:1.0`.
> A pull request is already merged in the cortex-m repository, so it should be available soon.
> Though, you can still use this example by using the `main` branch of the `cortex-m` repository.

## Hardware Required

- Raspberry Pi Pico
- DS18B20 temperature sensor
- 4.7kΩ resistor
- Breadboard
- Jumper wires
- Micro USB cable
- Raspberry Pi Debug Probe

## Software Required

- Probe-rs (or any other GDB server; configure the `runner` field in .cargo/config accordingly)

## Circuit Diagram

Connect the DS18B20 temperature sensor to the Raspberry Pi Pico as shown in the following diagram:

![DS18B20 temperature sensor with Pico](../../../assets/ds18b20-pico.png)

## Run the Example

Use the following command to run the example:

```bash
# from the example/rp-pico-ds18b20 directory
cargo run
```
