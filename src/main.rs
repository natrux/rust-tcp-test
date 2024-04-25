use std::io::Write;

use rust_tcp_test::server::{Server, TcpAcceptor};
use rust_tcp_test::gateway::{Gateway, TcpConnector};



struct ReverseGateway{
	stream: Option<std::net::TcpStream>,
}

impl ReverseGateway{
	fn new() -> ReverseGateway{
		ReverseGateway{
			stream: None
		}
	}

	fn create() -> TcpConnector<Self>{
		TcpConnector::new(Self::new())
	}
}

impl Gateway for ReverseGateway{
	fn on_connect(&mut self, new_stream: &std::net::TcpStream){
		self.stream = new_stream.try_clone().ok();
	}

	fn on_disconnect(&mut self){
		self.stream = None;
	}

	fn on_receive(&mut self, msg: &[u8]){
		if let Some(stream) = self.stream.as_mut(){
			let response = {
				let mut v = msg.to_vec();
				v.reverse();
				v
			};
			if let Err(err) = stream.write_all(&response){
				println!("TcpStream.write_all() failed with: {}", err);
			}
		}
	}
}



struct ReverseServer{
}

impl ReverseServer{
	fn new() -> ReverseServer{
		ReverseServer{
		}
	}

	fn create() -> TcpAcceptor<Self>{
		TcpAcceptor::new(Self::new())
	}
}

impl Server for ReverseServer{
	fn on_accept(&mut self, stream: std::net::TcpStream){
		TcpConnector::new(ReverseGateway::new()).start_read(stream);
	}
}



fn main(){
	println!("Hello, world!");

	ReverseServer::create().start("localhost:1234");
	ReverseGateway::create().start_connect("localhost:1235");

	std::thread::sleep(std::time::Duration::from_secs(60));
}

