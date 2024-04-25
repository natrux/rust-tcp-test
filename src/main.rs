use std::io::Read;
use std::io::Write;


trait Server{
	fn on_accept(&mut self, stream: std::net::TcpStream);
}


trait Gateway{
	fn on_connect(&mut self, new_stream: &std::net::TcpStream);
	fn on_disconnect(&mut self);
	fn on_receive(&mut self, msg: &[u8]);
}


struct TcpAcceptor<S: Server>{
	server: S,
}

impl<S: Server+Send+'static> TcpAcceptor<S>{
	fn new(server: S) -> TcpAcceptor<S>{
		TcpAcceptor{
			server: server,
		}
	}

	fn start<A: std::net::ToSocketAddrs+Clone+Send+'static>(mut self, address: A) -> std::thread::JoinHandle<()>{
		std::thread::spawn(move || { self.bind_loop(address); })
	}

	fn bind_loop<A: std::net::ToSocketAddrs+Clone>(&mut self, address: A){
		loop{
			match std::net::TcpListener::bind(address.clone()){
			Ok(listener) => self.accept_loop(listener),
			Err(err) => {
				println!("TcpListener::bind() failed with: {}", err);
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
			}
		}
	}

	fn accept_loop(&mut self, listener: std::net::TcpListener){
		for value in listener.incoming(){
			match value{
			Ok(stream) => {
				self.server.on_accept(stream);
			},
			Err(err) => {
				println!("TcpListener::accept() failed with: {}", err);
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
			};
		}
	}
}


struct TcpConnector<G: Gateway>{
	gateway: G,
}

impl<G: Gateway+Send+'static> TcpConnector<G>{
	fn new(gateway: G) -> TcpConnector<G>{
		TcpConnector{
			gateway: gateway,
		}
	}

	fn start_connect<A: std::net::ToSocketAddrs+Clone+Send+'static>(mut self, address: A) -> std::thread::JoinHandle<()>{
		std::thread::spawn(move ||{ self.connect_loop(address); })
	}

	fn start_read(mut self, stream: std::net::TcpStream) -> std::thread::JoinHandle<()>{
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

