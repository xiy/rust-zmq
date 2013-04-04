extern mod std;
extern mod zmq;

fn new_server(ctx: zmq::Context) {
    io::println("starting server");

    let socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://127.0.0.1:3456").get();

    let msg = socket.recv_str(0).unwrap();
    io::println(fmt!("server received %?", msg));

    let s = fmt!("hello %s", msg);
    io::println(fmt!("server sending %?", s));
    match socket.send_str(s, 0) {
        Ok(()) => { }
        Err(e) => fail!(e.to_str())
    }

    // Let the main thread know we're done.
    let finished_socket = ctx.socket(zmq::PAIR).unwrap();
    finished_socket.connect("inproc://finished");
    finished_socket.send_str("done", 0);
}

fn new_client(ctx: zmq::Context) {
    io::println("starting client");

    let socket = ctx.socket(zmq::REQ).unwrap();

    io::println(fmt!("hwm: %?", socket.get_sndhwm().get()));
    socket.set_sndhwm(10).get();
    io::println("here!");
    io::stdout().flush();
    io::println(fmt!("hwm: %?", socket.get_sndhwm().get()));

    socket.set_identity(str::to_bytes("identity")).get();

    let identity = socket.get_identity().unwrap();
    io::println(fmt!("client identity: %s", str::from_bytes(identity)));

    io::println("client connecting to server");
    socket.connect("tcp://127.0.0.1:3456").get();

    let s = "foo";
    io::println(fmt!("client sending %?", s));
    socket.send_str(s, 0).get();

    io::println(fmt!("client received %?", socket.recv_str(0).unwrap()));
    socket.close();
}

fn main() {
    let (major, minor, patch) = zmq::version();

    io::println(fmt!("version: %d %d %d", major, minor, patch));

    let ctx = zmq::init(1).unwrap();

    let finished_socket = ctx.socket(zmq::PAIR).unwrap();
    finished_socket.bind("inproc://finished");

    // We need to start the server in a separate scheduler as it blocks.
    do task::spawn_sched(task::SingleThreaded) {
        new_server(ctx)
    }

    new_client(ctx);

    // Wait for the server to shut down.
    let s = finished_socket.recv_str(0);
    io::println(fmt!("%?", s));
    finished_socket.close();

    ctx.term().get();
}
