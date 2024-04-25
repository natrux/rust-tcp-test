pub trait Server{
	fn on_accept(&mut self, stream: std::net::TcpStream);
}



pub struct TcpAcceptor<S: Server>{
	server: S,
}

impl<S: Server+Send+'static> TcpAcceptor<S>{
	pub fn new(server: S) -> TcpAcceptor<S>{
		TcpAcceptor{
			server: server,
		}
	}

	pub fn start<A: std::net::ToSocketAddrs+Clone+Send+'static>(mut self, address: A) -> std::thread::JoinHandle<()>{
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
