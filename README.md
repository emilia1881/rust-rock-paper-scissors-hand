# RoboHand — Rock Paper Scissors Robot

A robotic hand that plays Rock Paper Scissors against a human, using computer vision for gesture detection and embedded Rust for the game logic and hardware control.

<img width="3024" height="4032" alt="IMG_8277" src="https://github.com/user-attachments/assets/148c1eb0-9b72-49ae-8e20-6425c5a1b910" />

## How It Works

1. A laptop camera detects the player's gesture using Python + MediaPipe.
2. The gesture is sent over WiFi (HTTP) to an ESP32, which acts as its own Access Point and runs the game logic.
3. The ESP32 randomly picks the robot's move and sends it to an STM32 over UART.
4. The STM32 moves 4 servos to physically show the gesture and displays the result on an OLED screen.
5. A live web page (served by the ESP32) shows the score in real time.

## Components

- **ESP32-WROOM-32** — WiFi Access Point, HTTP server, game logic (Rust)
- **STM32U545RE** — servo control (PWM), OLED display (I2C) (Rust, Embassy)
- **Python script** — hand gesture detection (OpenCV + MediaPipe)

## Protocols

- WiFi (Access Point) — ESP32 creates its own network, no router needed
- HTTP — gesture and score exchange between Python, browser, and ESP32
- UART — ESP32 → STM32 communication (115200 baud)
- PWM — servo control on STM32 (50Hz, 700–2200µs pulses)
- I2C — OLED display on STM32

## Wiring

| From | To |
|---|---|
| ESP32 GPIO17 (TX) | STM32 PA3 / D0 (RX) |
| ESP32 GND | STM32 GND |
| OLED SCL / SDA | STM32 PB6 / PB7 |
| Servos | STM32 PC6, PC7, PC8, PC9 |

## How to Run

```bash
# 1. Flash STM32
cd stm32-servo
cargo run

# 2. Flash ESP32
cd esp32-robohand
cargo run

# 3. Connect laptop to WiFi "RoboHand" (password: robohand123)
# 4. Open http://192.168.71.1 and enter a name
# 5. Run the Python script
python hand_test.py

# 6. Press SPACE, show your gesture, play!
```
