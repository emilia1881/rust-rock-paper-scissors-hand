use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::uart::{UartDriver, config::Config as UartConfig};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi};
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use std::sync::{Arc, Mutex};


#[derive(Clone)]
struct Player {
    name: heapless::String<32>,
    score: i32,
    history: heapless::Vec<u8, 30>, 
}


struct GameState {
    players: heapless::Vec<Player, 15>,
    current_player: Option<usize>,
    last_player_move: heapless::String<16>,
    last_robot_move: heapless::String<16>,
    last_winner: heapless::String<16>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            players: heapless::Vec::new(),
            current_player: None,
            last_player_move: heapless::String::new(),
            last_robot_move: heapless::String::new(),
            last_winner: heapless::String::new(),
        }
    }


    fn predict_and_play(&self, _player_idx: usize) -> &'static str {
        let rand_val = unsafe { esp_idf_svc::sys::esp_random() };
        match rand_val % 3 {
            0 => "ROCK",
            1 => "PAPER",
            _ => "SCISSORS",
        }
    }

    fn gesture_to_u8(g: &str) -> Option<u8> {
        match g {
            "ROCK" => Some(0),
            "PAPER" => Some(1),
            "SCISSORS" => Some(2),
            _ => None,
        }
    }

    fn who_wins(player: &str, robot: &str) -> &'static str {
        match (player, robot) {
            ("ROCK", "SCISSORS") | ("PAPER", "ROCK") | ("SCISSORS", "PAPER") => "PLAYER",
            (a, b) if a == b => "DRAW",
            _ => "ROBOT",
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: "RoboHand".try_into().unwrap(),
        password: "robohand123".try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;
    log::info!("WiFi AP started! IP: 192.168.71.1");


    let uart_config = UartConfig::new().baudrate(Hertz(115_200));
    let uart = UartDriver::new(
        peripherals.uart2,
        peripherals.pins.gpio17,                             
        peripherals.pins.gpio16,                              
        Option::<esp_idf_svc::hal::gpio::AnyIOPin>::None,
        Option::<esp_idf_svc::hal::gpio::AnyIOPin>::None,
        &uart_config,
    )?;
    let uart = Arc::new(Mutex::new(uart));
    log::info!("UART2 started on GPIO17 (TX)");
  

    let state = Arc::new(Mutex::new(GameState::new()));
    let mut server = EspHttpServer::new(&HttpConfig::default())?;

    
    {
        let _state = state.clone();
        server.fn_handler("/", Method::Get, move |request| {
            let html = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>RoboHand Game</title>
<style>
  body { font-family: Arial, sans-serif; text-align: center; background: #1a1a2e; color: white; padding: 20px; }
  h1 { color: #e94560; }
  input { padding: 10px; font-size: 16px; border-radius: 8px; border: none; margin: 5px; }
  button { padding: 10px 20px; font-size: 16px; border-radius: 8px; border: none; background: #e94560; color: white; cursor: pointer; margin: 5px; }
  #players-list button { background: #16213e; border: 1px solid #e94560; }
  #game { display: none; }
  #result { font-size: 24px; margin: 20px; padding: 20px; background: #16213e; border-radius: 12px; }
  .score { font-size: 48px; color: #e94560; }
</style>
</head>
<body>
<h1> RoboHand</h1>
<div id="login">
  <h2>Who are you?</h2>
  <div id="players-list"></div>
  <br>
  <input type="text" id="new-name" placeholder="Write a new name...">
  <button onclick="login()">Lets go!</button>
</div>
<div id="game">
  <h2>Welcome, <span id="player-name"></span>!</h2>
  <p>Show the gesture to the camera</p>
  <div id="result">Waiting...</div>
  <p>Scor: <span class="score" id="score">0</span></p>
  <button onclick="logout()">Change player</button>
</div>
<script>
let currentPlayer = "";
async function loadPlayers() {
  const r = await fetch("/players");
  const data = await r.json();
  const div = document.getElementById("players-list");
  div.innerHTML = data.players.map(p =>
    `<button onclick="selectPlayer('${p.name}')">${p.name} (${p.score})</button>`
  ).join(" ");
}
function selectPlayer(name) {
  document.getElementById("new-name").value = name;
}
async function login() {
  const name = document.getElementById("new-name").value.trim();
  if (!name) return;
  await fetch("/login", { method: "POST", body: name });
  currentPlayer = name;
  document.getElementById("login").style.display = "none";
  document.getElementById("game").style.display = "block";
  document.getElementById("player-name").textContent = name;
  pollResult();
}
function logout() {
  document.getElementById("login").style.display = "block";
  document.getElementById("game").style.display = "none";
  loadPlayers();
}
async function pollResult() {
  while (document.getElementById("game").style.display !== "none") {
    const r = await fetch("/state");
    const data = await r.json();
    document.getElementById("score").textContent = data.score;
    if (data.last_player && data.last_robot) {
      const win = data.last_winner === "PLAYER" ? "You won!" :
                  data.last_winner === "DRAW" ? "Draw!" : "You have been beaten by RoboHand!";
      document.getElementById("result").innerHTML =
        `You: ${data.last_player} | Hand: ${data.last_robot}<br>${win}`;
    }
    await new Promise(r => setTimeout(r, 1000));
  }
}
loadPlayers();
</script>
</body>
</html>"#;
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok::<(), anyhow::Error>(())
        })?;
    }

    
    {
        let state = state.clone();
        server.fn_handler("/players", Method::Get, move |request| {
            let s = state.lock().unwrap();
            let mut json = String::from("{\"players\":[");
            for (i, p) in s.players.iter().enumerate() {
                if i > 0 { json.push(','); }
                json.push_str(&format!("{{\"name\":\"{}\",\"score\":{}}}", p.name, p.score));
            }
            json.push_str("]}");
            let mut resp = request.into_ok_response()?;
            resp.write_all(json.as_bytes())?;
            Ok::<(), anyhow::Error>(())
        })?;
    }

    
    {
        let state = state.clone();
        server.fn_handler("/login", Method::Post, move |mut request| {
            let mut buf = [0u8; 32];
            let n = request.read(&mut buf).unwrap_or(0);
            let name = std::str::from_utf8(&buf[..n]).unwrap_or("").trim();
            let mut s = state.lock().unwrap();
            let idx = s.players.iter().position(|p| p.name.as_str() == name);
            let idx = if let Some(i) = idx {
                i
            } else {
                let mut p = Player {
                    name: heapless::String::new(),
                    score: 0,
                    history: heapless::Vec::new(),
                };
                let _ = p.name.push_str(name);
                let _ = s.players.push(p);
                s.players.len() - 1
            };
            s.current_player = Some(idx);
            log::info!("Jucator activ: {}", name);
            request.into_ok_response()?.write_all(b"ok")?;
            Ok::<(), anyhow::Error>(())
        })?;
    }

    
    {
        let state = state.clone();
        let uart_g = uart.clone(); 
        server.fn_handler("/gesture", Method::Post, move |mut request| {
            let mut buf = [0u8; 16];
            let n = request.read(&mut buf).unwrap_or(0);
            let gesture = std::str::from_utf8(&buf[..n]).unwrap_or("").trim();
            log::info!("Gest primit: {}", gesture);
            let mut s = state.lock().unwrap();
            if let Some(idx) = s.current_player {
                let robot_move = s.predict_and_play(idx);
                let winner = GameState::who_wins(gesture, robot_move);
                log::info!("Player: {} | Robot: {} | Winner: {}", gesture, robot_move, winner);

                if winner == "PLAYER" { s.players[idx].score += 1; }
                else if winner == "ROBOT" { s.players[idx].score -= 1; }

                s.last_player_move = heapless::String::new();
                let _ = s.last_player_move.push_str(gesture);
                s.last_robot_move = heapless::String::new();
                let _ = s.last_robot_move.push_str(robot_move);
                s.last_winner = heapless::String::new();
                let _ = s.last_winner.push_str(winner);

                if let Some(g) = GameState::gesture_to_u8(gesture) {
                    let _ = s.players[idx].history.push(g);
                }

            
                let msg: &[u8] = match robot_move {
                    "ROCK"     => b"ROCK\n",
                    "PAPER"    => b"PAPER\n",
                    _          => b"SCISSORS\n",
                };
                if let Ok(mut u) = uart_g.lock() {
                    let _ = u.write_all(msg);
                }
                log::info!("Sent to STM32: {}", robot_move);
                

                let result_msg: &[u8] = match winner {
                    "PLAYER" => b"WIN\n",
                    "DRAW"   => b"DRAW\n",
                    _        => b"LOSE\n",
                };
                if let Ok(mut u) = uart_g.lock() {
                    let _ = u.write_all(result_msg);
                }
                log::info!("Result sent to STM32: {}", winner);
               
            }
            request.into_ok_response()?.write_all(b"ok")?;
            Ok::<(), anyhow::Error>(())
        })?;
    }

    
    {
        let state = state.clone();
        server.fn_handler("/state", Method::Get, move |request| {
            let s = state.lock().unwrap();
            let json = if let Some(idx) = s.current_player {
                format!(
                    "{{\"score\":{},\"last_player\":\"{}\",\"last_robot\":\"{}\",\"last_winner\":\"{}\"}}",
                    s.players[idx].score,
                    s.last_player_move,
                    s.last_robot_move,
                    s.last_winner
                )
            } else {
                String::from("{\"score\":0,\"last_player\":\"\",\"last_robot\":\"\",\"last_winner\":\"\"}")
            };
            request.into_ok_response()?.write_all(json.as_bytes())?;
            Ok::<(), anyhow::Error>(())
        })?;
    }

    log::info!("Server started on 192.168.71.1");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}