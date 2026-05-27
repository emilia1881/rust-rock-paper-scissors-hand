import cv2
import mediapipe as mp
import requests
import time

mp_hands = mp.solutions.hands
mp_draw = mp.solutions.drawing_utils

def finger_is_up(hand_landmarks, tip_id, pip_id):
    return hand_landmarks.landmark[tip_id].y < hand_landmarks.landmark[pip_id].y

hands = mp_hands.Hands(
    max_num_hands=1,
    min_detection_confidence=0.7,
    min_tracking_confidence=0.7
)

cap = cv2.VideoCapture(0)


STATE_WAITING = "waiting"
STATE_COUNTDOWN = "countdown"
STATE_SEND = "send"


state = STATE_WAITING
countdown_start = 0
last_gesture = "Unknown"

while True:
    success, frame = cap.read()
    if not success:
        break

    frame = cv2.flip(frame, 1)
    rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
    result = hands.process(rgb)
    gesture = "Unknown"

    if result.multi_hand_landmarks:
        for handLms in result.multi_hand_landmarks:
            mp_draw.draw_landmarks(frame, handLms, mp_hands.HAND_CONNECTIONS)
            index_up = finger_is_up(handLms, 8, 6)
            middle_up = finger_is_up(handLms, 12, 10)
            ring_up = finger_is_up(handLms, 16, 14)
            pinky_up = finger_is_up(handLms, 20, 18)
            up_count = sum([index_up, middle_up, ring_up, pinky_up])

            if up_count == 0:
                gesture = "ROCK"
            elif up_count == 4:
                gesture = "PAPER"
            elif index_up and middle_up and not ring_up and not pinky_up:
                gesture = "SCISSORS"

    
    if state == STATE_WAITING:
        cv2.putText(frame, "Press SPACE to start", (30, 50),
                    cv2.FONT_HERSHEY_SIMPLEX, 0.8, (0, 255, 0), 2)

    elif state == STATE_COUNTDOWN:
        elapsed = time.time() - countdown_start
        remaining = 3 - int(elapsed)

        if remaining > 0:
            cv2.putText(frame, str(remaining), (frame.shape[1]//2 - 30, frame.shape[0]//2),
                        cv2.FONT_HERSHEY_SIMPLEX, 4, (0, 0, 255), 6)
            cv2.putText(frame, gesture, (30, 50),
                        cv2.FONT_HERSHEY_SIMPLEX, 1.2, (0, 255, 0), 3)
            last_gesture = gesture
        else:
            state = STATE_SEND

    elif state == STATE_SEND:
        
        if last_gesture != "Unknown":
            try:
                r = requests.post("http://192.168.71.1/gesture",
                      data=last_gesture, timeout=2)
                print(f"Sent: {last_gesture} → response: {r.status_code}")
            except Exception as e:
                print(f"ERROR sending gesture: {e}")
        cv2.putText(frame, f"You played: {last_gesture}", (30, 50),
                    cv2.FONT_HERSHEY_SIMPLEX, 1.2, (255, 255, 0), 3)
        cv2.putText(frame, "Press SPACE for next round", (30, 100),
                    cv2.FONT_HERSHEY_SIMPLEX, 0.7, (0, 255, 0), 2)
        state = STATE_WAITING

    cv2.imshow("Rock Paper Scissors", frame)

    key = cv2.waitKey(1) & 0xFF
    if key == 27:  
        break
    elif key == 32:  
        if state == STATE_WAITING:
            countdown_start = time.time()
            state = STATE_COUNTDOWN

cap.release()
cv2.destroyAllWindows()