use std::io::Read;


pub trait Gateway{
	fn on_connect(&mut self, new_stream: &std::net::TcpStream);
	fn on_disconnect(&mut self);
	fn on_receive(&mut self, msg: &[u8]);
}



pub struct TcpConnector<G: Gateway>{
	gateway: G,
}

impl<G: Gateway+Send+'static> TcpConnector<G>{
	pub fn new(gateway: G) -> TcpConnector<G>{
		TcpConnector{
			gateway: gateway,
		}
	}

	pub fn start_connect<A: std::net::ToSocketAddrs+Clone+Send+'static>(mut self, address: A) -> std::thread::JoinHandle<()>{
		std::thread::spawn(move ||{ self.connect_loop(address); })
	}

	pub fn start_read(mut self, stream: std::net::TcpStream) -> std::thread::JoinHandle<()>{
		std::thread::spawn(move ||{ self.read_loop(stream); })
	}

	fn connect_loop<A: std::net::ToSocketAddrs+Clone>(&mut self, addr: A){
		loop{
			match std::net::TcpStream::connect(addr.clone()){
			Ok(stream) => self.read_loop(stream),
			Err(err) => {
				println!("TcpStream::connect() failed with: {}", err);
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
			}
		}
	}

	fn read_loop(&mut self, stream: std::net::TcpStream){
		self.gateway.on_connect(&stream);
		let mut reader = {
			match stream.try_clone(){
			Ok(stream_in) => std::io::BufReader::new(stream_in),
			Err(err) => {
				println!("TcpStream.clone() failed with: {}", err);
				return;
			}
			}
		};
		loop{
			let mut buf = std::vec![0; 1024];
			match reader.read(&mut buf){
			Ok(n) => {
				if n == 0{
					println!("EOF");
					break;
				}
				buf.resize(n, 0);
				self.gateway.on_receive(&buf);
			},
			Err(err) => println!("BufReader.read() failed with: {}", err)
			}
		}
		self.gateway.on_disconnect();
	}
}
