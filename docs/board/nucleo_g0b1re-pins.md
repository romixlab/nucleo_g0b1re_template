# NUCLEO-G0B1RE · pins

Every MCU pin of the STM32G0B1RET6 (U7) (MB1360 rev C02): where it is on the headers, what else sits on its net, and whether an application may use it. Roles: **free** reaches the headers and nothing else; **shared** has a board job by default that a solder bridge or jumper removes; **taken** is the debug port or reset; **function** is a crystal, VBAT or VREF pin.

| Pin | Arduino | Headers | Also on the net | Role | What it does on the board |
|---|---|---|---|---|---|
| `PA0` | A0 | CN7-28, CN8-1 |  | 🟢 free |  |
| `PA1` | A1 | CN7-30, CN8-2 |  | 🟢 free |  |
| `PA2` |  | CN10-34 | JP6, SB18, U2 | 🟠 shared | USART2 TX to the ST-LINK virtual COM port (SB18 ON by default). A GPIO on CN10-34 only with SB18 OFF; then JP6 [1-2] keeps the VCP from floating. |
| `PA3` |  | CN10-6 | JP6, SB16, U2 | 🟠 shared | USART2 RX from the ST-LINK virtual COM port (SB16 ON by default). A GPIO on CN10-6 only with SB16 OFF. |
| `PA4` | A2 | CN7-32, CN8-3 |  | 🟢 free |  |
| `PA5` | D13 SCK | CN10-11, CN5-6 | SB22 | 🟠 shared | Drives the user LED LD4 through T2 (SB22 ON by default); also Arduino D13 / SPI1 SCK. SB22 OFF isolates the LED. |
| `PA6` | D12 MISO | CN10-13, CN5-5 |  | 🟢 free |  |
| `PA7` | D11 MOSI | CN10-15, CN5-4 |  | 🟢 free |  |
| `PA8` | D7 | CN10-23, CN9-8 |  | 🟢 free |  |
| `PA9` | D8 | CN10-21, CN5-1 |  | 🟢 free |  |
| `PA10` | D2 | CN10-33, CN9-3 |  | 🟢 free |  |
| `PA11` |  | CN10-14 |  | 🟢 free | The G0B1's USB DM on CN10-14; no USB connector on the board, wire one here for a USB device. |
| `PA12` |  | CN10-12 |  | 🟢 free | The G0B1's USB DP on CN10-12; no USB connector on the board. |
| `PA13` |  | CN7-13, CN4-4 | SB12, SB13 | 🔴 taken | ST-LINK SWDIO through SB13 and the CN4 [3-4] jumper. Only free with both CN4 jumpers off, i.e. no on-board debugger. |
| `PA14` |  | CN7-7, CN4-2 | SB8, SB9 | 🔴 taken | ST-LINK SWCLK through SB9 and the CN4 [1-2] jumper; also BOOT0. Only free with the CN4 jumpers off. |
| `PA15` |  | CN7-17 |  | 🟢 free |  |
| `PB0` | D10 CS | CN10-17, CN5-3 |  | 🟢 free |  |
| `PB1` | A3 | CN7-34, CN8-4 |  | 🟢 free |  |
| `PB2` |  | CN10-22 |  | 🟢 free |  |
| `PB3` | D3 | CN10-31, CN9-4 |  | 🟢 free |  |
| `PB4` | D5 | CN10-27, CN9-6 |  | 🟢 free |  |
| `PB5` | D4 | CN10-29, CN9-5 |  | 🟢 free |  |
| `PB6` |  | CN10-24 |  | 🟢 free |  |
| `PB7` |  | CN7-21 |  | 🟢 free |  |
| `PB8` | D15 SCL | CN10-3, CN5-10 | SB5 | 🟢 free | Arduino D15 / I2C1 SCL on CN5-10. SB5 ON (default OFF) also ties it to CN8-6 (A5) for I2C shields. |
| `PB9` | D14 SDA | CN10-5, CN5-9 | SB2 | 🟢 free | Arduino D14 / I2C1 SDA on CN5-9. SB2 ON (default OFF) also ties it to CN8-5 (A4). |
| `PB10` |  | CN10-28 |  | 🟢 free |  |
| `PB11` | A4·SB4 | CN7-36, CN8-5 | SB2, SB4 | 🟢 free | Arduino A4 (ADC IN15) on CN8-5 and CN7-36 through SB4 (ON). SB4 OFF + SB2 ON puts PB9 there instead, for I2C shields. |
| `PB12` | A5·SB3 | CN7-38, CN8-6 | SB3, SB5 | 🟢 free | Arduino A5 (ADC IN16) on CN8-6 and CN7-38 through SB3 (ON). SB3 OFF + SB5 ON puts PB8 there instead, for I2C shields. |
| `PB13` |  | CN10-30 |  | 🟢 free |  |
| `PB14` | D6 | CN10-25, CN9-7 |  | 🟢 free |  |
| `PB15` |  | CN10-26 |  | 🟢 free |  |
| `PC0` |  | CN10-18 |  | 🟢 free |  |
| `PC1` |  | CN10-16 |  | 🟢 free |  |
| `PC2` |  | CN7-35 |  | 🟢 free |  |
| `PC3` |  | CN7-37 |  | 🟢 free |  |
| `PC4` | D1 TX | CN10-35, CN9-2 |  | 🟢 free |  |
| `PC5` | D0 RX | CN10-37, CN9-1 |  | 🟢 free |  |
| `PC6` |  | CN10-4 |  | 🟢 free |  |
| `PC7` | D9 | CN10-19, CN5-2 |  | 🟢 free |  |
| `PC8` |  | CN10-2 |  | 🟢 free |  |
| `PC9` |  | CN10-1 |  | 🟢 free |  |
| `PC10` |  | CN7-1 |  | 🟢 free |  |
| `PC11` |  | CN7-2 |  | 🟢 free |  |
| `PC12` |  | CN7-3 |  | 🟢 free |  |
| `PC13` |  | CN7-23 | B1, SB20 | 🟠 shared | USER button B1 to GND with a pull-up (SB20 ON by default); SB20 OFF frees it. Also on CN7-23. |
| `PC14` |  | — | SB23 | 🟣 function | 32.768 kHz crystal X2 (OSC32_IN). Not on the headers: SB23 OFF by default; SB23 ON + R31 out puts it on CN7-25 as a GPIO. |
| `PC15` |  | — | SB24 | 🟣 function | 32.768 kHz crystal X2 (OSC32_OUT). SB24 OFF by default; SB24 ON + R32 out puts it on CN7-27 as a GPIO. |
| `PD0` |  | CN7-9 |  | 🟢 free |  |
| `PD1` |  | CN7-10 |  | 🟢 free |  |
| `PD2` |  | CN7-4 |  | 🟢 free |  |
| `PD3` |  | CN7-11 |  | 🟢 free |  |
| `PD4` |  | CN7-15 |  | 🟢 free |  |
| `PD5` |  | CN7-26 |  | 🟢 free |  |
| `PD6` |  | CN10-10 |  | 🟢 free |  |
| `PD8` |  | CN10-36 | SB30 | 🟢 free | GPIO on CN10-36. SB30 ON (default OFF) ties it to GND, the 'power reserved' option in UM2324. |
| `PD9` |  | CN10-38 | SB29 | 🟢 free | GPIO on CN10-38. SB29 ON (default OFF) ties it to VDD. |
| `PF0` |  | CN7-29 | SB17, SB25 | 🟠 shared | OSC_IN: X3 is not fitted; SB17 ON feeds the ST-LINK's 8 MHz MCO here (an HSE bypass clock), SB25 ON puts it on CN7-29. For a GPIO: SB17 OFF. |
| `PF1` |  | CN7-31 | SB27 | 🟢 free | OSC_OUT of the unfitted X3; on CN7-31 through SB27 (ON). A GPIO as the board ships. |
| `PF2` |  | CN7-14, CN6-3 | B2, SB19, U2 | 🔴 taken | Reset: B2 button, the ST-LINK's T_NRST through SB19, Arduino CN6-3. Not a GPIO on this board. |

## Solder bridges

| Bridge | Default | Side | Meaning |
|---|---|---|---|
| SB1 | ON | top | U6 3V3 out |
| SB10 | OFF | bottom | SWDIO ext |
| SB11 | ON | bottom | SWDIO |
| SB12 | OFF | bottom | SWDIO ext |
| SB13 | ON | bottom | SWDIO PA13 |
| SB14 | ON | bottom | SWO |
| SB15 | OFF | top | CN2 100 mA |
| SB16 | ON | bottom | VCP TX→PA3 |
| SB17 | ON | bottom | MCO→PF0 |
| SB18 | ON | bottom | PA2→VCP RX |
| SB19 | ON | bottom | T_NRST |
| SB2 | OFF | top | PB9→A4 |
| SB20 | ON | bottom | B1→PC13 |
| SB21 | OFF | bottom | AVDD→CN5-8 |
| SB22 | ON | bottom | PA5→LD4 |
| SB23 | OFF | bottom | PC14→CN7-25 |
| SB24 | OFF | bottom | PC15→CN7-27 |
| SB25 | ON | bottom | PF0→CN7-29 |
| SB26 | ON | bottom | VDD→VBAT |
| SB27 | ON | bottom | PF1→CN7-31 |
| SB28 | ON | bottom | VDD→AVDD |
| SB29 | OFF | bottom | PD9→VDD |
| SB3 | ON | top | PB12→A5 |
| SB30 | OFF | bottom | PD8→GND |
| SB31 | ON | bottom | AGND→GND |
| SB4 | ON | top | PB11→A4 |
| SB5 | OFF | top | PB8→A5 |
| SB6 | OFF | bottom | SWCLK ext |
| SB7 | ON | bottom | SWCLK |
| SB8 | OFF | bottom | SWCLK ext |
| SB9 | ON | bottom | SWCLK PA14 |

Sources: UM2324 Rev 5; MB1360-G0B1RE-C02 schematic. Generated by boardmap (bedrock_board) from `ST MB1360 rev C Altium board file (mb1360_bdp.zip), kicad-cli 10 import`.
