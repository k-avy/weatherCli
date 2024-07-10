// src/main.rs
use serde::{Serialize, Deserialize};
use std::env;
use reqwest::blocking::Client;
use tiny_http::{Server, Response, Method, StatusCode};

#[derive(Serialize, Deserialize)]
struct Weather {
    temperature: f32,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct OpenWeatherMapResponse {
    main: Main,
    weather: Vec<WeatherDescription>,
}

#[derive(Serialize, Deserialize)]
struct Main {
    temp: f32,
}

#[derive(Serialize, Deserialize)]
struct WeatherDescription {
    description: String,
}
fn get_weather(api_key: &str, city: &str) -> Result<Weather, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    let resp: OpenWeatherMapResponse = client.get(&url).send()?.json()?;
    
    let weather = Weather {
        temperature: resp.main.temp,
        description: resp.weather[0].description.clone(),
    };
    
    Ok(weather)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <API_KEY> <CITY>", args[0]);
        std::process::exit(1);
    }
    let api_key = &args[1];
    let city = &args[2];
    
    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Server running on port 8000");

    for request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (&Method::Get, "/weather") => {
                match get_weather(api_key, city) {
                    Ok(weather) => {
                        let weather_json = serde_json::to_string(&weather).unwrap();
                        Response::from_string(weather_json)
                            .with_status_code(StatusCode(200))
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    }
                    Err(_) => Response::from_string("Error fetching weather data")
                            .with_status_code(StatusCode(500)),
                }
            },
            _ => Response::from_string("404 Not Found")
                    .with_status_code(StatusCode(404)),
        };
        request.respond(response).unwrap();
    }
}
