#include <Arduino.h>
#include <Servo.h>

// =============================================================================
// V.I.S.O.R. - Arduino Uno Dispenser Controller
// =============================================================================
// Serial Protocol:
//  - Baud Rate: 9600
//  - Command Format: <DISP:b,a>\n
//    - b: Bandage (1 = dispense, 0 = hold)
//    - a: Alcohol Pad (1 = dispense, 0 = hold)
//    // - g: Gauze Pad (1 = dispense, 0 = hold) [DISABLED: 2-item dispensing only]
// =============================================================================

// --- Pin Definitions ---
const uint8_t PIN_SERVO_BANDAGE = 9;
const uint8_t PIN_SERVO_ALCOHOL = 10;
// const uint8_t PIN_SERVO_GAUZE = 11; // [DISABLED: 2-item dispensing only]
const uint8_t PIN_LED_STATUS = 13;

const int SERVO_STOP = 90;
const int SERVO_FORWARD = 45;
const int SERVO_REVERSE = 135;

const unsigned long TIME_PUSH_MS = 2200;
const unsigned long TIME_PAUSE_MS = 150;
const unsigned long TIME_RETRACT_MS = 2300;

Servo servoBandage;
Servo servoAlcohol;
// Servo servoGauze; // [DISABLED: 2-item dispensing only]

const size_t SERIAL_BUF_SIZE = 64;
char serialBuffer[SERIAL_BUF_SIZE];
size_t bufferIndex = 0;
bool commandReady = false;

void processCommand(const char* cmd);
void dispenseSupplies(bool dispenseBandage, bool dispenseAlcohol/*, bool dispenseGauze*/);
void runDispenserCycle(Servo& servo, uint8_t pin);

/**
 * @brief Initializes Arduino I/O pins, serial communication, and blinks status LED.
 *
 * Configures the status LED pin as output, establishes Serial at 9600 baud,
 * resets servo trigger lines, emits STATUS:READY to the host Raspberry Pi,
 * and performs a 3-blink initialization sequence.
 */
void setup() {
  pinMode(PIN_LED_STATUS, OUTPUT);
  digitalWrite(PIN_LED_STATUS, LOW);

  Serial.begin(9600);
  while (!Serial && millis() < 3000) {
  }

  digitalWrite(PIN_SERVO_BANDAGE, LOW);
  digitalWrite(PIN_SERVO_ALCOHOL, LOW);
  // digitalWrite(PIN_SERVO_GAUZE, LOW); // [DISABLED: 2-item dispensing only]

  Serial.println("STATUS:READY");

  for (int i = 0; i < 3; i++) {
    digitalWrite(PIN_LED_STATUS, HIGH);
    delay(100);
    digitalWrite(PIN_LED_STATUS, LOW);
    delay(100);
  }
}

/**
 * @brief Main polling loop for receiving framing packets from the serial interface.
 *
 * Reads incoming characters into a static buffer framed by '<' and '>'.
 * Once a full packet is framed, triggers command processing via processCommand().
 */
void loop() {
  while (Serial.available() > 0) {
    char c = Serial.read();

    if (c == '<') {
      bufferIndex = 0;
      serialBuffer[bufferIndex++] = c;
    } else if (c == '>') {
      if (bufferIndex < SERIAL_BUF_SIZE - 1) {
        serialBuffer[bufferIndex++] = c;
        serialBuffer[bufferIndex] = '\0';
        commandReady = true;
      }
      break;
    } else if (c == '\n' || c == '\r') {
      continue;
    } else {
      if (bufferIndex > 0 && bufferIndex < SERIAL_BUF_SIZE - 1) {
        serialBuffer[bufferIndex++] = c;
      }
    }
  }

  if (commandReady) {
    processCommand(serialBuffer);
    commandReady = false;
    bufferIndex = 0;
  }
}

/**
 * @brief Parses and executes received protocol commands.
 *
 * Supported commands:
 *  - <DISP:b,a>: Parses binary flags for bandage and alcohol pad (gauze pad disabled).
 *                Sends ACK:DISP:b,a and initiates servo actuation or holds.
 *  - <PING>: Responds with PONG for heartbeat/liveness testing.
 *  - Unknown commands emit ERR:UNKNOWN_COMMAND:<cmd>.
 *
 * @param cmd Null-terminated character string containing the framed command.
 */
void processCommand(const char* cmd) {
  int b = 0, a = 0;
  // int g = 0; // [DISABLED: 3rd item gauze pad]

  if (sscanf(cmd, "<DISP:%d,%d>", &b, &a) == 2) {
    bool doBandage = (b == 1);
    bool doAlcohol = (a == 1);
    // bool doGauze = (g == 1);

    Serial.print("ACK:DISP:");
    Serial.print(doBandage ? "1," : "0,");
    Serial.println(doAlcohol ? "1" : "0");
    // Serial.println(doGauze ? "1" : "0");

    if (doBandage || doAlcohol /*|| doGauze*/) {
      dispenseSupplies(doBandage, doAlcohol /*, doGauze*/);
    } else {
      Serial.println("STATUS:HOLD_ALL");
    }
  } else if (strcmp(cmd, "<PING>") == 0) {
    Serial.println("PONG");
  } else {
    Serial.print("ERR:UNKNOWN_COMMAND:");
    Serial.println(cmd);
  }
}

/**
 * @brief Coordinates sequential dispensing of requested first-aid items.
 *
 * Turns on the active indicator LED and executes runDispenserCycle() for each
 * enabled item in sequence, emitting status strings over Serial.
 *
 * @param doBandage Whether to actuate the bandage dispenser servo.
 * @param doAlcohol Whether to actuate the alcohol pad dispenser servo.
 * // @param doGauze   Whether to actuate the gauze pad dispenser servo. [DISABLED: 2-item dispensing only]
 */
void dispenseSupplies(bool doBandage, bool doAlcohol/*, bool doGauze*/) {
  digitalWrite(PIN_LED_STATUS, HIGH);

  if (doBandage) {
    Serial.println("STATUS:DISPENSING_BANDAGE");
    runDispenserCycle(servoBandage, PIN_SERVO_BANDAGE);
  }

  if (doAlcohol) {
    Serial.println("STATUS:DISPENSING_ALCOHOL");
    runDispenserCycle(servoAlcohol, PIN_SERVO_ALCOHOL);
  }

  // [DISABLED: 3rd item gauze pad]
  // if (doGauze) {
  //   Serial.println("STATUS:DISPENSING_GAUZE");
  //   runDispenserCycle(servoGauze, PIN_SERVO_GAUZE);
  // }

  digitalWrite(PIN_LED_STATUS, LOW);
  Serial.println("STATUS:DISPENSE_COMPLETE");
}

/**
 * @brief Drives an individual continuous rotation/position servo through a push-and-retract cycle.
 *
 * Attaches the servo to its control pin, drives it forward (TIME_PUSH_MS) to eject supply,
 * pauses briefly (TIME_PAUSE_MS), drives in reverse (TIME_RETRACT_MS) to reset mechanism,
 * and detaches the servo to prevent continuous power draw and jitter.
 *
 * @param servo Reference to the Servo object.
 * @param pin   PWM/Digital pin number associated with the servo.
 */
void runDispenserCycle(Servo& servo, uint8_t pin) {
  servo.attach(pin);
  delay(50);

  servo.write(SERVO_FORWARD);
  delay(TIME_PUSH_MS);

  servo.write(SERVO_STOP);
  delay(TIME_PAUSE_MS);

  servo.write(SERVO_REVERSE);
  delay(TIME_RETRACT_MS);

  servo.write(SERVO_STOP);
  delay(50);
  servo.detach();
}
