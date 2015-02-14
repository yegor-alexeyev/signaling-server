extern crate websocket;
extern crate hyper;

use std::thread::Thread;
use std::old_io::{IoResult, Listener, Acceptor};
use websocket::{Server, Message};
use websocket::ws::receiver::Receiver as Reseiver_unused;
use websocket::ws::sender::Sender as Sender_unused;
use websocket::server::{Request, Sender, Receiver};
use websocket::stream::WebSocketStream;
use websocket::result::WebSocketResult;
use websocket::dataframe::DataFrame;
use std::collections::HashMap;

struct RoomMeta {
  sender: Sender<WebSocketStream>,
  receiver: Receiver<WebSocketStream>
}



fn process_request_holder(request_holder: IoResult<Request<WebSocketStream, WebSocketStream>>) -> WebSocketResult<Request<WebSocketStream, WebSocketStream>> {
  let mut request = try!(request_holder);
  
  let peer_name = request.get_mut_reader().peer_name().unwrap();
  println!("Received WebSocket connection from {}", peer_name);

  try!(request.validate());

  return Ok(request);
}

fn main() {
    let server = Server::bind("192.168.1.1:2794").unwrap();

    let mut rooms: HashMap<String, RoomMeta> = HashMap::new();
    let mut acceptor = server.listen().unwrap();
    for request_holder in acceptor.incoming() {

    match process_request_holder(request_holder) {
      Ok(request) => {
        match request.url.clone() {
          hyper::uri::RequestUri::AbsolutePath(path) => {
                let response = request.accept(); // Form a response
                let client = response.send().unwrap(); // Send the response
                let (mut sender, mut receiver) = client.split();

                  if rooms.contains_key(&path) {
                      let other = rooms.remove(&path).unwrap();
                      let mut other_receiver = other.receiver;
                      let mut other_sender = other.sender;
                      Thread::spawn(move || {
                              for message in receiver.incoming_dataframes() {
                                println!("got message from second");
      //                let message: WebSocketResult<Message> = receiver.recv_message();
                                match message {
                                  Ok(message) => {
                                    other_sender.send_dataframe(message);
                                    println!("send message to first");
                                  }
                                  Err(error) => {
                                    println!("other_sender done");
                                  }
                                }
                              }
                      });
                              for message in other_receiver.incoming_dataframes() {
                                println!("got message from first");
                                match message {
                                  Ok(message) => {
                                    sender.send_dataframe(message);
                                    println!("send message to second");
                                  }
                                  Err(error) => {
                                    println!("sender done");
                                  }
                                }
                              }
                    }
                    else {
                      sender.send_message(Message::Text("{ \"variant\":\"RequestOfOffer\" }".to_string()));
                      rooms.insert(path, RoomMeta{ sender: sender, receiver: receiver});
                    }
          }
          _ => {
            println!("Invalid or unsupported request url: ");
          }
        }
      }
      Err(error) => {
        println!("Error receiving connection: {}", error);
      }
    }
	}
}
