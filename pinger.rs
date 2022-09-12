thread::spawn(move || {
let mut i = 0;

loop {
if i >= 2 && !client.flags.contains_key(&"registered".to_string()) {
client.socket.send_bytes(
protoutils::BakaMessage {
author: server.address.to_string(),
content: format!(":{} -003 :Registration timeout (pinged {} times)",
message.author.clone(), i).to_string()
}
.build()
.write_to_bytes()
.unwrap()
);
client.socket.shutdown();
}

let ping_cookie = String::random(12);

client.last_ping = protoutils::BakaMessage {
author: server.address.to_string(),
content: format!(":{} ping :{}", message.author, ping_cookie).to_string()
};

let last_ping = client.last_ping
.clone()
.build()
.write_to_bytes()
.unwrap();

client.socket.send_bytes(last_ping);

println!("TEST2 {}", client.last_ping.content);

i += 1;

thread::sleep(Duration::from_millis(1000));
}
});