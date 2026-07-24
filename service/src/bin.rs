use interprocess::local_socket::tokio::Stream;
fn check(stream: &Stream) { let _ = stream.peer_pid(); }
