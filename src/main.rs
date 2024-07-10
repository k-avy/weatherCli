use serde::{Serialize, Deserialize};
use std::env;
use reqwest::blocking::Client;
use tiny_http::{Server, Response, Method, StatusCode};
use std::collections::HashMap;


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
    dotenv::dotenv().ok();
    let api_key = env::var("API_KEY").expect("API_KEY must be set");

    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Server running on port 8000");

    for request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (&Method::Get, url) if url.starts_with("/weather") => {
                let query_pairs: HashMap<_, _> = url.split('?')
                    .nth(1)
                    .unwrap_or("")
                    .split('&')
                    .filter_map(|s| {
                        let mut split = s.split('=');
                        if let (Some(k), Some(v)) = (split.next(), split.next()) {
                            Some((k, v))
                        } else {
                            None
                        }
                    })
                    .collect();

                if let Some(city) = query_pairs.get("city") {
                    match get_weather(&api_key, city) {
                        Ok(weather) => {
                            let weather_json = serde_json::to_string(&weather).unwrap();
                            Response::from_string(weather_json)
                                .with_status_code(StatusCode(200))
                                .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                        }
                        Err(_) => Response::from_string("Error fetching weather data")
                                .with_status_code(StatusCode(500)),
                    }
                } else {
                    Response::from_string("City not specified")
                        .with_status_code(StatusCode(400))
                }
            },
            _ => Response::from_string("404 Not Found")
                    .with_status_code(StatusCode(404)),
        };
        request.respond(response).unwrap();
    }
}
