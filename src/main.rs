#![allow(incomplete_features)]
#![feature(
    asm_const,
    asm_sym,
    asm_unwind,
    asm_experimental_arch,
    cfg_sanitize,
    cfg_target_abi,
    cfg_target_compact,
    cfg_target_has_atomic,
    cfg_target_has_atomic_equal_alignment,
    cfg_target_thread_local,
    cfg_version,
    async_closure,
    unboxed_closures,
    closure_lifetime_binder,
    closure_track_caller,
    extern_types,
    generic_arg_infer,
    generic_associated_types,
    const_async_blocks,
    const_eval_limit,
    const_extern_fn,
    const_fn_floating_point_arithmetic,
    const_for,
    const_mut_refs,
    const_precise_live_drops,
    const_refs_to_cell,
    const_trait_impl,
    const_try,
    generators,
    generator_trait,
    deprecated_safe,
    deprecated_suggestion,
    auto_traits,
    fn_traits,
    inline_const,
    inline_const_pat,
    decl_macro,
    box_syntax,
    box_patterns,
    try_blocks,
    if_let_guard,
    let_else,
    negative_impls,
    yeet_expr,
    exclusive_range_pattern,
    half_open_range_patterns,
    exhaustive_patterns,
    arbitrary_enum_discriminant,
    c_unwind,
    c_variadic
)]


use bakalib::socket::{Client, Server, ServerBuilder};
use bakalib::command::CommandParser;
use bakalib::io::Send;
use bakalib::utils::*;
use bakalib::protoutils;
use bakalib::set_interval;
use bakaproto::proto::*;

use std::io::prelude::*;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

use protobuf::Message;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = ServerBuilder::new("127.0.0.1:20030");

    println!("# bakaserver 0.1");

    server.event("on_client_connect", box |server: Arc<Mutex<&mut Server>>,
                                           client: Arc<Mutex<&mut Client>>,
                                           data: Result<message::Message, bakalib::socket::Error>| {
        let (server, client) = (server.clone(), client.clone());
        let (server, mut client) = (server.lock().unwrap(), client.lock().unwrap());

        println!("+{:#?}", client.socket.address);

        let client_address = client.socket.address.clone();

        client.socket.send_bytes(
            protoutils::BakaMessage {
                author: server.address.to_string(),
                content: format!(":{} 002 :Succefully connected, please register", client_address).to_string()
            }
                .build()
                .write_to_bytes()
                .unwrap()
        );
    });

    server.event("on_client_disconnect", box |server: Arc<Mutex<&mut Server>>,
                                              client: Arc<Mutex<&mut Client>>,
                                              data: Result<message::Message, bakalib::socket::Error>| {
        let client = client.clone();
        let client = client.lock().unwrap();

        println!("-{:#?}", client.socket.address);
    });

    server.event("on_error", box |server: Arc<Mutex<&mut Server>>,
                                  client: Arc<Mutex<&mut Client>>,
                                  data: Result<message::Message, bakalib::socket::Error>| {
        println!("Error");
    });

    server.event("on_message", box |server: Arc<Mutex<&mut Server>>,
                                    client: Arc<Mutex<&mut Client>>,
                                    data: Result<message::Message, bakalib::socket::Error>| {
        let (server, mut client) = (server.clone(), client.clone());

        if let Ok(message) = data {
            let mut parser = CommandParser::new(message.content.clone());
            let (mut server, mut client) = (server.lock().unwrap(), client.lock().unwrap());

            println!("({0}) [{1}] {2}", parser.command(), message.author.as_str(), message.content.as_str());

            match parser.target() {
                // Сервис регистрации сессий
                ":userserv" => {
                    match parser.command() {
                        // Зарегистрировать сессию
                        "register-session" => {
                            if !client.has_flag("tried_register-session") {
                                client.add_flag("tried_register-session", "true");
                                client.add_flag("registered", "true");

                                client.socket.send_bytes(
                                    protoutils::BakaMessage {
                                        author: server.address.to_string(),
                                        content: format!(":{} 003 :Successfully registered", message.author.as_str()).to_string()
                                    }
                                    .build()
                                    .write_to_bytes()
                                    .unwrap()
                                );
                                client.socket.send_bytes(
                                    protoutils::BakaMessage {
                                        author: server.address.to_string(),
                                        content: format!(":{:} 004 :Welcome", message.author.as_str()).to_string()
                                    }
                                    .build()
                                    .write_to_bytes()
                                    .unwrap()
                                );
                            }
                        }

                        // Уничтожить сессию
                        // Эта команда тут только для того, чтобы на сервере не было ошибок
                        // В реальности клиент никогда не успеет отправить её
                        "destroy-session" => {}

                        &_ => {}
                    }
                }

                // Основные команды

                &_ => match parser.command() {
                    "message" => {
                        let target = parser.target().as_str();

                        match target {
                            "all" => server.broadcast(format!(":all message :{}", message.content.as_str())),
                            &_ => client.socket.send(target, message.content.as_str())
                        }
                    }

                    "enable-feature" => {
                        let (feature, target) =
                            (parser.args()[0].clone(), parser.args()[1].clone());

                        match target {
                            "all" => {
                                {
                                    let mut clients = server.clients.lock().unwrap();

                                    for (_name, value) in &mut *clients {
                                        value.add_flag(format!("feature-enabled:{}", feature));
                                    }
                                }
                                server.broadcast(format!(":all enable-feature {}", feature));
                            }
                            
                            &_ => {
                                client.add_flag(format!("feature-enabled:{}", feature));
                                client.socket.broadcast(format!(":{} enable-feature {}", target, feature));
                            }
                        }
                    }

                    "disable-feature" => {

                    }

                    &_ => {}
                }
            }
        }
    });

    server.startup();
    server.polling();

    Ok(())
}
