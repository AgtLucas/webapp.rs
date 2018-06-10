//! The main backend interface

use failure::Error;
use serde_json;
use shared::{LoginResponseData, WsMessage};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use tungstenite::protocol::Message;
use tungstenite::protocol::Message::*;
use tungstenite::server::accept;
use tungstenite::WebSocket;

/// The websocket server instance
pub struct Server;

impl Server {
    /// Start the websocket server
    pub fn run() -> Result<(), Error> {
        let addr = "0.0.0.0:30000";
        info!("Starting server at {}", addr);
        let server = TcpListener::bind(addr)?;
        for stream in server.incoming() {
            spawn(move || match stream {
                Err(e) => error!("Unable to accept stream: {}", e),
                Ok(s) => match accept(s) {
                    Err(e) => error!("Unable to accept websocket connection: {}", e),
                    Ok(mut ws) => loop {
                        match ws.read_message() {
                            Err(e) => {
                                error!("Unable to read message: {}", e);
                                break;
                            }
                            Ok(msg) => {
                                debug!("Received message: {}", msg);
                                Self::handle_message(&mut ws, msg);
                            }
                        }
                    },
                },
            });
        }
        Ok(())
    }

    fn handle_message(ws: &mut WebSocket<TcpStream>, msg: Message) {
        match msg {
            Binary(b) => {
                let request: Result<WsMessage, _> = serde_json::from_slice(&b);
                match request {
                    Err(e) => error!("Unable to interpret message: {}", e),
                    Ok(WsMessage::LoginRequest(d)) => {
                        // Check for a login request
                        debug!("User {} is trying to auth", d.username);

                        // Write the response
                        let response_data = WsMessage::LoginResponse(LoginResponseData { success: true });
                        match serde_json::to_vec(&response_data) {
                            Err(e) => error!("Unable to serialize reponse data: {}", e),
                            Ok(login_response) => {
                                let msg = Message::from(login_response);
                                if let Err(e) = ws.write_message(msg) {
                                    error!("Unable to write message: {}", e);
                                }
                            }
                        }
                    }
                    _ => warn!("Unsuppored message type"),
                }
            }
            _ => warn!("No binary message"),
        }
    }
}