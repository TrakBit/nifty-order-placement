use colored::Colorize;
use dotenv::dotenv;
use serde_json::json;
use std::env;
use std::error::Error;
use std::io::{self, BufRead};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut terminal = true;
    while terminal {
        let mut line = String::new();
        let stdin = io::stdin();
        println!("{}", "1. authorize");
        println!("{}", "2. login");
        println!("{}", "3. track");
        println!("{}", "4. exit");
        stdin.lock().read_line(&mut line).unwrap();
        let option = line.trim();

        let one = "1";
        let two = "2";
        let three = "3";
        let four = "4";

        if option == one {
            authorize();
        } else if option == two {
            login();
        } else if option == three {
            track();
        } else if option == four {
            terminal = false;
        }
    }
}

fn login() {
    async fn call() -> Result<(), Box<dyn Error>> {
        dotenv().ok();
        let code = env::var("CODE").unwrap().to_string();
        let client_id = env::var("CLIENT_ID").unwrap().to_string();
        let client_secret = env::var("API_SECRET").unwrap().to_string();
        let redirect_url = env::var("REDIRECT_URL").unwrap().to_string();

        let data = json!({
            "code": code,
            "client_id": client_id,
            "client_secret": client_secret,
            "redirect_uri":redirect_url,
            "grant_type": "authorization_code",
        });
        let resp = reqwest::Client::new()
            .post("https://api.upstox.com/v2/login/authorization/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("accept", "application/json")
            .query(&data)
            .send()
            .await?;
        let resp_text = resp.text().await?;
        let user_info: serde_json::Value = serde_json::from_str(&resp_text).unwrap();
        if let Some(access_token) = user_info.get("access_token") {
            println!("Access Token: {}", access_token);
        } else {
            println!("Error: access_token field not found");
        }
        Ok(())
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(call());
}

fn authorize() {
    dotenv().ok();
    let authorize_url = "https://api.upstox.com/v2/login/authorization/dialog/";
    let client_id_string = "?client_id=";
    let client_id = env::var("CLIENT_ID").unwrap().to_string();
    let redirect_uri_string = "&redirect_uri=";
    let redirect_url = env::var("REDIRECT_URL").unwrap().to_string();
    let url = [
        authorize_url,
        client_id_string,
        &client_id,
        redirect_uri_string,
        &redirect_url,
    ]
    .join("");
    println!("{}", url);
    let _ = open::that(url);
}

fn track() {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct Instrument {
        pub name: &'static str,
        pub ticker: &'static str,
        pub key: u32,
    }

    const INSTRUMENTS: &'static [Instrument] = &[
        Instrument {
            ticker: "NSE_INDEX:Nifty 50",
            name: "NSE_INDEX|Nifty 50",
            key: 1,
        },
        Instrument {
            ticker: "NSE_INDEX:Nifty Bank",
            name: "NSE_INDEX|Nifty Bank",
            key: 2,
        },
    ];

    let mut selected_instrument_no: u32 = 0;

    let mut selected_instrument_name = "";
    let mut selected_instrument_ticker = "";

    let mut instrument_terminal = true;
    while instrument_terminal {
        println!("{}", "Select instrument to track".red().bold());
        for instrument in INSTRUMENTS.iter() {
            println!("{}.{}", instrument.key, instrument.name);
        }
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Not a valid number");
        selected_instrument_no = match line.trim().parse() {
            Ok(num) => num,
            Err(_) => 0,
        };
        for instrument in INSTRUMENTS.iter() {
            if selected_instrument_no == instrument.key {
                selected_instrument_name = instrument.name;
                selected_instrument_ticker = instrument.ticker;
                instrument_terminal = false;
            }
        }
    }

    let mut target: f64 = 0.0;
    let mut target_terminal = true;
    while target_terminal {
        println!("{}", "Enter target to trigger".red().bold());
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Not a valid string");
        target = match line.trim().parse() {
            Ok(num) => num,
            Err(_) => 0.0,
        };
        if target == 0.0 {
            println!("{} is not a number", target)
        } else if target != 0.0 {
            println!("target set as {}", target);
            target_terminal = false;
        }
    }

    let mut logic: u32 = 0;
    let mut logic_terminal = true;
    while logic_terminal {
        println!("{}", "Select price alert condition".red().bold());
        println!("1. {} greater than {}", selected_instrument_name, target);
        println!("2. {} less than {}", selected_instrument_name, target);
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Not a valid number");
        logic = match line.trim().parse() {
            Ok(num) => num,
            Err(_) => 0,
        };
        if logic == 1 || logic == 2 {
            logic_terminal = false;
        }
    }

    let mut scan_terminal = true;
    while scan_terminal {
        sleep(Duration::from_millis(500));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let scanner = scan_target(
            target,
            &selected_instrument_name,
            &selected_instrument_ticker,
            logic,
        );
        let continue_scan = match rt.block_on(scanner) {
            Ok(result) => result,
            Err(_) => false,
        };
        scan_terminal = continue_scan;
    }

    async fn scan_target(
        target: f64,
        selected_instrument_name: &str,
        selected_instrument_ticker: &str,
        logic: u32,
    ) -> Result<bool, Box<dyn Error>> {
        let mut _scan_terminal: bool = true;
        dotenv().ok();
        let data = json!({
            "instrument_key": selected_instrument_name
        });
        let access_token = env::var("ACCESS_TOKEN").unwrap().to_string();
        let bearer = "Bearer";
        let authorization = [bearer, &access_token].join(" ");
        let resp = reqwest::Client::new()
            .get("https://api.upstox.com/v2/market-quote/ltp")
            .header("Authorization", authorization)
            .header("accept", "application/json")
            .query(&data)
            .send()
            .await?;
        let resp_text = resp.text().await?;
        let instrument: serde_json::Value = serde_json::from_str(&resp_text).unwrap();

        // Extract the "last_price" field from the JSON response
        if let Some(data) = instrument.get("data") {
            if let Some(nifty_data) = data.get(selected_instrument_ticker) {
                if let Some(last_price) = nifty_data.get("last_price") {
                    if let Some(last_price_f64) = last_price.as_f64() {
                        println!("Last Price: {}", last_price_f64);
                        if logic == 1 {
                            if last_price_f64 > target {
                                println!("{}", "alert triggered");
                                _scan_terminal = false;
                            }
                        } else if logic == 2 {
                            if last_price_f64 < target {
                                println!("{}", "alert triggered");
                                _scan_terminal = false
                            }
                        }
                    } else {
                        println!("Error: last_price is not a valid f64");
                    }
                } else {
                    println!("Error: last_price field not found");
                }
            } else {
                println!("Error: NSE_INDEX:Nifty 50 field not found");
            }
        } else {
            println!("Error: data field not found");
        }
        Ok(_scan_terminal)
    }
}
